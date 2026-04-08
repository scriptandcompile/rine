use rine_common_kernel32 as common;
use rine_dlls::win32_stub;
use rine_types::handles::{Handle, INVALID_HANDLE_VALUE};
use rine_types::{
    errors::WinBool,
    strings::{read_cstr, read_wstr},
};

win32_stub!(ReadFile, "kernel32");

/// CreateFileA — open or create a file (ANSI path).
///
/// # Safety
/// `file_name` must be a valid null-terminated ANSI string.
#[allow(non_snake_case)]
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
/// # Safety
/// `file_name` must be a valid null-terminated UTF-16LE string.
#[allow(non_snake_case)]
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
/// # Safety
/// `file_name` must be a valid null-terminated UTF-16LE string.
#[allow(non_snake_case)]
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
/// # Safety
/// `file_name` must be a valid null-terminated ANSI string.
#[allow(non_snake_case)]
pub unsafe extern "stdcall" fn DeleteFileA(file_name: *const u8) -> WinBool {
    if file_name.is_null() {
        return WinBool::FALSE;
    }

    let c_str = unsafe { read_cstr(file_name).unwrap_or_default() };
    let path_str = c_str.to_string();
    common::file::delete_file(&path_str)
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
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

#[allow(non_snake_case, clippy::missing_safety_doc)]
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
#[allow(non_snake_case, clippy::missing_safety_doc)]
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

/// FindClose — close a search handle opened by FindFirstFile.
///
/// # Arguments
/// * `find_file` - The search handle returned by `FindFirstFile`.
///
/// # Safety
/// * `find_file` must be a valid search handle returned by `FindFirstFile`.
/// * After this call, `find_file` must not be used again.
#[allow(non_snake_case)]
pub unsafe extern "stdcall" fn FindClose(find_file: isize) -> WinBool {
    unsafe { CloseHandle(find_file) }
}
