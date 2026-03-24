use std::ptr;
use std::ptr::NonNull;

use goblin::pe::relocation;
use goblin::pe::section_table;
use nix::sys::mman;
use rine_types::memory::{BaseAddress, FileOffset, RelativeVirtualAddress, VirtualAddress};
use thiserror::Error;
use tracing::{debug, warn};

use crate::pe::parser::ParsedPe;

#[derive(Debug, Error)]
pub enum LoaderError {
    #[error("mmap failed: {0}")]
    Mmap(#[from] nix::Error),

    #[error("image size is zero")]
    ZeroImageSize,

    #[error("unsupported base relocation type {0} at RVA {1:#x}")]
    UnsupportedRelocation(u8, u64),

    #[error("relocation error: {0}")]
    RelocationParse(String),

    #[error(
        "section {name} raw data range [{offset:#x}..{end:#x}] exceeds file size {file_len:#x}"
    )]
    SectionOutOfBounds {
        name: String,
        offset: u32,
        end: u32,
        file_len: usize,
    },
}

/// A PE image loaded into memory with sections mapped at the correct virtual addresses.
pub struct LoadedImage {
    /// Base address where the image is mapped.
    base: BaseAddress,
    /// Total size of the mapped region.
    size: usize,
    /// The delta between the actual load address and the PE's preferred image base.
    /// Used for applying relocations.
    relocation_delta: i64,
}

// SAFETY: The mapped memory is owned exclusively by this struct.
unsafe impl Send for LoadedImage {}
unsafe impl Sync for LoadedImage {}

impl LoadedImage {
    /// Returns the base address of the loaded image.
    pub fn base(&self) -> BaseAddress {
        self.base
    }

    /// Returns the total mapped size of the image.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Returns the address of the entry point.
    pub fn entry_point(&self, entry_rva: RelativeVirtualAddress) -> VirtualAddress {
        self.base.offset(entry_rva)
    }

    /// Returns the address of a given RVA within the loaded image.
    pub fn rva_to_va(&self, rva: RelativeVirtualAddress) -> VirtualAddress {
        self.base.offset(rva)
    }

    /// Load a parsed PE into memory: allocate virtual address space, copy sections,
    /// apply relocations, and set memory protections.
    pub fn load(parsed: &ParsedPe) -> Result<Self, LoaderError> {
        let pe = &parsed.pe;

        let image_size = pe
            .header
            .optional_header
            .map(|oh| oh.windows_fields.size_of_image as usize)
            .unwrap_or(0);

        if image_size == 0 {
            return Err(LoaderError::ZeroImageSize);
        }

        let preferred_base = BaseAddress::new(pe.image_base as usize);

        // Try to map at the preferred base address first.
        let actual_base = alloc_image(preferred_base, image_size)?;

        let relocation_delta = actual_base.delta(preferred_base);
        debug!(
            preferred_base = format_args!("{preferred_base}"),
            actual_base = format_args!("{actual_base}"),
            relocation_delta,
            image_size = format_args!("{image_size:#x}"),
            "allocated image memory"
        );

        let file_bytes = parsed.file_bytes();

        // Copy PE headers into the base of the mapped region.
        let headers_size = pe
            .header
            .optional_header
            .map(|oh| oh.windows_fields.size_of_headers as usize)
            .unwrap_or(0);
        if headers_size > 0 && headers_size <= file_bytes.len() && headers_size <= image_size {
            unsafe {
                ptr::copy_nonoverlapping(
                    file_bytes.as_ptr(),
                    actual_base.as_mut_ptr(),
                    headers_size,
                );
            }
            debug!(
                headers_size = format_args!("{headers_size:#x}"),
                "copied PE headers"
            );
        }

        // Copy each section into the mapped image.
        copy_sections(actual_base, &pe.sections, file_bytes, image_size)?;

        // Apply base relocations if we didn't load at the preferred address.
        if relocation_delta != 0 {
            apply_relocations(actual_base, pe, relocation_delta, image_size)?;
        }

        // Set correct memory protections per section.
        set_section_protections(actual_base, &pe.sections, image_size)?;

        Ok(LoadedImage {
            base: actual_base,
            size: image_size,
            relocation_delta,
        })
    }
}

impl Drop for LoadedImage {
    fn drop(&mut self) {
        if !self.base.is_null() {
            unsafe {
                let _ = mman::munmap(
                    NonNull::new(self.base.as_mut_ptr() as *mut libc::c_void).unwrap(),
                    self.size,
                );
            }
        }
    }
}

