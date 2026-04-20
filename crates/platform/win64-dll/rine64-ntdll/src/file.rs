//! ntdll file I/O: NtCreateFile, NtReadFile, NtWriteFile, NtClose,
//! NtQueryInformationFile.

use rine_common_ntdll::file as common;
use rine_types::errors::NtStatus;
use rine_types::handles::{Handle, handle_to_fd};
use rine_types::os::IoStatusBlock;

/// Read data from a file identified by a HANDLE.
///
/// # Arguments
/// * `_file_handle`: the file handle to read from. (ignored)
/// * `_event`: optional event to signal when the read completes (ignored).
/// * `_apc_routine`: optional APC routine to call when the read completes (ignored).
/// * `_apc_context`: optional context for the APC routine (ignored).
/// * `_io_status_block`: pointer to an `IoStatusBlock` structure (ignored).
/// * `_buffer`: pointer to the buffer to receive the data (ignored).
/// * `_length`: number of bytes to read (ignored).
/// * `_byte_offset`: pointer to the byte offset to start reading from (ignored).
/// * `_key`: optional key for the I/O operation (ignored).
///
/// # Safety
/// All pointer parameters must be valid.
///
/// # Returns
/// The number of bytes read (always 0 in this stub implementation).
///
/// # Note
/// This is a stub implementation that does not perform any actual I/O.
/// It simply logs a warning and returns 0 bytes read.
#[allow(non_snake_case, clippy::too_many_arguments)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn NtReadFile(
    file_handle: isize,
    _event: usize,
    _apc_routine: usize,
    _apc_context: usize,
    io_status_block: *mut IoStatusBlock,
    buffer: *mut u8,
    length: u32,
    _byte_offset: *const i64,
    _key: *const u32,
) -> u32 {
    unsafe {
        let handle = Handle::from_raw(file_handle);

        common::nt_read_file(
            handle,
            _event,
            _apc_routine,
            _apc_context,
            io_status_block,
            buffer,
            length,
            _byte_offset,
            _key,
        )
    }
}

/// Write data to a file/pipe/device identified by a HANDLE.
///
/// # Arguments
/// * `file_handle`: the file handle to write to.
/// * `_event`: optional event to signal when the write completes (ignored).
/// * `_apc_routine`: optional APC routine to call when the write completes (ignored).
/// * `_apc_context`: optional context for the APC routine (ignored).
/// * `io_status_block`: pointer to an `IoStatusBlock` structure.
/// * `buffer`: pointer to the buffer to write.
/// * `length`: number of bytes to write.
/// * `_byte_offset`: pointer to the byte offset to start writing from (ignored).
/// * `_key`: optional key for the I/O operation (ignored).
///
/// # Safety
/// All pointer parameters must be valid.
/// `buffer` must point to at least `length` readable bytes.
///
/// # Returns
/// STATUS_SUCCESS (0) on success, or an appropriate NTSTATUS error code on failure.
#[allow(non_snake_case, clippy::too_many_arguments)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn NtWriteFile(
    file_handle: isize,
    _event: isize,
    _apc_routine: usize,
    _apc_context: usize,
    io_status_block: *mut IoStatusBlock,
    buffer: *const u8,
    length: u32,
    _byte_offset: *const i64,
    _key: *const u32,
) -> u32 {
    let handle = Handle::from_raw(file_handle);
    unsafe {
        common::nt_write_file(
            handle,
            _event,
            _apc_routine,
            _apc_context,
            io_status_block,
            buffer,
            length,
            _byte_offset,
            _key,
        )
    }
}

