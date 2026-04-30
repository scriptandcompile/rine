//! kernel32 console I/O: GetStdHandle, WriteConsoleA, WriteConsoleW.

use rine_common_kernel32 as common;

use rine_types::errors::WinBool;
use rine_types::handles::Handle;
use rine_types::strings::{read_cstr_counted, read_wstr_counted};

/// Get a standard handle (stdin, stdout, stderr) as a raw handle value.
///
/// # Arguments
/// * `nstd_handle`: STD_INPUT_HANDLE (−10), STD_OUTPUT_HANDLE (−11), STD_ERROR_HANDLE (−12).
///
/// # Safety
/// No pointer arguments; always safe at the ABI level.
///
/// # Returns
/// On success, returns a raw handle value corresponding to the requested standard handle.
/// If the specified standard handle is not available, the function returns INVALID_HANDLE_VALUE.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetStdHandle(nstd_handle: u32) -> Handle {
    unsafe { common::console::get_std_handle(nstd_handle) }
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
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn WriteConsoleA(
    console_output: Handle,
    buffer: *const u8,
    chars_to_write: u32,
    chars_written: *mut u32,
    _reserved: *const core::ffi::c_void,
) -> WinBool {
    let handle = console_output;

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
/// `console_output` must be a valid console handle in this runtime, `buffer` must point to at least `chars_to_write`
/// readable bytes, and `chars_written` may be null; if non-null it must be a writable `*mut u32`.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn WriteConsoleW(
    console_output: Handle,
    buffer: *const u16,
    chars_to_write: u32,
    chars_written: *mut u32,
    _reserved: *const core::ffi::c_void,
) -> WinBool {
    unsafe {
        let Some(text) = read_wstr_counted(buffer, chars_to_write as i32) else {
            return WinBool::FALSE;
        };

        common::console::write_console(console_output, &text, chars_written)
    }
}
