//! kernel32 file I/O: CreateFileA/W, ReadFile, WriteFile, CloseHandle,
//! GetFileSize, SetFilePointer, FindFirstFileA/W, FindNextFileA/W, FindClose.

use rine_common_kernel32 as common;
use rine_types::errors::BOOL;
use rine_types::handles::{HANDLE, HFILE, INVALID_FILE_SIZE, Win32FindDataA, Win32FindDataW};
use rine_types::strings::{LPCSTR, LPCWSTR};

/// CreateFileA — open or create a file (ANSI path).
///
/// # Arguments
/// * `file_name`: pointer to a null-terminated ANSI string with the file path.
/// * `desired_access`: bitmask of GENERIC_READ, GENERIC_WRITE, etc.
/// * `creation_disposition`: action to take on files that exist or do not exist.
/// * _share_mode - ignored
/// * _security_attributes - ignored
/// * _flags_and_attributes - ignored
/// * _template_file - ignored
///
/// # Safety
/// `file_name` must be a valid null-terminated ANSI string.
/// The caller must ensure that the file path is valid and that the desired
/// access and creation disposition are appropriate.
///
/// # Returns
/// A file handle on success, or INVALID_HANDLE_VALUE on failure.
///
/// # Note
/// This implementation does not support all features of the Windows API, such as
/// sharing modes, security attributes, or file attributes. It focuses on basic
/// file creation and opening functionality.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn CreateFileA(
    file_name: LPCSTR,
    desired_access: u32,
    _share_mode: u32,
    _security_attributes: usize,
    creation_disposition: u32,
    _flags_and_attributes: u32,
    _template_file: HANDLE,
) -> HANDLE {
    if file_name.is_null() {
        return HANDLE::NULL;
    }

    let c_str = unsafe { file_name.read_string().unwrap_or_default() };
    let path_str = c_str.to_string();

    common::file::create_file(&path_str, desired_access, creation_disposition)
}

/// CreateFileW — open or create a file (wide/UTF-16 path).
///
/// # Arguments
/// * `file_name`: pointer to a null-terminated UTF-16LE string with the file path.
/// * `desired_access`: bitmask of GENERIC_READ, GENERIC_WRITE, etc.
/// * `creation_disposition`: action to take on files that exist or do not exist.
/// * _share_mode - ignored
/// * _security_attributes - ignored
/// * _flags_and_attributes - ignored
/// * _template_file - ignored
///
/// # Safety
/// `file_name` must be a valid null-terminated UTF-16LE string.
/// The caller must ensure that the file path is valid and that the desired
/// access and creation disposition are appropriate.
///
/// # Returns
/// A file handle on success, or INVALID_HANDLE_VALUE on failure.
///
/// # Note
/// This implementation does not support all features of the Windows API, such as
/// sharing modes, security attributes, or file attributes. It focuses on basic
/// file creation and opening functionality.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn CreateFileW(
    file_name: LPCWSTR,
    desired_access: u32,
    _share_mode: u32,
    _security_attributes: usize,
    creation_disposition: u32,
    _flags_and_attributes: u32,
    _template_file: HANDLE,
) -> HANDLE {
    if file_name.is_null() {
        return HANDLE::NULL;
    }

    let wide_file_name = unsafe { file_name.read_string().unwrap_or_default() };
    let path_str = wide_file_name.to_string();

    common::file::create_file(&path_str, desired_access, creation_disposition)
}

/// DeleteFileW — delete a file (wide/UTF-16 path).
///
/// # Arguments
/// * `file_name`: pointer to a null-terminated UTF-16LE string with the file path.
///
/// # Safety
/// `file_name` must be a valid null-terminated UTF-16LE string.
/// The caller must ensure that the file path is valid and that the file can be deleted.
///
/// # Returns
/// `BOOL::TRUE` on success, `BOOL::FALSE` on failure.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn DeleteFileW(file_name: LPCWSTR) -> BOOL {
    if file_name.is_null() {
        return BOOL::FALSE;
    }

    let wide_file_name = unsafe { file_name.read_string().unwrap_or_default() };
    let path_str = wide_file_name.to_string();
    common::file::delete_file(&path_str)
}

