//! kernel32 file I/O: WriteFile (minimal, targeting stdout/stderr for Phase 1).

use rine_types::errors::{self, WinBool};
use rine_types::handles::{handle_to_fd, Handle};

/// WriteFile — write data to a file or I/O device.
///
/// Minimal implementation: translates HANDLE → fd and calls `libc::write`.
/// Overlapped I/O is not supported.
///
/// # Safety
/// `buffer` must point to at least `bytes_to_write` readable bytes.
/// `bytes_written` may be null; if non-null must be writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn WriteFile(
    file: isize,                  // HANDLE
    buffer: *const u8,            // LPCVOID
    bytes_to_write: u32,          // DWORD
    bytes_written: *mut u32,      // LPDWORD (out, optional)
    _overlapped: *mut core::ffi::c_void, // LPOVERLAPPED (ignored)
) -> WinBool {
    let handle = Handle::from_raw(file);
    let Some(fd) = handle_to_fd(handle) else {
        return errors::FALSE;
    };

    let written = unsafe { libc::write(fd, buffer.cast(), bytes_to_write as usize) };

    if written < 0 {
        return errors::FALSE;
    }

    if !bytes_written.is_null() {
        unsafe { *bytes_written = written as u32 };
    }
    errors::TRUE
}