/// Allocate a contiguous region of virtual memory for the PE image.
/// Tries the preferred base first; falls back to any available address.
fn alloc_image(preferred_base: BaseAddress, size: usize) -> Result<BaseAddress, LoaderError> {
    // First, try mapping at the preferred base address.
    if !preferred_base.is_null() {
        let result = unsafe {
            mman::mmap_anonymous(
                std::num::NonZeroUsize::new(preferred_base.as_usize()),
                size.try_into().map_err(|_| LoaderError::ZeroImageSize)?,
                mman::ProtFlags::PROT_READ | mman::ProtFlags::PROT_WRITE,
                mman::MapFlags::MAP_PRIVATE | mman::MapFlags::MAP_FIXED_NOREPLACE,
            )
        };

        if let Ok(ptr) = result {
            return Ok(BaseAddress::from_ptr(ptr.as_ptr()));
        }
        debug!(
            preferred_base = format_args!("{preferred_base}"),
            "preferred base unavailable, mapping at arbitrary address"
        );
    }

    // Fallback: let the kernel choose an address.
    let ptr = unsafe {
        mman::mmap_anonymous(
            None,
            size.try_into().map_err(|_| LoaderError::ZeroImageSize)?,
            mman::ProtFlags::PROT_READ | mman::ProtFlags::PROT_WRITE,
            mman::MapFlags::MAP_PRIVATE,
        )
    }?;

    Ok(BaseAddress::from_ptr(ptr.as_ptr()))
}

/// Copy each PE section's raw data into the mapped image at its virtual address.
fn copy_sections(
    base: BaseAddress,
    sections: &[section_table::SectionTable],
    file_bytes: &[u8],
    image_size: usize,
) -> Result<(), LoaderError> {
    for section in sections {
        let name = section_name(section);
        let rva = RelativeVirtualAddress::new(section.virtual_address);
        let file_off = FileOffset::new(section.pointer_to_raw_data);
        let raw_size = section.size_of_raw_data as usize;
        let virtual_size = section.virtual_size as usize;

        // Determine how many bytes to actually copy (min of raw size and virtual size).
        let copy_size = raw_size.min(virtual_size);

        if raw_size > 0 {
            let raw_end = file_off.as_usize() + raw_size;
            if raw_end > file_bytes.len() {
                return Err(LoaderError::SectionOutOfBounds {
                    name,
                    offset: file_off.as_u32(),
                    end: raw_end as u32,
                    file_len: file_bytes.len(),
                });
            }

            if rva.as_usize() + copy_size > image_size {
                warn!(
                    name,
                    rva = format_args!("{rva}"),
                    copy_size = format_args!("{copy_size:#x}"),
                    "section exceeds image size, skipping"
                );
                continue;
            }

            unsafe {
                let dest = base.offset(rva);
                ptr::copy_nonoverlapping(
                    file_bytes.as_ptr().add(file_off.as_usize()),
                    dest.as_mut_ptr(),
                    copy_size,
                );
            }
        }

        // If virtual_size > raw_size, the remainder is already zero (mmap gives zeroed pages).

        debug!(
            name,
            rva = format_args!("{rva}"),
            raw_size = format_args!("{raw_size:#x}"),
            virtual_size = format_args!("{virtual_size:#x}"),
            "mapped section"
        );
    }

    Ok(())
}

/// Apply base relocations when the image was loaded at a different address
/// than its preferred image base.
fn apply_relocations(
    base: BaseAddress,
    pe: &goblin::pe::PE,
    delta: i64,
    image_size: usize,
) -> Result<(), LoaderError> {
    let reloc_data = match &pe.relocation_data {
        Some(data) => data,
        None => {
            warn!("image needs relocation but has no relocation data");
            return Ok(());
        }
    };

    let mut count: usize = 0;

    for block_result in reloc_data.blocks() {
        let block = block_result.map_err(|e| LoaderError::RelocationParse(e.to_string()))?;
        let block_rva = RelativeVirtualAddress::new(block.rva);

        for word_result in block.words() {
            let word = word_result.map_err(|e| LoaderError::RelocationParse(e.to_string()))?;
            let reloc_type = word.reloc_type();
            let offset = word.offset() as u32;
            let rva = block_rva.add(offset);

            match reloc_type as u16 {
                relocation::IMAGE_REL_BASED_ABSOLUTE => {
                    // Padding entry, skip.
                }
                relocation::IMAGE_REL_BASED_DIR64 => {
                    if rva.as_usize() + 8 > image_size {
                        warn!(
                            rva = format_args!("{rva}"),
                            "DIR64 relocation out of bounds, skipping"
                        );
                        continue;
                    }
                    unsafe {
                        let ptr = base.offset(rva).as_mut_ptr() as *mut u64;
                        let value = ptr::read_unaligned(ptr);
                        ptr::write_unaligned(ptr, (value as i64 + delta) as u64);
                    }
                    count += 1;
                }
                relocation::IMAGE_REL_BASED_HIGHLOW => {
                    if rva.as_usize() + 4 > image_size {
                        warn!(
                            rva = format_args!("{rva}"),
                            "HIGHLOW relocation out of bounds, skipping"
                        );
                        continue;
                    }
                    unsafe {
                        let ptr = base.offset(rva).as_mut_ptr() as *mut u32;
                        let value = ptr::read_unaligned(ptr);
                        ptr::write_unaligned(ptr, (value as i64 + delta) as u32);
                    }
                    count += 1;
                }
                relocation::IMAGE_REL_BASED_HIGH => {
                    if rva.as_usize() + 2 > image_size {
                        continue;
                    }
                    unsafe {
                        let ptr = base.offset(rva).as_mut_ptr() as *mut u16;
                        let value = ptr::read_unaligned(ptr) as i32;
                        let adjusted = value + (delta >> 16) as i32;
                        ptr::write_unaligned(ptr, adjusted as u16);
                    }
                    count += 1;
                }
                relocation::IMAGE_REL_BASED_LOW => {
                    if rva.as_usize() + 2 > image_size {
                        continue;
                    }
                    unsafe {
                        let ptr = base.offset(rva).as_mut_ptr() as *mut u16;
                        let value = ptr::read_unaligned(ptr) as i32;
                        let adjusted = value + (delta & 0xFFFF) as i32;
                        ptr::write_unaligned(ptr, adjusted as u16);
                    }
                    count += 1;
                }
                _ => {
                    return Err(LoaderError::UnsupportedRelocation(
                        reloc_type,
                        rva.as_usize() as u64,
                    ));
                }
            }
        }
    }

    debug!(count, delta, "applied base relocations");
    Ok(())
}

