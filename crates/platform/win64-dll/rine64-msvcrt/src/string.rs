//! MSVCRT C string functions: strlen, strncmp, etc.
//!
//! Forwards to the host libc.

use core::ffi::c_char;

/// strlen — return the length of a null-terminated string.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "win64" fn strlen(s: *const c_char) -> usize {
    unsafe { libc::strlen(s) }
}

/// strncmp — compare at most n characters of two strings.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "win64" fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> i32 {
    unsafe { libc::strncmp(s1, s2, n) }
}

/// strcmp — compare two null-terminated strings.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "win64" fn strcmp(s1: *const c_char, s2: *const c_char) -> i32 {
    unsafe { libc::strcmp(s1, s2) }
}
