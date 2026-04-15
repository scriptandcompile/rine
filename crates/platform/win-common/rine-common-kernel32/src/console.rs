use rine_types::errors::WinBool;
use rine_types::handles::{Handle, handle_to_fd};

/// Write a string to a console handle.
///
/// # Arguments
/// * `handle`: A handle to the console output device. This should be a valid file descriptor corresponding to a console handle in this runtime.
/// * `text`: The string to be written to the console.
/// * `chars_written`: An optional pointer to a `u32` variable that receives the number of bytes actually written.
///   If this parameter is null, the function does not return this information.
///
/// # Safety
/// `handle` must be a valid file descriptor corresponding to a console handle in this runtime,
/// `chars_written` may be null; if non-null it must be a writable `*mut u32`.
#[unsafe(no_mangle)]
pub unsafe fn write_console(handle: Handle, text: &str, chars_written: *mut u32) -> WinBool {
    let Some(fd) = handle_to_fd(handle) else {
        return WinBool::FALSE;
    };

    unsafe {
        let utf = text.as_bytes();

        let written = libc::write(fd, utf.as_ptr().cast(), utf.len());

        if written < 0 {
            return WinBool::FALSE;
        }

        if !chars_written.is_null() {
            *chars_written = written as u32;
        }

        WinBool::TRUE
    }
}
