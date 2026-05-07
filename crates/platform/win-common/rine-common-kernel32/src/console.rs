use rine_types::errors::BOOL;
use rine_types::handles::{Handle, fd_to_handle, handle_to_fd, std_handle_to_fd};

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
/// If the specified standard handle is not available, the function returns `Handle::INVALID`.
#[unsafe(no_mangle)]
pub unsafe fn get_std_handle(nstd_handle: u32) -> Handle {
    match std_handle_to_fd(nstd_handle) {
        Some(fd) => fd_to_handle(fd),
        None => {
            tracing::warn!(nstd_handle, "GetStdHandle: unknown handle constant");
            Handle::INVALID
        }
    }
}

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
pub unsafe fn write_console(handle: Handle, text: &str, chars_written: *mut u32) -> BOOL {
    let Some(fd) = handle_to_fd(handle) else {
        return BOOL::FALSE;
    };

    unsafe {
        let utf = text.as_bytes();

        let written = libc::write(fd, utf.as_ptr().cast(), utf.len());

        if written < 0 {
            return BOOL::FALSE;
        }

        if !chars_written.is_null() {
            *chars_written = written as u32;
        }

        BOOL::TRUE
    }
}