/// Translate PE section characteristics to mmap protection flags.
fn section_characteristics_to_prot(characteristics: u32) -> mman::ProtFlags {
    let mut prot = mman::ProtFlags::empty();

    if characteristics & section_table::IMAGE_SCN_MEM_READ != 0 {
        prot |= mman::ProtFlags::PROT_READ;
    }
    if characteristics & section_table::IMAGE_SCN_MEM_WRITE != 0 {
        prot |= mman::ProtFlags::PROT_WRITE;
    }
    if characteristics & section_table::IMAGE_SCN_MEM_EXECUTE != 0 {
        prot |= mman::ProtFlags::PROT_EXEC;
    }

    // Sections must be at least readable.
    if prot.is_empty() {
        prot = mman::ProtFlags::PROT_READ;
    }

    prot
}

/// Set the final memory protections on each section after relocations are applied.
fn set_section_protections(
    base: BaseAddress,
    sections: &[section_table::SectionTable],
    image_size: usize,
) -> Result<(), LoaderError> {
    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize };

    for section in sections {
        let rva = RelativeVirtualAddress::new(section.virtual_address);
        let virtual_size = section.virtual_size as usize;

        if virtual_size == 0 || rva.as_usize() + virtual_size > image_size {
            continue;
        }

        // Align to page boundaries for mprotect.
        let section_start = base.offset(rva);
        let aligned_start = section_start.align_down(page_size);
        let aligned_end = section_start.add(virtual_size).align_up(page_size);
        let aligned_size = aligned_end.as_usize() - aligned_start.as_usize();

        let prot = section_characteristics_to_prot(section.characteristics);

        unsafe {
            mman::mprotect(
                NonNull::new(aligned_start.as_mut_ptr() as *mut libc::c_void).unwrap(),
                aligned_size,
                prot,
            )?;
        }

        debug!(
            name = section_name(section),
            rva = format_args!("{rva}"),
            size = format_args!("{aligned_size:#x}"),
            prot = format_args!("{prot:?}"),
            "set section protection"
        );
    }

    Ok(())
}

/// Extract a section name as a String from its raw bytes.
fn section_name(section: &section_table::SectionTable) -> String {
    if let Some(ref real_name) = section.real_name {
        return real_name.clone();
    }
    let name_bytes = &section.name;
    let len = name_bytes
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(name_bytes.len());
    String::from_utf8_lossy(&name_bytes[..len]).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use goblin::pe::section_table;

    #[test]
    fn prot_flags_read_only() {
        let prot = section_characteristics_to_prot(section_table::IMAGE_SCN_MEM_READ);
        assert!(prot.contains(mman::ProtFlags::PROT_READ));
        assert!(!prot.contains(mman::ProtFlags::PROT_WRITE));
        assert!(!prot.contains(mman::ProtFlags::PROT_EXEC));
    }

    #[test]
    fn prot_flags_read_execute() {
        let chars = section_table::IMAGE_SCN_MEM_READ | section_table::IMAGE_SCN_MEM_EXECUTE;
        let prot = section_characteristics_to_prot(chars);
        assert!(prot.contains(mman::ProtFlags::PROT_READ));
        assert!(prot.contains(mman::ProtFlags::PROT_EXEC));
        assert!(!prot.contains(mman::ProtFlags::PROT_WRITE));
    }

    #[test]
    fn prot_flags_read_write() {
        let chars = section_table::IMAGE_SCN_MEM_READ | section_table::IMAGE_SCN_MEM_WRITE;
        let prot = section_characteristics_to_prot(chars);
        assert!(prot.contains(mman::ProtFlags::PROT_READ));
        assert!(prot.contains(mman::ProtFlags::PROT_WRITE));
    }

    #[test]
    fn prot_flags_empty_defaults_to_read() {
        let prot = section_characteristics_to_prot(0);
        assert!(prot.contains(mman::ProtFlags::PROT_READ));
    }
}
