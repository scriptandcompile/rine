use rine_types::{
    errors::WinBool,
    handles::{
        CREATE_ALWAYS, CREATE_NEW, FILE_BEGIN, FILE_CURRENT, FILE_END, GENERIC_READ, GENERIC_WRITE,
        Handle, HandleEntry, INVALID_HANDLE_VALUE, INVALID_SET_FILE_POINTER, OPEN_ALWAYS,
        OPEN_EXISTING, TRUNCATE_EXISTING, handle_table, handle_to_fd, std_handle_to_fd,
    },
};

/// implementation of win32 WriteFile, shared between 32-bit and 64-bit DLLs.
///
/// # Arguments
/// * `handle`: Windows file handle (must have been created by CreateFile).
/// * `buffer`: pointer to data to write.
/// * `bytes_to_write`: number of bytes to write.
/// * `bytes_written`: output pointer for number of bytes actually written (can be null).
/// * `_overlapped`: ignored.
///
/// # Safety
/// * `handle` must be a valid file handle returned by CreateFile.
/// * `buffer` must point to at least `bytes_to_write` bytes of valid memory.
pub unsafe fn write_file(
    handle: Handle,
    buffer: *const u8,
    bytes_to_write: u32,
    bytes_written: *mut u32,
    _overlapped: *mut core::ffi::c_void,
) -> WinBool {
    let Some(fd) = handle_to_fd(handle) else {
        return WinBool::FALSE;
    };

    let written = unsafe { libc::write(fd, buffer.cast(), bytes_to_write as usize) };
    if written < 0 {
        return WinBool::FALSE;
    }

    if !bytes_written.is_null() {
        unsafe { *bytes_written = written as u32 };
    }
    WinBool::TRUE
}

/// Implementation of shared SetFilePointer logic for 32-bit and 64-bit DLLs.
///
/// # Arguments
/// * `handle`: Windows file handle (must have been created by CreateFile).
/// * `distance_to_move` - The low 32 bits of the distance to move, in bytes. Can be negative to move backwards.
/// * `distance_to_move_high` - Optional pointer to the high 32 bits of the distance to move.
///   If non-null, this is an input/output parameter that should be initialized to the high bits of the distance
///   before the call, and will be updated to the high bits of the new file pointer after the call.
/// * `move_method` - The starting point for the move. Must be one of `FILE_BEGIN`, `FILE_CURRENT`, or `FILE_END`.
///
/// # Safety
/// * `file` must be a valid file handle returned by `CreateFile`.
/// * `distance_to_move_high` must be null or point to a valid i32 variable if `distance_to_move` is negative
///   or the distance exceeds 2GB.
pub unsafe fn set_file_pointer(
    handle: Handle,
    distance_to_move: i32,           // low 32 bits
    distance_to_move_high: *mut i32, // high 32 bits (in/out, optional)
    move_method: u32,
) -> u32 {
    let Some(fd) = handle_to_fd(handle) else {
        return INVALID_SET_FILE_POINTER;
    };

    let offset: i64 = if !distance_to_move_high.is_null() {
        let high = unsafe { *distance_to_move_high } as i64;
        (high << 32) | (distance_to_move as u32 as i64)
    } else {
        distance_to_move as i64
    };

    let whence = match move_method {
        FILE_BEGIN => libc::SEEK_SET,
        FILE_CURRENT => libc::SEEK_CUR,
        FILE_END => libc::SEEK_END,
        _ => return INVALID_SET_FILE_POINTER,
    };

    // Use the 64-bit seek entrypoint so 32-bit builds can still address >2/4GB offsets.
    let result = unsafe { libc::lseek64(fd, offset as libc::off64_t, whence) };
    if result == -1 {
        return INVALID_SET_FILE_POINTER;
    }

    if !distance_to_move_high.is_null() {
        unsafe { *distance_to_move_high = ((result as i64) >> 32) as i32 };
    }
    result as u32
}