/// DeleteFileA — delete a file (ANSI path).
///
/// # Arguments
/// * `file_name`: pointer to a null-terminated ANSI string with the file path.
///
/// # Safety
/// `file_name` must be a valid null-terminated ANSI string.
/// The caller must ensure that the file path is valid and that the file can be deleted.
///
/// # Returns
/// `BOOL::TRUE` on success, `BOOL::FALSE` on failure.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn DeleteFileA(file_name: LPCSTR) -> BOOL {
    if file_name.is_null() {
        return BOOL::FALSE;
    }

    let c_str = unsafe { file_name.read_string().unwrap_or_default() };
    let path_str = c_str.to_string();
    common::file::delete_file(&path_str)
}

/// ReadFile — read data from a file.
///
/// # Arguments
/// * `file` - The file handle to read from. Must be a valid file handle returned by `CreateFile`.
/// * `buffer` - Pointer to a buffer that receives the data read from the file.
/// * `bytes_to_read` - The number of bytes to read.
/// * `bytes_read` - Optional pointer to a variable that receives the number of bytes read.
/// * `_overlapped` - ignored (asynchronous I/O is not supported).
///
/// # Safety
/// `file` must be a valid file handle returned by `CreateFile`.
/// `buffer` must be writable for at least `bytes_to_read` bytes.
/// `bytes_read` must be null or point to a valid u32 variable.
/// `_overlapped` must be null or point to a valid OVERLAPPED structure,
/// but asynchronous I/O is not supported so it will be ignored.
///
/// # Returns
/// `BOOL::TRUE` on success, `BOOL::FALSE` on failure.
///
/// # Notes
/// Missing implementation features:
/// - Overlapped/asynchronous I/O is not implemented (`_overlapped` is ignored).
/// - This implementation does not set `GetLastError` on failure.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn ReadFile(
    file: HANDLE,
    buffer: *mut u8,
    bytes_to_read: u32,
    bytes_read: *mut u32,
    _overlapped: *mut core::ffi::c_void,
) -> BOOL {
    unsafe { common::file::read_file(file, buffer, bytes_to_read, bytes_read, _overlapped) }
}

/// WriteFile — write data to a file or I/O device.
///
/// # Arguments
/// * `file` - The file handle to write to. Must be a valid file handle returned by `CreateFile`.
/// * `buffer` - Pointer to the data to be written to the file.
/// * `bytes_to_write` - The number of bytes to write.
/// * `bytes_written` - Optional pointer to a variable that receives the number of bytes written.
/// * `_overlapped` - ignored (asynchronous I/O is not supported).
///
/// # Safety
/// `file` must be a valid file handle returned by `CreateFile`.
/// `buffer` must point to at least `bytes_to_write` readable bytes.
/// `bytes_written` must be null or point to a valid u32 variable.
/// `_overlapped` must be null or point to a valid OVERLAPPED structure,
/// but asynchronous I/O is not supported so it will be ignored.
///
/// # Returns
/// `BOOL::TRUE` on success, `BOOL::FALSE` on failure.
///
/// # Notes
/// Missing implementation features:
/// - Overlapped/asynchronous I/O is not implemented (`_overlapped` is ignored).
/// - This implementation does not set `GetLastError` on failure.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn WriteFile(
    file: HANDLE,
    buffer: *const u8,
    bytes_to_write: u32,
    bytes_written: *mut u32,
    _overlapped: *mut core::ffi::c_void,
) -> BOOL {
    unsafe { common::file::write_file(file, buffer, bytes_to_write, bytes_written, _overlapped) }
}

/// FlushFileBuffers — flush file buffers to disk.
///
/// # Arguments
/// * `file` - The file handle to flush. Must be a valid file handle returned by `CreateFile`.
///
/// # Safety
/// `file` must be a valid file handle returned by `CreateFile`.
///
/// # Returns
/// `BOOL::TRUE` on success, `BOOL::FALSE` on failure.
///
/// # Note
/// This implementation does not support flushing of non-file handles (e.g. pipes, consoles).
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn FlushFileBuffers(file: HANDLE) -> BOOL {
    common::file::flush_file_buffers(file)
}

/// CloseHandle — close an open object handle (e.g. file handle).
///
/// # Arguments
/// * `object` - The handle to close. Must be a valid handle returned by `CreateFile` or other handle-returning function.
///
/// # Safety
/// `object` must be a valid handle returned by `CreateFile` or other handle-returning function.
/// After this call, `object` must not be used again.
///
/// # Returns
/// `BOOL::TRUE` on success, `BOOL::FALSE` on failure.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn CloseHandle(object: HANDLE) -> BOOL {
    rine_types::dev_notify!(on_handle_closed(object.as_raw() as i64));

    common::file::close_handle(object)
}

