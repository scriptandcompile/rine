//! PE resource table traversal to extract RT_GROUP_ICON and RT_ICON entries.
//!
//! The resource section contains a three-level directory tree:
//!   Level 1: resource type (we look for RT_GROUP_ICON = 14 and RT_ICON = 3)
//!   Level 2: resource name/ID
//!   Level 3: language variant
//!
//! Directory offsets in entries are relative to the start of the resource
//! section. Data entry `OffsetToData` values are image virtual addresses (RVAs)
//! that must be converted to file offsets via the enclosing section.

use goblin::pe::PE;

/// RT_ICON: individual icon bitmap data.
const RT_ICON: u32 = 3;
/// RT_GROUP_ICON: icon group directory referencing RT_ICON entries.
const RT_GROUP_ICON: u32 = 14;

/// Guard against maliciously deep or wide resource trees.
const MAX_DIR_ENTRIES: usize = 4096;

/// Upper bound on a single icon payload (16 MiB).
const MAX_ICON_BYTES: usize = 16 * 1024 * 1024;

// ── Resource tree structures (PE spec) ──────────────────────────────────────
// All structures are parsed manually from raw bytes.

// IMAGE_RESOURCE_DIRECTORY: 16 bytes (characteristics[4], timestamp[4], versions[4], named[2], id[2])
const RES_DIR_SIZE: usize = 16;
// IMAGE_RESOURCE_DIRECTORY_ENTRY: 8 bytes (name_or_id[4], data_or_subdir[4])
const RES_ENTRY_SIZE: usize = 8;
// IMAGE_RESOURCE_DATA_ENTRY: 16 bytes (rva[4], size[4], codepage[4], reserved[4])
const RES_DATA_ENTRY_SIZE: usize = 16;

// ── GRPICONDIR structures ────────────────────────────────────────────────────

/// A single entry within a GRPICONDIR, representing one icon variant.
#[derive(Clone)]
pub struct GroupIconEntry {
    /// Pixel width (0 in raw bytes means 256).
    pub width: u32,
    /// Pixel height (0 in raw bytes means 256).
    pub height: u32,
    pub bit_count: u16,
    /// RT_ICON resource ID for this variant's pixel data.
    pub id: u16,
}

/// Raw bytes for one RT_ICON entry.
pub struct IconData {
    pub id: u16,
    pub data: Vec<u8>,
}

pub struct ExtractedIcons {
    /// Entries from the first (best) GRPICONDIR found.
    pub group_entries: Vec<GroupIconEntry>,
    /// Icon data blobs indexed by the IDs referenced in `group_entries`.
    pub icons: Vec<IconData>,
}

// ── Section helpers ──────────────────────────────────────────────────────────

struct RsrcSection<'a> {
    /// Raw file bytes for the entire .rsrc section.
    data: &'a [u8],
    /// Section virtual address (RVA of section start within the image).
    sec_va: u32,
    /// Offset of the resource *directory* within the section.
    /// Usually 0; equals (rsrc_dir_rva - sec_va).
    dir_offset_in_sec: u32,
}

impl<'a> RsrcSection<'a> {
    /// Convert a resource-tree offset (relative to directory start) to a
    /// slice of `len` bytes from `self.data`.
    fn slice_at_dir_offset(&self, offset: u32, len: usize) -> Option<&'a [u8]> {
        let base = self.dir_offset_in_sec as usize;
        let start = base.checked_add(offset as usize)?;
        let end = start.checked_add(len)?;
        self.data.get(start..end)
    }

    /// Convert an RVA (from a `ResDataEntry.offset_to_data`) to a file-byte
    /// slice of `len` bytes.
    fn slice_at_rva(&self, rva: u32, len: usize) -> Option<&'a [u8]> {
        let offset = rva.checked_sub(self.sec_va)? as usize;
        let end = offset.checked_add(len)?;
        self.data.get(offset..end)
    }
}

// ── Public extraction entry point ────────────────────────────────────────────

pub fn extract_icons(file_bytes: &[u8]) -> Result<ExtractedIcons, crate::ThumbnailError> {
    let pe = PE::parse(file_bytes).map_err(|_| crate::ThumbnailError::MalformedResource)?;
    let rsrc = find_rsrc_section(file_bytes, &pe)?;

    let group_entries = collect_group_icon_entries(&rsrc)?;
    if group_entries.is_empty() {
        return Err(crate::ThumbnailError::NoIconResource);
    }

    let needed_ids: Vec<u16> = group_entries.iter().map(|e| e.id).collect();
    let icons = collect_icon_data(&rsrc, &needed_ids)?;

    Ok(ExtractedIcons { group_entries, icons })
}

// ── Locate the resource section ──────────────────────────────────────────────