/// Shared implementation for CreateFileA/W.
///
/// # Arguments
/// * `win_path`: Windows-style file path (e.g. `C:\foo\bar.txt`).
/// * `desired_access`: Windows access mask (e.g. `GENERIC_READ | GENERIC_WRITE`).
/// * `creation_disposition`: Windows creation disposition (e.g. `CREATE_ALWAYS`).
///
/// # Returns
/// On success, returns a valid Windows file handle (which must be closed with `CloseHandle` when no longer needed).
/// On failure, returns `INVALID_HANDLE_VALUE`.
pub fn create_file(win_path: &str, desired_access: u32, creation_disposition: u32) -> isize {
    tracing::debug!(
        path = win_path,
        access = desired_access,
        disp = creation_disposition,
        "CreateFile"
    );

    // Build Linux open flags from Windows parameters.
    let mut flags: i32 = 0;

    let read = (desired_access & GENERIC_READ) != 0;
    let write = (desired_access & GENERIC_WRITE) != 0;
    if read && write {
        flags |= libc::O_RDWR;
    } else if write {
        flags |= libc::O_WRONLY;
    } else {
        flags |= libc::O_RDONLY;
    }

    match creation_disposition {
        CREATE_NEW => flags |= libc::O_CREAT | libc::O_EXCL,
        CREATE_ALWAYS => flags |= libc::O_CREAT | libc::O_TRUNC,
        OPEN_EXISTING => {} // no extra flags
        OPEN_ALWAYS => flags |= libc::O_CREAT,
        TRUNCATE_EXISTING => flags |= libc::O_TRUNC,
        _ => {
            tracing::warn!(
                disp = creation_disposition,
                "CreateFile: unknown creation disposition"
            );
            return INVALID_HANDLE_VALUE.as_raw();
        }
    }

    // Translate Windows path → Linux path.
    let linux_path = translate_win_path(win_path);

    let c_path = match std::ffi::CString::new(linux_path.to_string_lossy().as_bytes()) {
        Ok(s) => s,
        Err(_) => return INVALID_HANDLE_VALUE.as_raw(),
    };

    let mode: libc::mode_t = 0o644;
    let fd = unsafe { libc::open(c_path.as_ptr(), flags, mode as libc::c_uint) };
    if fd < 0 {
        tracing::debug!(path = %linux_path.display(), errno = std::io::Error::last_os_error().raw_os_error(), "CreateFile: open failed");
        return INVALID_HANDLE_VALUE.as_raw();
    }

    let h = handle_table().insert(HandleEntry::File(fd));
    tracing::debug!(handle = ?h, fd, path = %linux_path.display(), "CreateFile: opened");
    rine_types::dev_notify!(on_handle_created(
        h.as_raw() as i64,
        "File",
        &linux_path.display().to_string()
    ));
    h.as_raw()
}

