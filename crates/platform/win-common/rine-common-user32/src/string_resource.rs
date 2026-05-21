//! String resource operations shared by 32-bit and 64-bit user32 wrappers.

use rine_types::strings::LPSTR;

const IMAGE_DOS_SIGNATURE: u16 = 0x5A4D;
const IMAGE_NT_SIGNATURE: u32 = 0x0000_4550;
const IMAGE_NT_OPTIONAL_HDR32_MAGIC: u16 = 0x10B;
const IMAGE_NT_OPTIONAL_HDR64_MAGIC: u16 = 0x20B;

const RT_STRING: u32 = 6;
const RES_DIR_SIZE: usize = 16;
const RES_ENTRY_SIZE: usize = 8;

#[derive(Clone, Copy)]
enum ModuleLayout {
    MappedImage,
    FileImage,
}

#[derive(Clone, Copy)]
struct SectionHeader {
    virtual_address: u32,
    virtual_size: u32,
    raw_data_ptr: u32,
    raw_data_size: u32,
}

struct PeView<'a> {
    bytes: &'a [u8],
    layout: ModuleLayout,
    size_of_headers: u32,
    resource_rva: u32,
    sections: Vec<SectionHeader>,
}

/// LoadStringA - load an ANSI string resource into a caller-provided buffer.
///
/// This implementation performs a no-cache lookup of the PE resource directory
/// each call and returns a deterministic result. Unsupported IDs, missing
/// resources, invalid modules, and malformed resource structures all fail softly
/// with return value `0`.
///
/// # Safety
/// `lp_buffer` must be null or point to at least `cch_buffer_max` writable bytes.
pub unsafe fn load_string_a(
    hinstance: u32,
    u_id: u32,
    lp_buffer: LPSTR,
    cch_buffer_max: i32,
) -> i32 {
    if lp_buffer.is_null() || cch_buffer_max <= 0 {
        return 0;
    }

    let buffer = lp_buffer.as_mut_ptr();
    unsafe {
        *buffer = 0;
    }

    let maybe_ansi = if hinstance == 0 {
        load_string_from_main_module_file(u_id)
    } else {
        unsafe { load_string_from_mapped_module(hinstance as usize, u_id) }
    };

    let Some(ansi) = maybe_ansi else {
        return 0;
    };

    let copy_len = ansi.len().min((cch_buffer_max - 1) as usize);

    unsafe {
        std::ptr::copy_nonoverlapping(ansi.as_ptr(), buffer, copy_len);
        *buffer.add(copy_len) = 0;
    }

    copy_len as i32
}

unsafe fn load_string_from_mapped_module(module_base: usize, u_id: u32) -> Option<Vec<u8>> {
    let mapped_len = readable_contiguous_len_from(module_base)?;
    if mapped_len < 64 {
        return None;
    }

    let mapped = unsafe { std::slice::from_raw_parts(module_base as *const u8, mapped_len) };
    let pe = parse_pe_view(mapped, ModuleLayout::MappedImage)?;
    let utf16 = find_string_resource_utf16(&pe, u_id)?;
    Some(utf16_to_ansi_lossy(&utf16))
}

fn load_string_from_main_module_file(u_id: u32) -> Option<Vec<u8>> {
    let executable_path = guest_main_module_path()?;
    let bytes = std::fs::read(executable_path).ok()?;
    let pe = parse_pe_view(&bytes, ModuleLayout::FileImage)?;
    let utf16 = find_string_resource_utf16(&pe, u_id)?;
    Some(utf16_to_ansi_lossy(&utf16))
}

fn guest_main_module_path() -> Option<std::path::PathBuf> {
    std::env::args_os()
        .nth(1)
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::current_exe().ok())
}

