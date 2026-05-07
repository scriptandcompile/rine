use rine_types::{
    errors::BOOL,
    handles::{
        CREATE_ALWAYS, CREATE_NEW, FILE_BEGIN, FILE_CURRENT, FILE_END, FindDataState, GENERIC_READ,
        GENERIC_WRITE, HANDLE, HFile, HandleEntry, INVALID_SET_FILE_POINTER, OPEN_ALWAYS,
        OPEN_EXISTING, TRUNCATE_EXISTING, Win32FindDataA, Win32FindDataW, collect_find_entries,
        handle_table, handle_to_fd, split_find_path, std_handle_to_fd,
    },
    strings::LPCSTR,
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
///
/// # Notes
/// Missing implementation features:
/// - Overlapped/asynchronous I/O is not implemented (`_overlapped` is ignored).
/// - This implementation does not set `GetLastError` on failure.
pub unsafe fn write_file(
    handle: HANDLE,
    buffer: *const u8,
    bytes_to_write: u32,
    bytes_written: *mut u32,
    _overlapped: *mut core::ffi::c_void,
) -> BOOL {
    let Some(fd) = handle_to_fd(handle) else {
        return BOOL::FALSE;
    };

    let written = unsafe { libc::write(fd, buffer.cast(), bytes_to_write as usize) };
    if written < 0 {
        return BOOL::FALSE;
    }

    if !bytes_written.is_null() {
        unsafe { *bytes_written = written as u32 };
    }
    BOOL::TRUE
}

/// SetFilePointer — move the file pointer for a file handle.
///
/// # Arguments
/// * `file` - The file handle whose pointer to move.
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
///
/// # Returns
/// The low 32 bits of the new file pointer on success, or INVALID_SET_FILE_POINTER (0xFFFFFFFF) on failure.
/// If the return value is INVALID_SET_FILE_POINTER, the caller should call `GetLastError` to determine
/// if an error occurred or if the new file pointer is actually at 0xFFFFFFFF.
/// Currently, this implementation does not set the error code, so it will return INVALID_SET_FILE_POINTER on
/// failure and 0xFFFFFFFF on success if the new file pointer is exactly 0xFFFFFFFF.
pub unsafe fn set_file_pointer(
    handle: HANDLE,
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
/// On failure, returns `HANDLE::INVALID`.
pub fn create_file(win_path: &str, desired_access: u32, creation_disposition: u32) -> HANDLE {
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
            return HANDLE::INVALID;
        }
    }

    // Translate Windows path → Linux path.
    let linux_path = translate_win_path(win_path);

    let c_path = match std::ffi::CString::new(linux_path.to_string_lossy().as_bytes()) {
        Ok(s) => s,
        Err(_) => return HANDLE::INVALID,
    };

    let mode: libc::mode_t = 0o644;
    let fd = unsafe { libc::open(c_path.as_ptr(), flags, mode as libc::c_uint) };
    if fd < 0 {
        tracing::debug!(path = %linux_path.display(), errno = std::io::Error::last_os_error().raw_os_error(), "CreateFile: open failed");
        return HANDLE::INVALID;
    }

    let h = handle_table().insert(HandleEntry::File(fd));
    tracing::debug!(handle = ?h, fd, path = %linux_path.display(), "CreateFile: opened");
    rine_types::dev_notify!(on_handle_created(
        h.as_raw() as i64,
        "File",
        &linux_path.display().to_string()
    ));
    h
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
/// `BOOL::TRUE` on success, `BOOL::FALSE` on failure.
///
/// # Note
/// This implementation only supports closing of file handles.
/// Currently it does not support closing of other handle types tracked in the handle table
/// (threads, events, processes, mutexes, semaphores, heaps, registry keys, and FindFirstFile find data).
/// It does not support closing of window handles, which are not tracked in the handle table.
#[allow(non_snake_case)]
pub fn close_handle(handle: HANDLE) -> BOOL {
    match handle_table().remove(handle) {
        Some(HandleEntry::Thread(_)) => BOOL::TRUE,
        Some(HandleEntry::Event(_)) => BOOL::TRUE,
        Some(HandleEntry::Process(_)) => BOOL::TRUE,
        Some(HandleEntry::Mutex(_)) => BOOL::TRUE,
        Some(HandleEntry::Semaphore(_)) => BOOL::TRUE,
        Some(HandleEntry::Heap(_)) => BOOL::TRUE,
        Some(HandleEntry::RegistryKey(_)) => BOOL::TRUE,
        Some(HandleEntry::FindData(_)) => BOOL::TRUE,
        Some(HandleEntry::File(object)) => {
            let Some(fd) = std_handle_to_fd(object as u32) else {
                return BOOL::FALSE;
            };

            // 'fd' should be the linux file descriptor.
            // If it's a standard stream, we don't actually want to close it.
            if fd == libc::STDERR_FILENO || fd == libc::STDOUT_FILENO || fd == libc::STDIN_FILENO {
                BOOL::TRUE
            } else {
                unsafe { libc::close(fd) };
                BOOL::TRUE
            }
        }
        Some(HandleEntry::Window(_)) => BOOL::FALSE,
        None => BOOL::FALSE,
    }
}

/// Delete a file at the given Windows path.
///
/// # Arguments
/// * `win_path`: Windows-style file path (e.g. `C:\foo\bar.txt`).
///
/// # Returns
/// `BOOL::TRUE` if the file was successfully deleted, `BOOL::FALSE` if an error occurred (e.g. file not found).
pub fn delete_file(win_path: &str) -> BOOL {
    tracing::debug!(path = win_path, "DeleteFile");

    let linux_path = translate_win_path(win_path);
    let c_path = match std::ffi::CString::new(linux_path.to_string_lossy().as_bytes()) {
        Ok(s) => s,
        Err(_) => return BOOL::FALSE,
    };

    match unsafe { libc::unlink(c_path.as_ptr()) } {
        0 => BOOL::TRUE,
        _ => {
            tracing::debug!(path = %linux_path.display(), errno = std::io::Error::last_os_error().raw_os_error(), "DeleteFile: unlink failed");
            BOOL::FALSE
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
pub fn flush_file_buffers(handle: HANDLE) -> BOOL {
    match handle_table().get_fd(handle) {
        Some(fd) => {
            if unsafe { libc::fsync(fd) } == 0 {
                BOOL::TRUE
            } else {
                BOOL::FALSE
            }
        }
        _ => BOOL::FALSE,
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
pub fn get_file_size(handle: HANDLE) -> Option<u64> {
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
///
/// # Notes
/// Missing implementation features:
/// - Overlapped/asynchronous I/O is not implemented (`_overlapped` is ignored).
/// - This implementation does not set `GetLastError` on failure.
pub unsafe fn read_file(
    handle: HANDLE,
    buffer: *mut u8,
    bytes_to_read: u32,
    bytes_read: *mut u32,
    _overlapped: *mut core::ffi::c_void,
) -> BOOL {
    let Some(fd) = handle_to_fd(handle) else {
        return BOOL::FALSE;
    };

    let read = unsafe { libc::read(fd, buffer.cast(), bytes_to_read as usize) };
    if read < 0 {
        return BOOL::FALSE;
    }

    if !bytes_read.is_null() {
        unsafe { *bytes_read = read as u32 };
    }
    BOOL::TRUE
}

/// Begin searching for files matching a pattern (ANSI).
///
/// # Arguments
/// * `file_path` - Windows-style file path with optional wildcards (e.g. `C:\foo\*.txt`).
/// * `find_data` - Output pointer for file data of the first matching file. Must point to a writable `WIN32_FIND_DATAA` structure.
///
/// # Safety
/// `find_data` must point to a writable `WIN32_FIND_DATAA`.
/// The caller is responsible for calling `FindClose` with the returned handle when the search is finished.
///
/// # Returns
/// A search handle that can be used with `FindNextFile` and `FindClose`, or `HANDLE::INVALID` if no
/// matching files were found or an error occurred.
pub unsafe fn find_first_file_a(file_path: &str, find_data: *mut Win32FindDataA) -> HANDLE {
    if find_data.is_null() {
        return HANDLE::INVALID;
    }

    let (dir_part, pattern) = split_find_path(file_path);

    let linux_dir = translate_find_dir(dir_part);
    let entries = collect_find_entries(&linux_dir, pattern);
    if entries.is_empty() {
        return HANDLE::INVALID;
    }

    // Write the first entry.
    unsafe { core::ptr::write(find_data, Win32FindDataA::from_entry(&entries[0])) };

    let handle = handle_table().insert(HandleEntry::FindData(FindDataState { entries, cursor: 1 }));
    rine_types::dev_notify!(on_handle_created(
        handle.as_raw() as i64,
        "FindData",
        file_path
    ));

    handle
}

/// Begin searching for files matching a pattern (wide).
///
/// # Arguments
/// * `file_path` - Windows-style file path with optional wildcards (e.g. `C:\foo\*.txt`).
/// * `find_data` - Output pointer for file data of the first matching file. Must point to a writable `WIN32_FIND_DATAW` structure.
///
/// # Safety
/// `find_data` must point to a writable `WIN32_FIND_DATAW`.
/// The caller is responsible for calling `FindClose` with the returned handle when the search is finished.
///
/// # Returns
/// A search handle that can be used with `FindNextFile` and `FindClose`, or `HANDLE::INVALID` if no
/// matching files were found or an error occurred.
pub unsafe fn find_first_file_w(file_path: &str, find_data: *mut Win32FindDataW) -> HANDLE {
    if find_data.is_null() {
        return HANDLE::INVALID;
    }

    let (dir_part, pattern) = split_find_path(file_path);

    let linux_dir = translate_find_dir(dir_part);
    let entries = collect_find_entries(&linux_dir, pattern);
    if entries.is_empty() {
        return HANDLE::INVALID;
    }

    // Write the first entry.
    unsafe { core::ptr::write(find_data, Win32FindDataW::from_entry(&entries[0])) };

    let handle = handle_table().insert(HandleEntry::FindData(FindDataState { entries, cursor: 1 }));
    rine_types::dev_notify!(on_handle_created(
        handle.as_raw() as i64,
        "FindData",
        file_path
    ));

    handle
}

/// Continue a directory search (ANSI).
///
/// # Arguments
/// * `handle` - A search handle returned by `FindFirstFileA`.
/// * `find_data` - Output pointer for file data of the next matching file. Must point to a writable `WIN32_FIND_DATAA` structure.
///
/// # Safety
/// `find_data` must point to a writable `WIN32_FIND_DATAA`.
/// The caller is responsible for calling `FindClose` with the search handle when the search is finished.
///
/// # Returns
/// `BOOL::TRUE` if the next matching file was found and `find_data` was updated,
/// or `BOOL::FALSE` if no more matching files were found or an error occurred.
#[unsafe(no_mangle)]
pub unsafe fn find_next_file_a(handle: HANDLE, find_data: *mut Win32FindDataA) -> BOOL {
    if find_data.is_null() {
        return BOOL::FALSE;
    }

    let result = handle_table().with_find_data(handle, |state| {
        if state.cursor >= state.entries.len() {
            return false;
        }
        let entry = &state.entries[state.cursor];
        unsafe { core::ptr::write(find_data, Win32FindDataA::from_entry(entry)) };
        state.cursor += 1;
        true
    });

    match result {
        Some(true) => BOOL::TRUE,
        _ => BOOL::FALSE,
    }
}

/// Continue a directory search (wide).
///
/// # Arguments
/// * `handle` - A search handle returned by `FindFirstFileW`.
/// * `find_data` - Output pointer for file data of the next matching file. Must point to a writable `WIN32_FIND_DATAW` structure.
///
/// # Safety
/// `find_data` must point to a writable `WIN32_FIND_DATAW`.
/// The caller is responsible for calling `FindClose` with the search handle when the search is finished.
///
/// # Returns
/// `BOOL::TRUE` if the next matching file was found and `find_data` was updated,
/// or `BOOL::FALSE` if no more matching files were found or an error occurred.
#[unsafe(no_mangle)]
pub unsafe fn find_next_file_w(handle: HANDLE, find_data: *mut Win32FindDataW) -> BOOL {
    if find_data.is_null() {
        return BOOL::FALSE;
    }

    let result = handle_table().with_find_data(handle, |state| {
        if state.cursor >= state.entries.len() {
            return false;
        }
        let entry = &state.entries[state.cursor];
        unsafe { core::ptr::write(find_data, Win32FindDataW::from_entry(entry)) };
        state.cursor += 1;
        true
    });

    match result {
        Some(true) => BOOL::TRUE,
        _ => BOOL::FALSE,
    }
}

/// Open a file handle from the legacy _lopen API.
///
/// # Arguments
/// * `_lppathname` - Windows-style file path (e.g. `C:\foo\bar.txt`).
/// * `_ireadwrite` - Access mode (0 for read-only, 1 for write-only, 2 for read/write).
///
/// # Returns
/// A file handle on success, or `HFile::NULL` on failure.
///
/// # Notes
/// The _lopen/_lclose APIs are legacy and not commonly used.
/// This is a stub implementation that doesn't actually track or open these handles,
/// but it allows the DLLs to link successfully if they reference _lopen.
pub fn _lopen(_lppathname: LPCSTR, _ireadwrite: i32) -> HFile {
    // We don't support the legacy _lopen API, but we need to provide a stub implementation to link successfully.
    // Just return a non-error value (HFile::NULL is the error value for _lopen, so we return something else to indicate success).
    HFile::NULL
}

/// Close a file handle from the legacy _lclose API.
///
/// # Arguments
/// * `hfile` - The file handle to close.
///
/// # Returns
/// The input `hfile` on success, or an error code on failure.
///
/// # Notes
/// The _lopen/_lclose APIs are legacy and not commonly used.
/// This is a stub implementation that doesn't actually track or close these handles,
/// but it allows the DLLs to link successfully if they reference _lclose.
pub fn _lclose(hfile: HFile) -> HFile {
    // HFile is a 16-bit API handle type used by legacy file I/O APIs like _lopen/_lclose.
    // We don't support those APIs, but we need to provide a stub implementation to link successfully.
    // Just return the input value, which is what the Windows implementation does on success.
    hfile
}

/// Read from a file handle using the legacy _lread API.
///
/// # Arguments
/// * `_hfile` - The file handle to read from.
/// * `_buffer` - Pointer to a buffer to receive the data.
/// * `_count` - Number of bytes to read.
///
/// # Returns
/// The number of bytes read on success, or an error code on failure.
///
/// # Notes
/// The _lopen/_lclose APIs are legacy and not commonly used.
/// This is a stub implementation that doesn't actually track or read from these handles,
/// but it allows the DLLs to link successfully if they reference _lread.
pub fn _lread(_hfile: HFile, _buffer: *mut core::ffi::c_void, _count: u32) -> i32 {
    // Stub implementation for legacy _lread API.
    // This API is not commonly used and we don't need it for our purposes, but we provide a stub to link successfully.
    // Just return an error code to indicate failure.
    HFile::INVALID.as_raw()
}

/// Write to a file handle using the legacy _lwrite API.
///
/// # Arguments
/// * `_hfile` - The file handle to write to.
/// * `_buffer` - Pointer to the data to write.
/// * `_count` - Number of bytes to write.
///
/// # Returns
/// The number of bytes written on success, or an error code on failure.
///
/// # Notes
/// The _lopen/_lclose APIs are legacy and not commonly used.
/// This is a stub implementation that doesn't actually track or write to these handles,
/// but it allows the DLLs to link successfully if they reference _lwrite.
pub fn _lwrite(_hfile: HFile, _buffer: *const core::ffi::c_void, _count: u32) -> i32 {
    // Stub implementation for legacy _lwrite API.
    // This API is not commonly used and we don't need it for our purposes, but we provide a stub to link successfully.
    // Just return an error code to indicate failure.
    HFile::INVALID.as_raw()
}

/// Move the file pointer for a file handle using the legacy _llseek API.
///
/// # Arguments
/// * `_hfile` - The file handle whose pointer to move.
/// * `_offset` - The distance to move the file pointer, in bytes. Can be negative to move backwards.
/// * `_origin` - The starting point for the move. Must be one of `FILE_BEGIN` (0), `FILE_CURRENT` (1), or `FILE_END` (2).
///
/// # Returns
/// The new file pointer position on success, or an error code on failure.
/// Currently always returns `HFILE_ERROR` (-1).
///
/// # Notes
/// The _lopen/_lclose APIs are legacy and not commonly used.
/// This is a stub implementation that doesn't actually track or move these handles,
/// This function does not currently report an error through `GetLastError`.
pub fn _llseek(_hfile: HFile, _offset: i64, _origin: u32) -> i64 {
    // Stub implementation for legacy _llseek API.
    // This API is not commonly used and we don't need it for our purposes, but we provide a stub to link successfully.
    // Just return an error code to indicate failure.
    HFile::INVALID.as_raw() as i64
}

/// Create a file handle using the legacy _lcreat API.
///
/// # Arguments
/// * `_lppathname` - Windows-style file path (e.g. `C:\foo\bar.txt`).
/// * `_iattribute` - File attribute flags.
///   Normal (0), Can be read from or written to without restrictions.
///   Read-only (1), Cannot be written to. Attempting to write will fail with an error.
///   Hidden (2), Not visible when enumerating files in a directory. This attribute has no effect on file access permissions.
///   System (4), Reserved for use by the operating system. This attribute has no effect on file access permissions.
///
/// # Safety
/// * `_lppathname` must be a valid Windows-style file path string.
/// * `_iattribute` must be a valid file attribute flag value (0, 1, 2, or 4).
///
/// # Returns
/// A file handle on success, or `HFile::INVALID` on failure.
/// Currently always returns `HFile::INVALID` since we don't support this legacy API.
///
/// # Notes
/// The _lopen/_lclose APIs are legacy and not commonly used.
/// This is a stub implementation that doesn't actually track or create these handles,
/// but it allows the DLLs to link successfully if they reference _lcreat.
pub fn _lcreat(_lppathname: LPCSTR, _iattribute: i32) -> HFile {
    // Stub implementation for legacy _lcreat API.
    // This API is not commonly used and we don't need it for our purposes, but we provide a stub to link successfully.
    // Just return an error code to indicate failure.
    HFile::INVALID
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

    fn create_test_file_handle() -> (PathBuf, HANDLE) {
        let path = unique_temp_file_path();
        let raw = create_file(
            path.to_str()
                .unwrap_or("/tmp/rine_set_file_pointer_fallback"),
            GENERIC_READ | GENERIC_WRITE,
            CREATE_ALWAYS,
        );
        assert_ne!(raw, HANDLE::INVALID);
        (path, raw)
    }

    fn cleanup_test_file(path: &PathBuf, handle: HANDLE) {
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
            unsafe { set_file_pointer(HANDLE::INVALID, 0, core::ptr::null_mut(), FILE_BEGIN) };
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
