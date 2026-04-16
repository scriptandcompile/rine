use rine_common_kernel32 as common;
use rine_types::handles::{
    Handle, INVALID_FILE_SIZE, INVALID_HANDLE_VALUE, Win32FindDataA, Win32FindDataW,
};
use rine_types::{
    errors::WinBool,
    strings::{read_cstr, read_wstr},
};

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
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn CreateFileA(
    file_name: *const u8,
    desired_access: u32,
    _share_mode: u32,
    _security_attributes: usize, // LPSECURITY_ATTRIBUTES (ignored)
    creation_disposition: u32,
    _flags_and_attributes: u32,
    _template_file: isize, // HANDLE (ignored)
) -> isize {
    if file_name.is_null() {
        return INVALID_HANDLE_VALUE.as_raw();
    }

    let c_str = unsafe { read_cstr(file_name).unwrap_or_default() };
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
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn CreateFileW(
    file_name: *const u16,
    desired_access: u32,
    _share_mode: u32,
    _security_attributes: usize, // LPSECURITY_ATTRIBUTES (ignored)
    creation_disposition: u32,
    _flags_and_attributes: u32,
    _template_file: isize, // HANDLE (ignored)
) -> isize {
    if file_name.is_null() {
        return INVALID_HANDLE_VALUE.as_raw();
    }

    let wide_file_name = unsafe { read_wstr(file_name).unwrap_or_default() };
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
/// `WinBool::TRUE` on success, `WinBool::FALSE` on failure.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn DeleteFileW(file_name: *const u16) -> WinBool {
    if file_name.is_null() {
        return WinBool::FALSE;
    }

    let wide_file_name = unsafe { read_wstr(file_name).unwrap_or_default() };
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
/// `WinBool::TRUE` on success, `WinBool::FALSE` on failure.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn DeleteFileA(file_name: *const u8) -> WinBool {
    if file_name.is_null() {
        return WinBool::FALSE;
    }

    let c_str = unsafe { read_cstr(file_name).unwrap_or_default() };
    let path_str = c_str.to_string();
    common::file::delete_file(&path_str)
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
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetFileSize(file: isize, file_size_high: *mut u32) -> u32 {
    let handle = Handle::from_raw(file);

    let Some(size) = common::file::get_file_size(handle) else {
        return INVALID_FILE_SIZE;
    };

    if !file_size_high.is_null() {
        unsafe { *file_size_high = (size >> 32) as u32 };
    }
    size as u32
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
/// `WinBool::TRUE` on success, `WinBool::FALSE` on failure.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn WriteFile(
    file: isize,
    buffer: *const u8,
    bytes_to_write: u32,
    bytes_written: *mut u32,
    _overlapped: *mut core::ffi::c_void,
) -> WinBool {
    let handle = Handle::from_raw(file);
    unsafe { common::file::write_file(handle, buffer, bytes_to_write, bytes_written, _overlapped) }
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
/// `WinBool::TRUE` on success, `WinBool::FALSE` on failure.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn ReadFile(
    file: isize,
    buffer: *mut u8,
    bytes_to_read: u32,
    bytes_read: *mut u32,
    _overlapped: *mut core::ffi::c_void,
) -> WinBool {
    let handle = Handle::from_raw(file);

    unsafe { common::file::read_file(handle, buffer, bytes_to_read, bytes_read, _overlapped) }
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
/// `WinBool::TRUE` on success, `WinBool::FALSE` on failure.
///
/// # Note
/// This implementation does not support flushing of non-file handles (e.g. pipes, consoles).
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn FlushFileBuffers(file: isize) -> WinBool {
    let handle = Handle::from_raw(file);
    common::file::flush_file_buffers(handle)
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
/// `WinBool::TRUE` on success, `WinBool::FALSE` on failure.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn CloseHandle(object: isize) -> WinBool {
    let handle = Handle::from_raw(object);

    rine_types::dev_notify!(on_handle_closed(object as i64));

    common::file::close_handle(handle)
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
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn SetFilePointer(
    file: isize,
    distance_to_move: i32,           // low 32 bits
    distance_to_move_high: *mut i32, // high 32 bits (in/out, optional)
    move_method: u32,
) -> u32 {
    let handle = Handle::from_raw(file);

    unsafe {
        common::file::set_file_pointer(handle, distance_to_move, distance_to_move_high, move_method)
    }
}
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
/// A search handle that can be used with `FindNextFile` and `FindClose`, or `INVALID_HANDLE_VALUE` if no
/// matching files were found or an error occurred.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn FindFirstFileA(
    file_name: *const u8,
    find_data: *mut Win32FindDataA,
) -> isize {
    if file_name.is_null() {
        return INVALID_HANDLE_VALUE.as_raw();
    }

    unsafe {
        let Some(path_str) = read_cstr(file_name) else {
            return INVALID_HANDLE_VALUE.as_raw();
        };

        common::file::find_first_file_a(&path_str, find_data).as_raw()
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
/// A search handle that can be used with `FindNextFile` and `FindClose`, or `INVALID_HANDLE_VALUE` if no
/// matching files were found or an error occurred.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn FindFirstFileW(
    file_name: *const u16,
    find_data: *mut Win32FindDataW,
) -> isize {
    if file_name.is_null() {
        return INVALID_HANDLE_VALUE.as_raw();
    }
    unsafe {
        let Some(path_str) = read_wstr(file_name) else {
            return INVALID_HANDLE_VALUE.as_raw();
        };

        common::file::find_first_file_w(&path_str, find_data).as_raw()
    }
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
/// `WinBool::TRUE` on success, `WinBool::FALSE` on failure.
///
/// # Note
/// This implementation does not set the error code and will currently always return `WinBool::TRUE` at the moment.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn FindClose(find_file: isize) -> WinBool {
    unsafe { CloseHandle(find_file) }
}