fn parse_pe_view(bytes: &[u8], layout: ModuleLayout) -> Option<PeView<'_>> {
    if read_u16(bytes, 0)? != IMAGE_DOS_SIGNATURE {
        return None;
    }

    let pe_offset = read_u32(bytes, 0x3C)? as usize;
    if read_u32(bytes, pe_offset)? != IMAGE_NT_SIGNATURE {
        return None;
    }

    let file_header_offset = pe_offset.checked_add(4)?;
    let number_of_sections = read_u16(bytes, file_header_offset + 2)? as usize;
    let optional_size = read_u16(bytes, file_header_offset + 16)? as usize;

    let optional_offset = file_header_offset.checked_add(20)?;
    let optional_end = optional_offset.checked_add(optional_size)?;
    if optional_end > bytes.len() || optional_size < 96 {
        return None;
    }

    let optional_magic = read_u16(bytes, optional_offset)?;
    let data_directory_offset = match optional_magic {
        IMAGE_NT_OPTIONAL_HDR32_MAGIC => optional_offset.checked_add(96)?,
        IMAGE_NT_OPTIONAL_HDR64_MAGIC => optional_offset.checked_add(112)?,
        _ => return None,
    };

    let size_of_headers = read_u32(bytes, optional_offset + 60)?;
    let resource_rva = read_u32(bytes, data_directory_offset + 8 * 2)?;

    let section_table_offset = optional_offset.checked_add(optional_size)?;
    let mut sections = Vec::with_capacity(number_of_sections);
    for idx in 0..number_of_sections {
        let base = section_table_offset.checked_add(idx.checked_mul(40)?)?;
        let virtual_size = read_u32(bytes, base + 8)?;
        let virtual_address = read_u32(bytes, base + 12)?;
        let raw_data_size = read_u32(bytes, base + 16)?;
        let raw_data_ptr = read_u32(bytes, base + 20)?;

        sections.push(SectionHeader {
            virtual_address,
            virtual_size,
            raw_data_ptr,
            raw_data_size,
        });
    }

    Some(PeView {
        bytes,
        layout,
        size_of_headers,
        resource_rva,
        sections,
    })
}

fn find_string_resource_utf16(pe: &PeView<'_>, u_id: u32) -> Option<Vec<u16>> {
    if pe.resource_rva == 0 {
        return None;
    }

    let resource_root = rva_to_offset(pe, pe.resource_rva)?;
    let type_dir_entries = resource_directory_entries(pe.bytes, resource_root)?;

    let mut type_dir_rel = None;
    for entry in type_dir_entries {
        let type_id = entry.name_or_id;
        if (type_id & 0x8000_0000) != 0 || type_id != RT_STRING || !entry.points_to_directory {
            continue;
        }
        type_dir_rel = Some(entry.target_offset);
        break;
    }

    let type_dir_rel = type_dir_rel?;
    let type_dir_abs = add_offsets(resource_root, type_dir_rel as usize)?;
    let name_entries = resource_directory_entries(pe.bytes, type_dir_abs)?;

    let block_id = (u_id >> 4).checked_add(1)?;
    let mut lang_dir_rel = None;
    let mut leaf_data_rel = None;
    for entry in name_entries {
        if (entry.name_or_id & 0x8000_0000) != 0 || entry.name_or_id != block_id {
            continue;
        }

        if entry.points_to_directory {
            lang_dir_rel = Some(entry.target_offset);
        } else {
            leaf_data_rel = Some(entry.target_offset);
        }
        break;
    }

    let data_entry_rel = if let Some(leaf) = leaf_data_rel {
        leaf
    } else {
        let lang_dir_rel = lang_dir_rel?;
        let lang_dir_abs = add_offsets(resource_root, lang_dir_rel as usize)?;
        let lang_entries = resource_directory_entries(pe.bytes, lang_dir_abs)?;
        let mut candidates = Vec::new();

        for entry in lang_entries {
            if (entry.name_or_id & 0x8000_0000) != 0 || entry.points_to_directory {
                continue;
            }
            candidates.push((entry.name_or_id, entry.target_offset));
        }

        choose_lowest_lang_data_entry(&candidates)?
    };

    let data_entry_abs = add_offsets(resource_root, data_entry_rel as usize)?;
    let data_rva = read_u32(pe.bytes, data_entry_abs)?;
    let data_size = read_u32(pe.bytes, data_entry_abs + 4)? as usize;
    let data_offset = rva_to_offset(pe, data_rva)?;
    let data_end = data_offset.checked_add(data_size)?;
    let data = pe.bytes.get(data_offset..data_end)?;

    let slot = (u_id & 0xF) as usize;
    extract_string_slot_utf16(data, slot)
}

