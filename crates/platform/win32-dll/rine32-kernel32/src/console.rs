use rine_types::errors::WinBool;
use rine_types::handles::{Handle, INVALID_HANDLE_VALUE, handle_to_fd, std_handle_to_fd};

#[allow(non_snake_case, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetStdHandle(nstd_handle: u32) -> isize {
    match std_handle_to_fd(nstd_handle) {
        Some(fd) => (fd as isize) + 0x1000,
        None => INVALID_HANDLE_VALUE.as_raw(),
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn WriteConsoleA(
    console_output: isize,
    buffer: *const u8,
    chars_to_write: u32,
    chars_written: *mut u32,
    _reserved: *const core::ffi::c_void,
) -> WinBool {
    let handle = Handle::from_raw(console_output);
    let Some(fd) = handle_to_fd(handle) else {
        return WinBool::FALSE;
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

#[allow(non_snake_case, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn WriteConsoleW(
    console_output: isize,
    buffer: *const u16,
    chars_to_write: u32,
    chars_written: *mut u32,
    _reserved: *const core::ffi::c_void,
) -> WinBool {
    let handle = Handle::from_raw(console_output);
    let Some(fd) = handle_to_fd(handle) else {
        return WinBool::FALSE;
    };

    let wide_slice = unsafe { core::slice::from_raw_parts(buffer, chars_to_write as usize) };
    let utf8: String = String::from_utf16_lossy(wide_slice);
    let written = unsafe { libc::write(fd, utf8.as_ptr().cast(), utf8.len()) };

    if written < 0 {
        return WinBool::FALSE;
    }
    if !chars_written.is_null() {
        unsafe { *chars_written = chars_to_write };
    }
    WinBool::TRUE
}
