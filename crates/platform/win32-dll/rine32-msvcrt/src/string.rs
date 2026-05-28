//! MSVCRT C string and memory functions: strlen, strcmp, memcpy, memset.

use core::ffi::c_char;

use rine_common_msvcrt as common;

/// Get the length of a null-terminated string.
///
/// # Arguments
/// - `s`: A pointer to a null-terminated string. May be null, in which case this function returns 0.
///
/// # Safety
/// - The caller must ensure that `s` is either null or points to a valid null-terminated string.
///   If `s` is non-null and not properly null-terminated, this function may read out of bounds, leading to undefined behavior.
///
/// # Returns
/// - The length of the string pointed to by `s`, excluding the null terminator.
#[rine_dlls::implemented]
pub unsafe extern "C" fn strlen(s: *const c_char) -> usize {
    unsafe { common::strlen(s) }
}

/// Compare at most n characters of two strings.
///
/// # Arguments
/// - `s1`: A pointer to the first null-terminated string. May be null, in which case it is treated as an empty string.
/// - `s2`: A pointer to the second null-terminated string. May be null, in which case it is treated as an empty string.
/// - `n`: The maximum number of characters to compare.
///
/// # Safety
/// - The caller must ensure that `s1` and `s2` are either null or point to valid null-terminated strings.
///   If either pointer is non-null and not properly null-terminated, this function may read out of bounds,
///   leading to undefined behavior.
/// - The caller must ensure that `n` does not exceed the length of either string if they are non-null,
///   to avoid reading out of bounds.
///
/// # Returns
/// - An integer less than, equal to, or greater than zero if `s1` is found, respectively, to be less than, to match,
///   or be greater than `s2` when comparing at most `n` characters.
#[rine_dlls::implemented]
pub unsafe extern "C" fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> i32 {
    unsafe { common::strncmp(s1, s2, n) }
}

/// Compare two null-terminated strings.
///
/// # Arguments
/// - `s1`: A pointer to the first null-terminated string. May be null, in which case it is treated as an empty string.
/// - `s2`: A pointer to the second null-terminated string. May be null, in which case it is treated as an empty string.
///
/// # Safety
/// - The caller must ensure that `s1` and `s2` are either null or point to valid null-terminated strings.
///   If either pointer is non-null and not properly null-terminated, this function may read out of bounds,
///   leading to undefined behavior.
///
/// # Returns
/// - An integer less than, equal to, or greater than zero if `s1` is found, respectively, to be less than, to match,
///   or be greater than `s2`.
#[rine_dlls::implemented]
pub unsafe extern "C" fn strcmp(s1: *const c_char, s2: *const c_char) -> i32 {
    unsafe { common::strcmp(s1, s2) }
}

/// Return a string describing the error code.
///
/// # Arguments
/// - `errnum`: The error number to describe. Standard errno values (e.g., `EINVAL`, `ENOENT`) are
///   recognized and will be mapped to a corresponding human-readable message. Unknown values produce
///   a fallback string containing the numeric code.
///
/// # Returns
/// - A pointer to a thread-local null-terminated C string containing a human-readable error message.
///   The pointer remains valid until the next call to `strerror` in the same thread.
///   The caller should copy the string if it needs to persist across subsequent calls.
///
/// # Safety
/// - The returned pointer points to a thread-local buffer that is overwritten on each call to
///   `strerror` in the same thread. The caller must copy the string if it needs to outlive the
///   next call. The pointer is also invalidated when the thread exits.
///
/// # Notes
/// - This implementation uses the host system's `libc::strerror_r` to obtain error messages.
///   The messages are those of the host C library, not MSVC-specific messages. MSVC-compatible
///   error strings are not yet implemented.
/// - A secure, bounds-checked variant `strerror_s` is not yet implemented.
#[rine_dlls::partial]
pub unsafe extern "C" fn strerror(errnum: i32) -> *mut c_char {
    unsafe { common::strerror(errnum) }
}

/// Get the length of a null-terminated wide string.
///
/// # Arguments
/// - `s`: A pointer to a null-terminated wide (UTF-16) string. May be null, in which case this function returns 0.
///
/// # Safety
/// - The caller must ensure that `s` is either null or points to a valid null-terminated wide string.
///   If `s` is non-null and not properly null-terminated, this function may read out of bounds,
///   leading to undefined behavior.
///
/// # Returns
/// - The length of the wide string pointed to by `s`, excluding the null terminator,
///   measured in wide characters (u16 code units).
#[rine_dlls::implemented]
pub unsafe extern "C" fn wcslen(s: *const u16) -> usize {
    unsafe { common::wcslen(s) }
}
