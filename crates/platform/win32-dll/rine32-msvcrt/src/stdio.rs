//! MSVCRT stdio functions: printf, puts, fprintf, etc.
//!
//! `printf` remains a conservative stub for now, but other core output paths
//! (`puts`, `fwrite`, `fprintf`, `vfprintf`) are implemented to preserve
//! fixture stdout behavior in 32-bit mode.

use core::ffi::{c_char, c_int};

/// printf — print formatted output to stdout.
///
/// Stub implementation in Phase 1.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn printf(_format: *const c_char) -> c_int {
    tracing::trace!("msvcrt::printf (stub)");
    0
}

/// puts — print a string to stdout followed by a newline.
///
/// # Safety
/// `s` must be a valid null-terminated C string.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn puts(s: *const c_char) -> c_int {
    if s.is_null() {
        return libc::EOF;
    }
    tracing::trace!("msvcrt::puts");
    unsafe { libc::puts(s) }
}

/// fprintf — print formatted output to a file.
///
/// Minimal behavior: writes the format string bytes directly to the stream fd.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn fprintf(stream: *mut u8, format: *const c_char) -> c_int {
    if format.is_null() || stream.is_null() {
        return -1;
    }
    let fd = unsafe { *(stream as *const i32) };
    let len = unsafe { libc::strlen(format) };
    let written = unsafe { libc::write(fd, format.cast(), len) };
    if written < 0 { -1 } else { written as c_int }
}

/// vfprintf — print formatted output to a file using a va_list.
///
/// Minimal behavior: writes the format string bytes directly to the stream fd.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn vfprintf(stream: *mut u8, format: *const c_char, _args: *mut u8) -> c_int {
    if format.is_null() || stream.is_null() {
        return -1;
    }
    let fd = unsafe { *(stream as *const i32) };
    let len = unsafe { libc::strlen(format) };
    let written = unsafe { libc::write(fd, format.cast(), len) };
    if written < 0 { -1 } else { written as c_int }
}

/// fwrite — write data to a file.
///
/// Writes raw bytes to the stream fd marker stored in fake FILE structs.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn fwrite(
    ptr: *const core::ffi::c_void,
    size: usize,
    nmemb: usize,
    stream: *const core::ffi::c_void,
) -> usize {
    let total = size.saturating_mul(nmemb);
    if ptr.is_null() || stream.is_null() || total == 0 {
        return 0;
    }

    let fd = unsafe { *(stream as *const i32) };
    let written = unsafe { libc::write(fd, ptr, total) };
    if written < 0 {
        return 0;
    }
    (written as usize) / size.max(1)
}