fn find_rsrc_section<'a>(
    file_bytes: &'a [u8],
    pe: &PE<'_>,
) -> Result<RsrcSection<'a>, crate::ThumbnailError> {
    let opt = pe
        .header
        .optional_header
        .as_ref()
        .ok_or(crate::ThumbnailError::NoIconResource)?;

    let rsrc_dir = opt
        .data_directories
        .get_resource_table()
        .ok_or(crate::ThumbnailError::NoIconResource)?;

    let rsrc_rva = rsrc_dir.virtual_address;
    let rsrc_size = rsrc_dir.size as usize;

    if rsrc_rva == 0 || rsrc_size == 0 {
        return Err(crate::ThumbnailError::NoIconResource);
    }

    for section in &pe.sections {
        let sec_va = section.virtual_address;
        let sec_vsize = section.virtual_size;
        let sec_raw = section.pointer_to_raw_data as usize;
        let sec_raw_size = section.size_of_raw_data as usize;

        if rsrc_rva < sec_va || rsrc_rva >= sec_va.saturating_add(sec_vsize) {
            continue;
        }

        let dir_offset_in_sec = (rsrc_rva - sec_va) as usize;
        let sec_end = sec_raw
            .checked_add(sec_raw_size)
            .ok_or(crate::ThumbnailError::MalformedResource)?;

        if sec_end > file_bytes.len() {
            return Err(crate::ThumbnailError::MalformedResource);
        }

        return Ok(RsrcSection {
            data: &file_bytes[sec_raw..sec_end],
            sec_va,
            dir_offset_in_sec: dir_offset_in_sec as u32,
        });
    }

    Err(crate::ThumbnailError::NoIconResource)
}

// ── Directory helpers ────────────────────────────────────────────────────────

fn read_u16_le(data: &[u8], offset: usize) -> Option<u16> {
    let b = data.get(offset..offset + 2)?;
    Some(u16::from_le_bytes([b[0], b[1]]))
}