/// CloseHandle — close an open object handle (e.g. file handle).
///
/// # Arguments
/// * `handle` - The handle to close. Must be a valid handle returned by `CreateFile` or other handle-returning function.
///
/// # Safety
/// `handle` must be a valid handle returned by `CreateFile` or other handle-returning function.
/// After this call, `handle` must not be used again.
///
/// # Returns
/// `WinBool::TRUE` on success, `WinBool::FALSE` on failure.
///
/// # Note
/// This implementation only supports closing of file handles.
/// Currently it does not support closing of other handle types tracked in the handle table
/// (threads, events, processes, mutexes, semaphores, heaps, registry keys, and FindFirstFile find data).
/// It does not support closing of window handles, which are not tracked in the handle table.
#[allow(non_snake_case)]
pub fn close_handle(handle: Handle) -> WinBool {
    match handle_table().remove(handle) {
        Some(HandleEntry::Thread(_)) => WinBool::TRUE,
        Some(HandleEntry::Event(_)) => WinBool::TRUE,
        Some(HandleEntry::Process(_)) => WinBool::TRUE,
        Some(HandleEntry::Mutex(_)) => WinBool::TRUE,
        Some(HandleEntry::Semaphore(_)) => WinBool::TRUE,
        Some(HandleEntry::Heap(_)) => WinBool::TRUE,
        Some(HandleEntry::RegistryKey(_)) => WinBool::TRUE,
        Some(HandleEntry::FindData(_)) => WinBool::TRUE,
        Some(HandleEntry::File(object)) => {
            let Some(fd) = std_handle_to_fd(object as u32) else {
                return WinBool::FALSE;
            };

            // 'fd' should be the linux file descriptor.
            // If it's a standard stream, we don't actually want to close it.
            if fd == libc::STDERR_FILENO || fd == libc::STDOUT_FILENO || fd == libc::STDIN_FILENO {
                WinBool::TRUE
            } else {
                unsafe { libc::close(fd) };
                WinBool::TRUE
            }
        }
        Some(HandleEntry::Window(_)) => WinBool::FALSE,
        None => WinBool::FALSE,
    }
}
/// Delete a file at the given Windows path.
///
/// # Arguments
/// * `win_path`: Windows-style file path (e.g. `C:\foo\bar.txt`).
///
/// # Returns
/// `WinBool::TRUE` if the file was successfully deleted, `WinBool::FALSE` if an error occurred (e.g. file not found).
pub fn delete_file(win_path: &str) -> WinBool {
    tracing::debug!(path = win_path, "DeleteFile");

    let linux_path = translate_win_path(win_path);
    let c_path = match std::ffi::CString::new(linux_path.to_string_lossy().as_bytes()) {
        Ok(s) => s,
        Err(_) => return WinBool::FALSE,
    };

    match unsafe { libc::unlink(c_path.as_ptr()) } {
        0 => WinBool::TRUE,
        _ => {
            tracing::debug!(path = %linux_path.display(), errno = std::io::Error::last_os_error().raw_os_error(), "DeleteFile: unlink failed");
            WinBool::FALSE
        }
    }
}

/// Flush file buffers to disk.
///
/// # Arguments
/// * `handle` - A Windows file handle returned by `CreateFile`.
///
/// # Returns
/// `TRUE` if the buffers were successfully flushed, or `FALSE` if an error occurred (e.g. invalid handle).
pub fn flush_file_buffers(handle: Handle) -> WinBool {
    match handle_table().get_fd(handle) {
        Some(fd) => {
            if unsafe { libc::fsync(fd) } == 0 {
                WinBool::TRUE
            } else {
                WinBool::FALSE
            }
        }
        _ => WinBool::FALSE,
    }
}

/// Get the size of a file in bytes.
///
///
/// # Arguments
/// * `handle` - A Windows file handle returned by `CreateFile`.
///
/// # Safety
/// * `handle` must be a valid file handle returned by `CreateFile`.
/// * The caller must ensure that the handle refers to a file object and not some other type of handle.
///
/// # Returns
/// The size of the file in bytes, or `None` if the handle is invalid or an error occurs.
pub fn get_file_size(handle: Handle) -> Option<u64> {
    let fd = handle_to_fd(handle)?;

    let mut stat: libc::stat = unsafe { std::mem::zeroed() };
    if unsafe { libc::fstat(fd, &mut stat) } != 0 {
        return None;
    }

    Some(stat.st_size as u64)
}

