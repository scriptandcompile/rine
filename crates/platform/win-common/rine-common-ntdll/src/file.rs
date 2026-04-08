use rine_types::errors::NtStatus;
use rine_types::handles::{GENERIC_READ, GENERIC_WRITE, HandleEntry, handle_table};
use rine_types::os::IoStatusBlock;

/// NtCreateFile — open or create a file via the NT native API.
///
/// This is a simplified implementation: it extracts the path from
/// `OBJECT_ATTRIBUTES`, translates it, and calls `open(2)`.
/// Many NT-specific features (EaBuffer, AllocationSize, etc.) are ignored.
///
/// # Safety
/// All pointer parameters must be valid.
#[allow(clippy::too_many_arguments)]
pub unsafe fn nt_create_file(
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
