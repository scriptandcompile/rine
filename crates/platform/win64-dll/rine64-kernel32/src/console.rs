//! kernel32 console I/O: GetStdHandle, WriteConsoleA, WriteConsoleW.

use rine_types::errors::{self, WinBool};
use rine_types::handles::{INVALID_HANDLE_VALUE, fd_to_handle, handle_to_fd, std_handle_to_fd};

/// GetStdHandle — return a HANDLE for stdin, stdout, or stderr.
///
/// `nstd_handle`: STD_INPUT_HANDLE (−10), STD_OUTPUT_HANDLE (−11),
///                STD_ERROR_HANDLE (−12).
///
/// # Safety
/// No pointer arguments; always safe at the ABI level.
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetStdHandle(nstd_handle: u32) -> isize {
    match std_handle_to_fd(nstd_handle) {
        Some(fd) => fd_to_handle(fd).as_raw(),
        None => {
            tracing::warn!(nstd_handle, "GetStdHandle: unknown handle constant");
            INVALID_HANDLE_VALUE.as_raw()
        }
    }
}

/// WriteConsoleA — write an ANSI (byte) string to a console handle.
///
/// # Safety
/// `buffer` must point to at least `chars_to_write` readable bytes.
/// `chars_written` may be null; if non-null it must be a writable `*mut u32`.
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn WriteConsoleA(
    console_output: isize,               // HANDLE
    buffer: *const u8,                   // const VOID*
    chars_to_write: u32,                 // DWORD (number of bytes)
    chars_written: *mut u32,             // LPDWORD (out, optional)
    _reserved: *const core::ffi::c_void, // must be NULL
) -> WinBool {
    let handle = rine_types::handles::Handle::from_raw(console_output);
    let Some(fd) = handle_to_fd(handle) else {
        return errors::WinBool::FALSE;
    };

    let written = unsafe { libc::write(fd, buffer.cast(), chars_to_write as usize) };

    if written < 0 {
        return WinBool::FALSE;
    }

    if !chars_written.is_null() {
        unsafe { *chars_written = written as u32 };
    }
    WinBool::TRUE
}

/// WriteConsoleW — write a wide (UTF-16LE) string to a console handle.
///
/// Converts to UTF-8 before writing to the Linux file descriptor, since
/// Linux terminals speak UTF-8.
///
/// # Safety
/// `buffer` must point to at least `chars_to_write` valid `u16` values.
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn WriteConsoleW(
    console_output: isize,
    buffer: *const u16,
    chars_to_write: u32,
    chars_written: *mut u32,
    _reserved: *const core::ffi::c_void,
) -> WinBool {
    let handle = rine_types::handles::Handle::from_raw(console_output);
    let Some(fd) = handle_to_fd(handle) else {
        return WinBool::FALSE;
    };

    // Build a &[u16] from the raw pointer and decode to UTF-8.
    let wide_slice = unsafe { core::slice::from_raw_parts(buffer, chars_to_write as usize) };
    let utf8: String = String::from_utf16_lossy(wide_slice);

    let written = unsafe { libc::write(fd, utf8.as_ptr().cast(), utf8.len()) };

    if written < 0 {
        return WinBool::FALSE;
    }

    // Report the number of *wide chars* consumed (all of them on success).
    if !chars_written.is_null() {
        unsafe { *chars_written = chars_to_write };
    }
    WinBool::TRUE
}