/// Open or create a file via the NT native API.
///
/// # Arguments
/// * `file_handle`: pointer to receive the file handle (out).
/// * `desired_access`: the desired access rights (ACCESS_MASK).
/// * `object_attributes`: pointer to an OBJECT_ATTRIBUTES structure (opaque for now).
/// * `io_status_block`: pointer to an IoStatusBlock structure.
/// * `_allocation_size`: optional pointer to the allocation size (ignored).
/// * `_file_attributes`: file attributes (ignored).
/// * `_share_access`: share access flags (ignored).
/// * `create_disposition`: the action to take if the file exists or does not exist (NT disposition, not the same as Win32).
/// * `_create_options`: creation options (ignored).
/// * `_ea_buffer`: optional pointer to the extended attributes buffer (ignored).
/// * `_ea_length`: length of the extended attributes buffer (ignored).
///
/// # Safety
/// All pointer parameters must be valid.
///
/// # Returns
/// STATUS_SUCCESS (0) on success, or an appropriate NTSTATUS error code on failure.
///
/// # Notes
/// This is a simplified implementation: it extracts the path from
/// `OBJECT_ATTRIBUTES`, translates it, and calls `open(2)`.
/// Many NT-specific features (EaBuffer, AllocationSize, etc.) are ignored.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn NtCreateFile(
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
        common::nt_create_file(
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

/// Close an NT handle.
///
/// # Arguments
/// * `object_handle`: the handle to close.
///
/// # Safety
/// `object_handle` must be a valid handle returned by a previous call to NtCreateFile or similar functions.
/// Closing an invalid handle may lead to undefined behavior.
///
/// # Returns
/// STATUS_SUCCESS (0) on success, or an appropriate NTSTATUS error code on failure.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn NtClose(object_handle: isize) -> u32 {
    let handle = Handle::from_raw(object_handle);
    unsafe { common::nt_close(handle) }
}

// ---------------------------------------------------------------------------
// NtQueryInformationFile
// ---------------------------------------------------------------------------

/// File information classes used by NtQueryInformationFile.
#[allow(dead_code)]
const FILE_STANDARD_INFORMATION: u32 = 5;
#[allow(dead_code)]
const FILE_POSITION_INFORMATION: u32 = 14;

/// NtQueryInformationFile — query metadata about an open file.
///
/// Currently supports:
///  - `FileStandardInformation` (class 5): returns file size, link count, etc.
///  - other classes: returns NOT_IMPLEMENTED
#[allow(non_snake_case, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn NtQueryInformationFile(
    file_handle: isize,
    io_status_block: *mut IoStatusBlock,
    file_information: *mut u8,
    _length: u32,
    file_information_class: u32,
) -> u32 {
    let handle = Handle::from_raw(file_handle);
    let Some(fd) = handle_to_fd(handle) else {
        return NtStatus::INVALID_HANDLE.0;
    };

    match file_information_class {
        FILE_STANDARD_INFORMATION => {
            // FILE_STANDARD_INFORMATION layout (x64):
            //   LARGE_INTEGER AllocationSize  (offset 0, 8 bytes)
            //   LARGE_INTEGER EndOfFile       (offset 8, 8 bytes)
            //   ULONG         NumberOfLinks   (offset 16, 4 bytes)
            //   BOOLEAN       DeletePending   (offset 20, 1 byte)
            //   BOOLEAN       Directory       (offset 21, 1 byte)
            let mut stat: libc::stat = unsafe { core::mem::zeroed() };
            if unsafe { libc::fstat(fd, &mut stat) } != 0 {
                return NtStatus::INVALID_PARAMETER.0;
            }

            let info = file_information;
            let size = stat.st_size as u64;
            let alloc_size = stat.st_blocks as u64 * 512;
            let is_dir: u8 = if (stat.st_mode & libc::S_IFDIR) != 0 {
                1
            } else {
                0
            };

            unsafe {
                core::ptr::write_unaligned(info as *mut u64, alloc_size);
                core::ptr::write_unaligned(info.add(8) as *mut u64, size);
                core::ptr::write_unaligned(info.add(16) as *mut u32, stat.st_nlink as u32);
                *info.add(20) = 0; // DeletePending = false
                *info.add(21) = is_dir;
            }

            if !io_status_block.is_null() {
                unsafe {
                    (*io_status_block).status = NtStatus::SUCCESS.0;
                    (*io_status_block).information = 24; // bytes written
                }
            }
            NtStatus::SUCCESS.0
        }
        _ => {
            tracing::warn!(
                class = file_information_class,
                "NtQueryInformationFile: unsupported information class"
            );
            NtStatus::NOT_IMPLEMENTED.0
        }
    }
}
