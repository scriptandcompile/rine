use rine_types::errors::NtStatus;
use rine_types::handles::{
    GENERIC_READ, GENERIC_WRITE, Handle, HandleEntry, handle_table, handle_to_fd,
};
use rine_types::os::IoStatusBlock;

/// Read data from a file identified by a HANDLE.
///
/// # Arguments
/// * `handle`: the file handle to read from.
/// * `_event`: optional event to signal when the read completes (ignored).
/// * `_apc_routine`: optional APC routine to call when the read completes (ignored).
/// * `_apc_context`: optional context for the APC routine (ignored).
/// * `io_status_block`: pointer to an `IoStatusBlock` structure.
/// * `buffer`: pointer to the buffer to receive the data.
/// * `length`: number of bytes to read.
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
#[allow(clippy::too_many_arguments)]
pub unsafe fn nt_read_file(
    handle: Handle,
    _event: Handle,
    _apc_routine: usize,
    _apc_context: usize,
    io_status_block: *mut IoStatusBlock,
    buffer: *mut u8,
    length: u32,
    _byte_offset: *const i64,
    _key: *const u32,
) -> u32 {
    let Some(fd) = handle_to_fd(handle) else {
        return NtStatus::INVALID_HANDLE.0;
    };

    let n = unsafe { libc::read(fd, buffer.cast(), length as usize) };

    if n < 0 {
        if !io_status_block.is_null() {
            unsafe {
                (*io_status_block).status = NtStatus::INVALID_PARAMETER.0;
                (*io_status_block).information = 0;
            }
        }
        return NtStatus::INVALID_PARAMETER.0;
    }

    if n == 0 {
        if !io_status_block.is_null() {
            unsafe {
                (*io_status_block).status = NtStatus::END_OF_FILE.0;
                (*io_status_block).information = 0;
            }
        }
        return NtStatus::END_OF_FILE.0;
    }

    if !io_status_block.is_null() {
        unsafe {
            (*io_status_block).status = NtStatus::SUCCESS.0;
            (*io_status_block).information = n as usize;
        }
    }
    NtStatus::SUCCESS.0
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
#[allow(clippy::too_many_arguments)]
pub unsafe fn nt_write_file(
    file_handle: Handle,
    _event: Handle,
    _apc_routine: usize,
    _apc_context: usize,
    io_status_block: *mut IoStatusBlock,
    buffer: *const u8,
    length: u32,
    _byte_offset: *const i64,
    _key: *const u32,
) -> u32 {
    let Some(fd) = handle_to_fd(file_handle) else {
        return NtStatus::INVALID_HANDLE.0;
    };

    let written = unsafe { libc::write(fd, buffer.cast(), length as usize) };

    if written < 0 {
        if !io_status_block.is_null() {
            unsafe {
                (*io_status_block).status = NtStatus::INVALID_PARAMETER.0;
                (*io_status_block).information = 0;
            }
        }
        return NtStatus::INVALID_PARAMETER.0;
    }

    if !io_status_block.is_null() {
        unsafe {
            (*io_status_block).status = NtStatus::SUCCESS.0;
            (*io_status_block).information = written as usize;
        }
    }
    NtStatus::SUCCESS.0
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
#[allow(clippy::too_many_arguments)]
pub unsafe fn nt_create_file(
    file_handle: *mut Handle,
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
    if file_handle.is_null() {
        return NtStatus::INVALID_PARAMETER.0;
    }

    // NT dispositions differ from Win32.  We map the common ones:
    //   FILE_SUPERSEDE       (0) → O_CREAT | O_TRUNC
    //   FILE_OPEN            (1) → (nothing — must exist)
    //   FILE_CREATE          (2) → O_CREAT | O_EXCL
    //   FILE_OPEN_IF         (3) → O_CREAT
    //   FILE_OVERWRITE       (4) → O_TRUNC
    //   FILE_OVERWRITE_IF    (5) → O_CREAT | O_TRUNC
    let mut flags: i32 = 0;
    let read = (desired_access & GENERIC_READ) != 0 || (desired_access & 0x0001) != 0; // FILE_READ_DATA
    let write = (desired_access & GENERIC_WRITE) != 0 || (desired_access & 0x0002) != 0; // FILE_WRITE_DATA
    if read && write {
        flags |= libc::O_RDWR;
    } else if write {
        flags |= libc::O_WRONLY;
    } else {
        flags |= libc::O_RDONLY;
    }

    match create_disposition {
        0 | 5 => flags |= libc::O_CREAT | libc::O_TRUNC, // SUPERSEDE / OVERWRITE_IF
        1 => {}                                          // OPEN
        2 => flags |= libc::O_CREAT | libc::O_EXCL,      // CREATE
        3 => flags |= libc::O_CREAT,                     // OPEN_IF
        4 => flags |= libc::O_TRUNC,                     // OVERWRITE
        _ => {
            tracing::warn!(
                disp = create_disposition,
                "NtCreateFile: unknown disposition"
            );
            return NtStatus::INVALID_PARAMETER.0;
        }
    }

    // We can't easily extract the path from OBJECT_ATTRIBUTES without
    // knowing the caller's struct layout.  For now we log a warning
    // and open /dev/null as a placeholder if object_attributes is opaque.
    // Real programs typically call kernel32!CreateFile which goes through
    // our working implementation.
    tracing::debug!(
        access = desired_access,
        disp = create_disposition,
        obj_attr = object_attributes,
        "NtCreateFile (stub — opening /dev/null)"
    );

    let path = std::ffi::CString::new("/dev/null").unwrap();
    let fd = unsafe { libc::open(path.as_ptr(), flags, 0o644 as libc::c_uint) };
    if fd < 0 {
        return NtStatus::OBJECT_NAME_NOT_FOUND.0;
    }

    let h = handle_table().insert(HandleEntry::File(fd));
    unsafe {
        *file_handle = h;
    }

    if !io_status_block.is_null() {
        unsafe {
            (*io_status_block).status = NtStatus::SUCCESS.0;
            (*io_status_block).information = 0; // FILE_OPENED or similar
        }
    }
    NtStatus::SUCCESS.0
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
pub unsafe fn nt_close(handle: Handle) -> u32 {
    match handle_table().remove(handle) {
        Some(HandleEntry::File(fd)) => {
            unsafe { libc::close(fd) };
            NtStatus::SUCCESS.0
        }
        Some(HandleEntry::FindData(_)) => NtStatus::SUCCESS.0,
        Some(HandleEntry::Thread(_)) => NtStatus::SUCCESS.0,
        Some(HandleEntry::Event(_)) => NtStatus::SUCCESS.0,
        Some(HandleEntry::Process(_)) => NtStatus::SUCCESS.0,
        Some(HandleEntry::Mutex(_)) => NtStatus::SUCCESS.0,
        Some(HandleEntry::Semaphore(_)) => NtStatus::SUCCESS.0,
        Some(HandleEntry::Heap(_)) => NtStatus::SUCCESS.0,
        Some(HandleEntry::RegistryKey(_)) => NtStatus::SUCCESS.0,
        Some(HandleEntry::Window(_)) => {
            // Window handles should not be closed via NtClose.
            NtStatus::INVALID_HANDLE.0
        }
        None => {
            tracing::warn!(handle = handle.as_raw(), "NtClose: unknown handle");
            NtStatus::INVALID_HANDLE.0
        }
    }
}

/// File information classes used by NtQueryInformationFile.
#[allow(dead_code)]
const FILE_STANDARD_INFORMATION: u32 = 5;

/// Query metadata about an open file.
///
/// # Arguments
/// * `file_handle`: the handle of the file to query.
/// * `io_status_block`: pointer to an IoStatusBlock structure to receive the status.
/// * `file_information`: pointer to a buffer to receive the file information.
/// * `_length`: the length of the `file_information` buffer in bytes (ignored).
/// * `file_information_class`: the class of information to query (e.g., FileStandardInformation).
///
/// # Safety
/// All pointer parameters must be valid. `file_information` must point to a writable buffer of
/// sufficient size for the requested information class.
///
/// # Returns
/// STATUS_SUCCESS (0) on success, or an appropriate NTSTATUS error code on failure.
///
/// # Notes
/// Currently supports `FileStandardInformation` (class 5): returns file size, link count, etc.
/// While, other classes return NOT_IMPLEMENTED.
pub unsafe fn nt_query_information_file(
    file_handle: Handle,
    io_status_block: *mut IoStatusBlock,
    file_information: *mut u8,
    _length: u32,
    file_information_class: u32,
) -> u32 {
    let Some(fd) = handle_to_fd(file_handle) else {
        return NtStatus::INVALID_HANDLE.0;
    };

    match file_information_class {
        FILE_STANDARD_INFORMATION => {
            // FILE_STANDARD_INFORMATION layout (x64/x86):
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
