//! ntdll file I/O: NtCreateFile, NtReadFile, NtWriteFile, NtClose,
//! NtQueryInformationFile.

use rine_common_ntdll as common;
use rine_types::os::IoStatusBlock;

// ---------------------------------------------------------------------------
// NtCreateFile
// ---------------------------------------------------------------------------

/// NtCreateFile — open or create a file via the NT native API.
///
/// This is a simplified implementation: it extracts the path from
/// `OBJECT_ATTRIBUTES`, translates it, and calls `open(2)`.
/// Many NT-specific features (EaBuffer, AllocationSize, etc.) are ignored.
///
/// # Safety
/// All pointer parameters must be valid.
#[allow(non_snake_case)]
pub unsafe extern "stdcall" fn NtCreateFile(
    file_handle: *mut isize,  // PHANDLE (out)
    desired_access: u32,      // ACCESS_MASK
    object_attributes: usize, // POBJECT_ATTRIBUTES (opaque for now)
    io_status_block: *mut IoStatusBlock,
    _allocation_size: *const i64,
    _file_attributes: u32,
    _share_access: u32,
    create_disposition: u32, // NT disposition (not the same as Win32)
    _create_options: u32,
    _ea_buffer: usize,
    _ea_length: u32,
) -> u32 {
    unsafe {
        common::file::nt_create_file(
            file_handle,
            desired_access,
            object_attributes,
            io_status_block,
            _allocation_size,
            _file_attributes,
            _share_access,
            create_disposition,
            _create_options,
            _ea_buffer,
            _ea_length,
        )
    }
}
