//! MSVCRT C string functions: strlen, strncmp, etc.
//!
//! Forwards to the host libc.

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
pub unsafe extern "win64" fn strlen(s: *const c_char) -> usize {
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
pub unsafe extern "win64" fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> i32 {
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
pub unsafe extern "win64" fn strcmp(s1: *const c_char, s2: *const c_char) -> i32 {
    unsafe { common::strcmp(s1, s2) }
}