/// GetFileSize — return the size of a file in bytes.
///
/// Returns the low 32 bits. If `file_size_high` is non-null, the high
/// 32 bits are written there.
///
/// # Arguments
/// * `file` - The file handle to query. Must be a valid file handle returned by `CreateFile`.
/// * `file_size_high` - Optional pointer to receive the high 32 bits of the file size.
///   If the file size exceeds 4GB, this must be non-null and will be set to the high bits of the file size.
///   If the file size is 4GB or less, this can be null or will be set to zero.
///
/// # Safety
/// * `file` must be a valid file handle returned by `CreateFile`.
/// * `file_size_high` must be null or point to a valid u32 variable.
///
/// # Returns
/// The low 32 bits of the file size on success, or INVALID_FILE_SIZE (0xFFFFFFFF) on failure.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetFileSize(file: HANDLE, file_size_high: *mut u32) -> u32 {
    let Some(size) = common::file::get_file_size(file) else {
        return INVALID_FILE_SIZE;
    };

    if !file_size_high.is_null() {
        unsafe { *file_size_high = (size >> 32) as u32 };
    }
    size as u32
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
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn SetFilePointer(
    file: HANDLE,
    distance_to_move: i32,           // low 32 bits
    distance_to_move_high: *mut i32, // high 32 bits (in/out, optional)
    move_method: u32,
) -> u32 {
    unsafe {
        common::file::set_file_pointer(file, distance_to_move, distance_to_move_high, move_method)
    }
}

// ---------------------------------------------------------------------------
// FindFirstFileA / FindFirstFileW
// ---------------------------------------------------------------------------

/// Begin searching for files matching a pattern (ansi).
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
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn FindFirstFileA(
    file_name: LPCSTR,
    find_data: *mut Win32FindDataA,
) -> HANDLE {
    if file_name.is_null() {
        return HANDLE::INVALID;
    }

    unsafe {
        let Some(path_str) = file_name.read_string() else {
            return HANDLE::INVALID;
        };

        common::file::find_first_file_a(&path_str, find_data)
    }
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
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn FindFirstFileW(
    file_name: LPCWSTR,
    find_data: *mut Win32FindDataW,
) -> HANDLE {
    if file_name.is_null() {
        return HANDLE::INVALID;
    }
    unsafe {
        let Some(path_str) = file_name.read_string() else {
            return HANDLE::INVALID;
        };

        common::file::find_first_file_w(&path_str, find_data)
    }
}

// ---------------------------------------------------------------------------
// FindNextFileA / FindNextFileW
// ---------------------------------------------------------------------------

/// Continue a directory search (ansi).
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
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn FindNextFileA(
    find_file: HANDLE,
    find_data: *mut Win32FindDataA,
) -> BOOL {
    if find_data.is_null() {
        return BOOL::FALSE;
    }

    unsafe { common::file::find_next_file_a(find_file, find_data) }
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
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn FindNextFileW(
    find_file: HANDLE,
    find_data: *mut Win32FindDataW,
) -> BOOL {
    if find_data.is_null() {
        return BOOL::FALSE;
    }

    unsafe { common::file::find_next_file_w(find_file, find_data) }
}

/// FindClose — close a search handle opened by FindFirstFile.
///
/// # Arguments
/// * `find_file` - The search handle returned by `FindFirstFile`.
///
/// # Safety
/// * `find_file` must be a valid search handle returned by `FindFirstFile`.
/// * After this call, `find_file` must not be used again.
///
/// # Returns
/// `BOOL::TRUE` on success, `BOOL::FALSE` on failure.
///
/// # Note
/// This implementation does not set the error code and will currently always return `BOOL::TRUE` at the moment.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn FindClose(find_file: HANDLE) -> BOOL {
    unsafe { CloseHandle(find_file) }
}

// ----- Legacy 16 bit windows APIs (not commonly used) -----

/// Open a file handle from the legacy _lopen API.
///
/// # Arguments
/// * `_lppathname` - Windows-style file path (e.g. `C:\foo\bar.txt`).
/// * `_ireadwrite` - Access mode (0 for read-only, 1 for write-only, 2 for read/write).
///
/// # Safety
/// `_lppathname` must be a valid null-terminated ANSI string.
/// The caller must ensure that the file path is valid and that the access mode is appropriate.
///
/// # Returns
/// A file handle on success, or `HFILE::NULL` on failure.
///
/// # Notes
/// The _lopen/_lclose APIs are legacy and not commonly used.
/// This is a stub implementation that doesn't actually track or open these handles,
/// but it allows the DLLs to link successfully if they reference _lopen.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn _lopen(_lppathname: LPCSTR, _ireadwrite: i32) -> HFILE {
    common::file::_lopen(_lppathname, _ireadwrite)
}

