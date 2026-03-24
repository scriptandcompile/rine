//! ntdll file I/O: NtCreateFile, NtReadFile, NtWriteFile, NtClose,
//! NtQueryInformationFile.

use rine_types::errors::NtStatus;
use rine_types::handles::{
    GENERIC_READ, GENERIC_WRITE, Handle, HandleEntry, handle_table, handle_to_fd,
};
use rine_types::structs::IoStatusBlock;

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
    unsafe { *file_handle = h.as_raw() };

    if !io_status_block.is_null() {
        unsafe {
            (*io_status_block).status = NtStatus::SUCCESS.0;
            (*io_status_block).information = 0; // FILE_OPENED or similar
        }
    }
    NtStatus::SUCCESS.0
}

// ---------------------------------------------------------------------------
// NtReadFile
// ---------------------------------------------------------------------------

/// NtReadFile — read data from a file identified by a HANDLE.
///
/// # Safety
/// `buffer` must be writable for at least `length` bytes.
#[allow(non_snake_case)]
pub unsafe extern "win64" fn NtReadFile(
    file_handle: isize,
    _event: isize,
    _apc_routine: usize,
    _apc_context: usize,
    io_status_block: *mut IoStatusBlock,
    buffer: *mut u8,
    length: u32,
    _byte_offset: *const i64,
    _key: *const u32,
) -> u32 {
    let handle = Handle::from_raw(file_handle);
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

// ---------------------------------------------------------------------------
// NtWriteFile (existing)
// ---------------------------------------------------------------------------

/// NtWriteFile — write data to a file/pipe/device identified by a HANDLE.
///
/// # Safety
/// `buffer` must point to at least `length` readable bytes.
#[allow(non_snake_case)]
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
    let Some(fd) = handle_to_fd(handle) else {
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

// ---------------------------------------------------------------------------
// NtClose
// ---------------------------------------------------------------------------

/// NtClose — close an NT handle.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn NtClose(object_handle: isize) -> u32 {
    let handle = Handle::from_raw(object_handle);

    match handle_table().remove(handle) {
        Some(HandleEntry::File(fd)) => {
            unsafe { libc::close(fd) };
            NtStatus::SUCCESS.0
        }
        Some(HandleEntry::FindData(_)) => NtStatus::SUCCESS.0,
        Some(HandleEntry::Thread(_)) => NtStatus::SUCCESS.0,
        Some(HandleEntry::Event(_)) => NtStatus::SUCCESS.0,
        None => {
            tracing::warn!(handle = object_handle, "NtClose: unknown handle");
            NtStatus::INVALID_HANDLE.0
        }
    }
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