fn resource_directory_entries(bytes: &[u8], dir_abs: usize) -> Option<Vec<ResourceEntry>> {
    let named_count = read_u16(bytes, dir_abs + 12)? as usize;
    let id_count = read_u16(bytes, dir_abs + 14)? as usize;
    let total = named_count.checked_add(id_count)?;

    let mut entries = Vec::with_capacity(total);
    let entries_start = dir_abs.checked_add(RES_DIR_SIZE)?;
    for idx in 0..total {
        let base = entries_start.checked_add(idx.checked_mul(RES_ENTRY_SIZE)?)?;
        let name_or_id = read_u32(bytes, base)?;
        let data_or_dir = read_u32(bytes, base + 4)?;

        entries.push(ResourceEntry {
            name_or_id,
            points_to_directory: (data_or_dir & 0x8000_0000) != 0,
            target_offset: data_or_dir & 0x7FFF_FFFF,
        });
    }

    Some(entries)
}

struct ResourceEntry {
    name_or_id: u32,
    points_to_directory: bool,
    target_offset: u32,
}

fn choose_lowest_lang_data_entry(candidates: &[(u32, u32)]) -> Option<u32> {
    candidates
        .iter()
        .min_by_key(|(lang, _)| *lang)
        .map(|(_, offset)| *offset)
}

fn extract_string_slot_utf16(block_data: &[u8], slot: usize) -> Option<Vec<u16>> {
    if slot >= 16 {
        return None;
    }

    let mut cursor = 0usize;
    for idx in 0..16 {
        let len = read_u16(block_data, cursor)? as usize;
        cursor = cursor.checked_add(2)?;
        let byte_len = len.checked_mul(2)?;
        let raw = block_data.get(cursor..cursor.checked_add(byte_len)?)?;

        if idx == slot {
            let mut out = Vec::with_capacity(len);
            for pair in raw.chunks_exact(2) {
                out.push(u16::from_le_bytes([pair[0], pair[1]]));
            }
            return Some(out);
        }

        cursor = cursor.checked_add(byte_len)?;
    }

    None
}

fn utf16_to_ansi_lossy(input: &[u16]) -> Vec<u8> {
    std::char::decode_utf16(input.iter().copied())
        .map(|decoded| match decoded {
            Ok(ch) if (ch as u32) <= 0x7F => ch as u8,
            _ => b'?',
        })
        .collect()
}

fn rva_to_offset(pe: &PeView<'_>, rva: u32) -> Option<usize> {
    match pe.layout {
        ModuleLayout::MappedImage => {
            let offset = rva as usize;
            (offset < pe.bytes.len()).then_some(offset)
        }
        ModuleLayout::FileImage => {
            for section in &pe.sections {
                let section_start = section.virtual_address;
                let section_span = section.virtual_size.max(section.raw_data_size);
                let section_end = section_start.checked_add(section_span)?;

                if rva < section_start || rva >= section_end {
                    continue;
                }

                let within_section = rva.checked_sub(section_start)?;
                if within_section >= section.raw_data_size {
                    return None;
                }

                let raw = section.raw_data_ptr.checked_add(within_section)? as usize;
                return (raw < pe.bytes.len()).then_some(raw);
            }

            if rva < pe.size_of_headers {
                let header_offset = rva as usize;
                return (header_offset < pe.bytes.len()).then_some(header_offset);
            }

            None
        }
    }
}