fn read_u32_le(data: &[u8], offset: usize) -> Option<u32> {
    let b = data.get(offset..offset + 4)?;
    Some(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

/// Read an `IMAGE_RESOURCE_DIRECTORY` header; return (num_named, num_id).
fn read_res_dir(rsrc: &RsrcSection, dir_offset: u32) -> Option<(u16, u16)> {
    let raw = rsrc.slice_at_dir_offset(dir_offset, RES_DIR_SIZE)?;
    let num_named = read_u16_le(raw, 12)?;
    let num_id = read_u16_le(raw, 14)?;
    Some((num_named, num_id))
}

/// Iterate entries of a directory node.  Each closure call receives
/// `(name_or_id, data_or_dir)` raw u32 values; returns `Err` early if the
/// closure returns `Err`.
fn iter_dir_entries(
    rsrc: &RsrcSection,
    dir_offset: u32,
    mut f: impl FnMut(u32, u32) -> Result<(), crate::ThumbnailError>,
) -> Result<(), crate::ThumbnailError> {
    let (num_named, num_id) = read_res_dir(rsrc, dir_offset)
        .ok_or(crate::ThumbnailError::MalformedResource)?;

    let total = (num_named as usize)
        .checked_add(num_id as usize)
        .ok_or(crate::ThumbnailError::MalformedResource)?;

    if total > MAX_DIR_ENTRIES {
        return Err(crate::ThumbnailError::MalformedResource);
    }

    for i in 0..total {
        let entry_offset = dir_offset as usize + RES_DIR_SIZE + i * RES_ENTRY_SIZE;
        let raw = rsrc
            .slice_at_dir_offset(entry_offset as u32, RES_ENTRY_SIZE)
            .ok_or(crate::ThumbnailError::MalformedResource)?;
        let name_or_id = read_u32_le(raw, 0)
            .ok_or(crate::ThumbnailError::MalformedResource)?;
        let data_or_dir = read_u32_le(raw, 4)
            .ok_or(crate::ThumbnailError::MalformedResource)?;
        f(name_or_id, data_or_dir)?;
    }
    Ok(())
}

/// Walk a level-2 directory (name level) and collect all data entries.
fn collect_data_entries(
    rsrc: &RsrcSection,
    l2_dir_offset: u32,
    out: &mut Vec<(u32, u32)>, // (rva, size)
) -> Result<(), crate::ThumbnailError> {
    iter_dir_entries(rsrc, l2_dir_offset, |_name_or_id, data_or_dir| {
        // Level 3: language directory
        if data_or_dir & 0x8000_0000 != 0 {
            let l3_offset = data_or_dir & 0x7FFF_FFFF;
            iter_dir_entries(rsrc, l3_offset, |_lang, leaf| {
                if leaf & 0x8000_0000 == 0 {
                    // leaf is an offset to IMAGE_RESOURCE_DATA_ENTRY
                    let raw = rsrc
                        .slice_at_dir_offset(leaf, RES_DATA_ENTRY_SIZE)
                        .ok_or(crate::ThumbnailError::MalformedResource)?;
                    let rva = read_u32_le(raw, 0).ok_or(crate::ThumbnailError::MalformedResource)?;
                    let size =
                        read_u32_le(raw, 4).ok_or(crate::ThumbnailError::MalformedResource)?;
                    out.push((rva, size));
                }
                Ok(())
            })?;
        }
        Ok(())
    })
}

// ── Collect group icon entries ────────────────────────────────────────────────

fn collect_group_icon_entries(
    rsrc: &RsrcSection,
) -> Result<Vec<GroupIconEntry>, crate::ThumbnailError> {
    let mut entries = Vec::new();

    iter_dir_entries(rsrc, 0, |name_or_id, data_or_dir| {
        // Level 1: resource type
        let type_id = name_or_id & 0x7FFF_FFFF;
        if name_or_id & 0x8000_0000 != 0 || type_id != RT_GROUP_ICON {
            return Ok(());
        }
        if data_or_dir & 0x8000_0000 == 0 {
            return Ok(());
        }
        let l2_offset = data_or_dir & 0x7FFF_FFFF;

        let mut data_entries: Vec<(u32, u32)> = Vec::new();
        collect_data_entries(rsrc, l2_offset, &mut data_entries)?;

        // Parse the first GRPICONDIR found.
        if let Some((rva, size)) = data_entries.into_iter().next() {
            if size as usize > MAX_ICON_BYTES {
                return Err(crate::ThumbnailError::MalformedResource);
            }
            let data = rsrc
                .slice_at_rva(rva, size as usize)
                .ok_or(crate::ThumbnailError::MalformedResource)?;
            entries = parse_grp_icon_dir(data)?;
        }
        Ok(())
    })?;

    Ok(entries)
}

/// Parse a GRPICONDIR blob into icon variant entries.
fn parse_grp_icon_dir(data: &[u8]) -> Result<Vec<GroupIconEntry>, crate::ThumbnailError> {
    if data.len() < 6 {
        return Err(crate::ThumbnailError::MalformedResource);
    }
    // Reserved (2) + Type (2) + Count (2)
    let count = read_u16_le(data, 4).ok_or(crate::ThumbnailError::MalformedResource)? as usize;

    let entry_size = 14; // GRPICONDIRENTRY
    let required = 6 + count * entry_size;
    if data.len() < required || count > MAX_DIR_ENTRIES {
        return Err(crate::ThumbnailError::MalformedResource);
    }

    let mut entries = Vec::with_capacity(count);
    for i in 0..count {
        let off = 6 + i * entry_size;
        let raw_width = data[off];
        let raw_height = data[off + 1];
        let width = if raw_width == 0 { 256 } else { raw_width as u32 };
        let height = if raw_height == 0 { 256 } else { raw_height as u32 };
        let bit_count = read_u16_le(data, off + 6).ok_or(crate::ThumbnailError::MalformedResource)?;
        // bytes_in_res at off+8 is informational only; skip it.
        let id = read_u16_le(data, off + 12).ok_or(crate::ThumbnailError::MalformedResource)?;
        entries.push(GroupIconEntry { width, height, bit_count, id });
    }
    Ok(entries)
}

// ── Collect RT_ICON data ──────────────────────────────────────────────────────

fn collect_icon_data(
    rsrc: &RsrcSection,
    needed_ids: &[u16],
) -> Result<Vec<IconData>, crate::ThumbnailError> {
    let mut icons = Vec::new();

    iter_dir_entries(rsrc, 0, |name_or_id, data_or_dir| {
        let type_id = name_or_id & 0x7FFF_FFFF;
        if name_or_id & 0x8000_0000 != 0 || type_id != RT_ICON {
            return Ok(());
        }
        if data_or_dir & 0x8000_0000 == 0 {
            return Ok(());
        }
        let l2_offset = data_or_dir & 0x7FFF_FFFF;

        // Level 2: icon ID entries.
        iter_dir_entries(rsrc, l2_offset, |icon_name_or_id, l2_data_or_dir| {
            if icon_name_or_id & 0x8000_0000 != 0 {
                return Ok(());
            }
            let icon_id = (icon_name_or_id & 0xFFFF) as u16;
            if !needed_ids.contains(&icon_id) {
                return Ok(());
            }
            if l2_data_or_dir & 0x8000_0000 == 0 {
                return Ok(());
            }
            let l3_offset = l2_data_or_dir & 0x7FFF_FFFF;

            // Level 3: language variants – take the first.
            let mut found = false;
            iter_dir_entries(rsrc, l3_offset, |_lang, leaf| {
                if found || leaf & 0x8000_0000 != 0 {
                    return Ok(());
                }
                let raw = rsrc
                    .slice_at_dir_offset(leaf, RES_DATA_ENTRY_SIZE)
                    .ok_or(crate::ThumbnailError::MalformedResource)?;
                let rva = read_u32_le(raw, 0).ok_or(crate::ThumbnailError::MalformedResource)?;
                let size = read_u32_le(raw, 4).ok_or(crate::ThumbnailError::MalformedResource)?;
                if size as usize > MAX_ICON_BYTES {
                    return Err(crate::ThumbnailError::MalformedResource);
                }
                let data = rsrc
                    .slice_at_rva(rva, size as usize)
                    .ok_or(crate::ThumbnailError::MalformedResource)?;
                icons.push(IconData { id: icon_id, data: data.to_vec() });
                found = true;
                Ok(())
            })?;
            Ok(())
        })?;
        Ok(())
    })?;

    Ok(icons)
}