/// Close a file handle from the legacy _lopen API.
///
/// # Arguments
/// * `hfile` - The file handle to close.
///
/// # Safety
/// `hfile` must be a valid file handle returned by `_lopen`.
/// After this call, `hfile` must not be used again.
///
/// # Returns
/// The input `hfile` on success, or an error code on failure.
///
/// # Notes
/// The _lopen/_lclose APIs are legacy and not commonly used.
/// This is a stub implementation that doesn't actually track or close these handles,
/// but it allows the DLLs to link successfully if they reference _lclose.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn _lclose(hfile: HFILE) -> HFILE {
    common::file::_lclose(hfile)
}

/// Read from a file handle using the legacy _lread API.
///
/// # Arguments
/// * `_hfile` - The file handle to read from.
/// * `_buffer` - Pointer to a buffer to receive the data.
/// * `_count` - Number of bytes to read.
///
/// # Safety
/// `_hfile` must be a valid file handle returned by `_lopen`.
/// `_buffer` must point to at least `_count` writable bytes.
/// After this call, the caller must ensure that the file handle is properly closed with `_lclose`.
///
/// # Returns
/// The number of bytes read on success, or an error code on failure.
///
/// # Notes
/// The _lopen/_lclose APIs are legacy and not commonly used.
/// This is a stub implementation that doesn't actually track or read from these handles,
/// but it allows the DLLs to link successfully if they reference _lread.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn _lread(
    _hfile: HFILE,
    _buffer: *mut core::ffi::c_void,
    _count: u32,
) -> i32 {
    common::file::_lread(_hfile, _buffer, _count)
}

/// Write to a file handle using the legacy _lwrite API.
///
/// # Arguments
/// * `_hfile` - The file handle to write to.
/// * `_buffer` - Pointer to the data to write.
/// * `_count` - Number of bytes to write.
///
/// # Safety
/// `_hfile` must be a valid file handle returned by `_lopen`.
/// `_buffer` must point to at least `_count` readable bytes.
/// After this call, the caller must ensure that the file handle is properly closed with `_lclose`.
///
/// # Returns
/// The number of bytes written on success, or an error code on failure.
///
/// # Notes
/// The _lopen/_lclose APIs are legacy and not commonly used.
/// This is a stub implementation that doesn't actually track or write to these handles,
/// but it allows the DLLs to link successfully if they reference _lwrite.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn _lwrite(
    _hfile: HFILE,
    _buffer: *const core::ffi::c_void,
    _count: u32,
) -> i32 {
    common::file::_lwrite(_hfile, _buffer, _count)
}

/// Move the file pointer for a file handle using the legacy _llseek API.
///
/// # Arguments
/// * `_hfile` - The file handle whose pointer to move.
/// * `_offset` - The distance to move the file pointer, in bytes. Can be negative to move backwards.
/// * `_origin` - The starting point for the move. Must be one of `FILE_BEGIN` (0), `FILE_CURRENT` (1), or `FILE_END` (2).
///
/// # Safety
/// `_hfile` must be a valid file handle returned by `_lopen`.
/// After this call, the caller must ensure that the file handle is properly closed with `_lclose`.
/// `_origin` must be a valid seek method must be one of `FILE_BEGIN` (0), `FILE_CURRENT` (1), or `FILE_END` (2).
///
/// # Returns
/// The new file pointer position on success, or an error code on failure.
/// Currently always returns `HFILE_ERROR` (-1).
///
/// # Notes
/// The _lopen/_lclose APIs are legacy and not commonly used.
/// This is a stub implementation that doesn't actually track or move these handles,
/// This function does not currently report an error through `GetLastError`.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn _llseek(_hfile: HFILE, _offset: i64, _origin: u32) -> i64 {
    common::file::_llseek(_hfile, _offset, _origin)
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
/// A file handle on success, or `HFILE::INVALID` on failure.
/// Currently always returns `HFILE::INVALID` since we don't support this legacy API.
///
/// # Notes
/// The _lopen/_lclose APIs are legacy and not commonly used.
/// This is a stub implementation that doesn't actually track or create these handles,
/// but it allows the DLLs to link successfully if they reference _lcreat.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn _lcreat(_lppathname: LPCSTR, _iattribute: i32) -> HFILE {
    common::file::_lcreat(_lppathname, _iattribute)
}
