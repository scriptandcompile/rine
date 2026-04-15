use rine_common_kernel32 as common;

use rine_types::errors::WinBool;
use rine_types::handles::{Handle, INVALID_HANDLE_VALUE, std_handle_to_fd};
use rine_types::strings::{read_cstr_counted, read_wstr_counted};

#[allow(non_snake_case, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetStdHandle(nstd_handle: u32) -> isize {
    match std_handle_to_fd(nstd_handle) {
        Some(fd) => (fd as isize) + 0x1000,
        None => INVALID_HANDLE_VALUE.as_raw(),
    }
}

/// WriteConsoleA — write an ANSI (byte) string to a console handle.
///
/// # Arguments
/// * `console_output` must be a valid console handle in this runtime, `buffer` must point to at least
///   `chars_to_write` readable bytes, and `chars_written` may be null; if non-null it must be a writable `*mut u32`.
/// * `buffer` must point to at least `chars_to_write` readable bytes, and `chars_written` may be null;
///   if non-null it must be a writable `*mut u32`.
/// * `chars_to_write` specifies the number of bytes to write from the buffer.
/// * `chars_written` is an optional pointer to a `u32` variable that receives the number of bytes actually written.
///   If this parameter is null, the function does not return this information.
/// * `_reserved` is reserved and must be set to null. It is included for compatibility with the Windows API but is
///   not used in this implementation.
///
/// # Safety
/// `console_output` must be a valid console handle in this runtime, `buffer` must
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn WriteConsoleA(
    console_output: isize,
    buffer: *const u8,
    chars_to_write: u32,
    chars_written: *mut u32,
    _reserved: *const core::ffi::c_void,
) -> WinBool {
    let handle = Handle::from_raw(console_output);

    unsafe {
        let Some(text) = read_cstr_counted(buffer, chars_to_write as i32) else {
            return WinBool::FALSE;
        };

        common::console::write_console(handle, &text, chars_written)
    }
}

/// WriteConsoleW — write a wide (UTF-16) string to a console handle.
///
/// # Arguments
/// * `console_output` must be a valid console handle in this runtime, `buffer` must point to at least
///   `chars_to_write` readable bytes, and `chars_written` may be null; if non-null it must be a writable `*mut u32`.
/// * `buffer` must point to at least `chars_to_write` readable bytes, and `chars_written` may be null;
///   if non-null it must be a writable `*mut u32`.
/// * `chars_to_write` specifies the number of bytes to write from the buffer.
/// * `chars_written` is an optional pointer to a `u32` variable that receives the number of bytes actually written.
///   If this parameter is null, the function does not return this information.
/// * `_reserved` is reserved and must be set to null. It is included for compatibility with the Windows API but is
///   not used in this implementation.
///
/// # Safety
/// `console_output` must be a valid console handle in this runtime, `buffer` must
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn WriteConsoleW(
    console_output: isize,
    buffer: *const u16,
    chars_to_write: u32,
    chars_written: *mut u32,
    _reserved: *const core::ffi::c_void,
) -> WinBool {
    let handle = Handle::from_raw(console_output);

    unsafe {
        let Some(text) = read_wstr_counted(buffer, chars_to_write as i32) else {
            return WinBool::FALSE;
        };

        common::console::write_console(handle, &text, chars_written)
    }
}