fn add_offsets(base: usize, rel: usize) -> Option<usize> {
    base.checked_add(rel)
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    let raw = bytes.get(offset..offset.checked_add(2)?)?;
    Some(u16::from_le_bytes([raw[0], raw[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let raw = bytes.get(offset..offset.checked_add(4)?)?;
    Some(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

unsafe fn readable_contiguous_len_from(address: usize) -> Option<usize> {
    let maps = std::fs::read_to_string("/proc/self/maps").ok()?;
    let mut spans = Vec::new();

    for line in maps.lines() {
        let mut parts = line.split_whitespace();
        let range = parts.next()?;
        let perms = parts.next().unwrap_or_default();
        if !perms.as_bytes().first().is_some_and(|p| *p == b'r') {
            continue;
        }

        let (start_hex, end_hex) = range.split_once('-')?;
        let start = usize::from_str_radix(start_hex, 16).ok()?;
        let end = usize::from_str_radix(end_hex, 16).ok()?;
        if end > start {
            spans.push((start, end));
        }
    }

    spans.sort_unstable_by_key(|(start, _)| *start);
    let mut idx = spans
        .iter()
        .position(|(start, end)| *start <= address && address < *end)?;

    let mut current_end = spans[idx].1;
    while idx + 1 < spans.len() && spans[idx + 1].0 == current_end {
        idx += 1;
        current_end = spans[idx].1;
    }

    current_end.checked_sub(address)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lpstr_from_ptr(ptr: *mut u8) -> LPSTR {
        unsafe { std::mem::transmute(ptr) }
    }

    #[test]
    fn load_string_a_returns_zero_and_clears_buffer_when_id_missing() {
        let mut buffer = [b'X', b'Y', 0, b'Z'];
        let written = unsafe {
            load_string_a(
                0,
                1,
                lpstr_from_ptr(buffer.as_mut_ptr()),
                buffer.len() as i32,
            )
        };

        assert_eq!(written, 0);
        assert_eq!(buffer[0], 0);
    }

    #[test]
    fn load_string_a_returns_zero_for_null_buffer() {
        let written = unsafe { load_string_a(0, 1, LPSTR::NULL, 8) };
        assert_eq!(written, 0);
    }

    #[test]
    fn load_string_a_returns_zero_for_non_positive_capacity() {
        let mut buffer = [b'Q', 0];
        let written_zero = unsafe { load_string_a(0, 1, lpstr_from_ptr(buffer.as_mut_ptr()), 0) };
        let written_negative =
            unsafe { load_string_a(0, 1, lpstr_from_ptr(buffer.as_mut_ptr()), -1) };

        assert_eq!(written_zero, 0);
        assert_eq!(written_negative, 0);
        assert_eq!(buffer[0], b'Q');
    }

    #[test]
    fn choose_lowest_lang_data_entry_picks_lowest_langid() {
        let selected = choose_lowest_lang_data_entry(&[(0x0409, 11), (0x0000, 22), (0x0411, 33)]);
        assert_eq!(selected, Some(22));
    }

    #[test]
    fn utf16_to_ansi_lossy_replaces_non_ascii() {
        let input = [b'H' as u16, b'i' as u16, 0x03BB, 0xD83D, 0xDE00];
        let out = utf16_to_ansi_lossy(&input);
        assert_eq!(out, b"Hi??");
    }

    #[test]
    fn extract_string_slot_utf16_returns_expected_slot() {
        let mut data = Vec::new();
        for idx in 0..16u16 {
            data.extend_from_slice(&1u16.to_le_bytes());
            data.extend_from_slice(&(b'A' as u16 + idx).to_le_bytes());
        }

        let slot_three = extract_string_slot_utf16(&data, 3).unwrap();
        assert_eq!(slot_three, vec![b'D' as u16]);
    }
}