/// Read from a file handle into a buffer.
///
/// # Arguments
/// * `handle` - A Windows file handle returned by `CreateFile`.
/// * `buffer` - Pointer to a buffer to receive the data.
/// * `bytes_to_read` - Number of bytes to read.
/// * `bytes_read` - Optional output pointer for number of bytes actually read (can be null).
/// * `_overlapped` - Ignored.
///
/// # Safety
/// * `handle` must be a valid file handle returned by `CreateFile`.
/// * `buffer` must point to at least `bytes_to_read` bytes of valid memory
/// * The caller must ensure that the handle refers to a file object and not some other type of handle.
pub unsafe fn read_file(
    handle: Handle,
    buffer: *mut u8,
    bytes_to_read: u32,
    bytes_read: *mut u32,
    _overlapped: *mut core::ffi::c_void,
) -> WinBool {
    let Some(fd) = handle_to_fd(handle) else {
        return WinBool::FALSE;
    };

    let read = unsafe { libc::read(fd, buffer.cast(), bytes_to_read as usize) };
    if read < 0 {
        return WinBool::FALSE;
    }

    if !bytes_read.is_null() {
        unsafe { *bytes_read = read as u32 };
    }
    WinBool::TRUE
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Translate the directory portion of a FindFirstFile path to a Linux path.
pub fn translate_find_dir(dir_part: &str) -> std::path::PathBuf {
    if dir_part.is_empty() {
        return std::path::PathBuf::from(".");
    }
    translate_win_path(dir_part)
}

/// Translate a Windows path to a Linux path.
///
/// If the path already looks like a Linux path (`/…`), it's returned as-is.
/// Otherwise we apply a simple drive-letter mapping:
///   `X:\rest` → `~/.rine/drives/x/rest`
/// Backslashes are converted to forward slashes.
fn translate_win_path(win_path: &str) -> std::path::PathBuf {
    // Already a Linux absolute path — pass through.
    if win_path.starts_with('/') {
        return std::path::PathBuf::from(win_path);
    }

    let normalized = win_path.replace('\\', "/");

    // Strip \\?\ and \\.\ prefixes (now //?/ and //./).
    let stripped = normalized
        .strip_prefix("//?/")
        .or_else(|| normalized.strip_prefix("//./"))
        .unwrap_or(&normalized);

    // Check for drive letter: X:/…
    let bytes = stripped.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        let drive = (bytes[0] as char).to_ascii_lowercase();
        let rest = &stripped[2..];
        let rest = rest.strip_prefix('/').unwrap_or(rest);
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        let mut path = std::path::PathBuf::from(home);
        path.push(".rine/drives");
        path.push(drive.to_string());
        if !rest.is_empty() {
            path.push(rest);
        }
        return path;
    }

    // Relative or unrecognized — return as-is with normalized slashes.
    std::path::PathBuf::from(stripped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_file_path() -> PathBuf {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        path.push(format!(
            "rine_set_file_pointer_{}_{}",
            std::process::id(),
            nanos
        ));
        path
    }

    fn create_test_file_handle() -> (PathBuf, Handle) {
        let path = unique_temp_file_path();
        let raw = create_file(
            path.to_str()
                .unwrap_or("/tmp/rine_set_file_pointer_fallback"),
            GENERIC_READ | GENERIC_WRITE,
            CREATE_ALWAYS,
        );
        assert_ne!(raw, INVALID_HANDLE_VALUE.as_raw());
        (path, Handle::from_raw(raw))
    }

    fn cleanup_test_file(path: &PathBuf, handle: Handle) {
        if let Some(fd) = handle_to_fd(handle) {
            unsafe { libc::close(fd) };
        }
        let _ = handle_table().remove(handle);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn set_file_pointer_begin_and_current_work() {
        let (path, handle) = create_test_file_handle();

        let pos = unsafe { set_file_pointer(handle, 123, core::ptr::null_mut(), FILE_BEGIN) };
        assert_eq!(pos, 123);

        let pos = unsafe { set_file_pointer(handle, 7, core::ptr::null_mut(), FILE_CURRENT) };
        assert_eq!(pos, 130);

        cleanup_test_file(&path, handle);
    }

    #[test]
    fn set_file_pointer_sets_and_returns_high_bits_for_large_offsets() {
        let (path, handle) = create_test_file_handle();

        let mut high: i32 = 1;
        let low = unsafe { set_file_pointer(handle, 0, &mut high, FILE_BEGIN) };
        assert_eq!(low, 0);
        assert_eq!(high, 1);

        cleanup_test_file(&path, handle);
    }

    #[test]
    fn set_file_pointer_invalid_handle_returns_invalid_set_file_pointer() {
        let pos =
            unsafe { set_file_pointer(INVALID_HANDLE_VALUE, 0, core::ptr::null_mut(), FILE_BEGIN) };
        assert_eq!(pos, INVALID_SET_FILE_POINTER);
    }

    #[test]
    fn set_file_pointer_invalid_move_method_returns_invalid_set_file_pointer() {
        let (path, handle) = create_test_file_handle();

        let pos = unsafe { set_file_pointer(handle, 0, core::ptr::null_mut(), u32::MAX) };
        assert_eq!(pos, INVALID_SET_FILE_POINTER);

        cleanup_test_file(&path, handle);
    }
}
