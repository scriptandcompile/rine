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
