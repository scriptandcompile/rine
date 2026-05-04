use rine_types::strings::{LPCSTR, LPSTR};

use core::ffi::c_char;
use libc::strlen;

/// A safe wrapper around `strlen` that returns 0 for null pointers, matching the behavior of `lstrlenA` in the Windows API.
///
/// # Arguments
/// * `lpstring` - A pointer to a null-terminated string. Can be null, in which case the function returns 0.
///
/// # Safety
/// This function is safe to call with a null pointer, as it checks for null before calling `strlen`.
/// However, if a non-null pointer is passed, it must point to a valid null-terminated string, or the behavior is undefined.
///
/// # Returns
/// The length of the string in bytes, not including the null terminator. Returns 0 if `lpstring` is null.
pub fn lstrlena(lpstring: LPCSTR) -> u32 {
    if lpstring.is_null() {
        return 0;
    }

    unsafe { strlen(lpstring.as_ptr() as *const c_char) as u32 }
}

/// A safe wrapper around `strcmp` that handles null pointers by treating them as empty strings.
///
/// # Arguments
/// * `lpstring1` - A pointer to the first null-terminated string. Can be null, in which case it is treated as an empty string.
/// * `lpstring2` - A pointer to the second null-terminated string. Can be null, in which case it is treated as an empty string.
///
/// # Safety
/// If a non-null pointer is passed for either argument, it must point to a valid null-terminated string, or the behavior is undefined.
///
/// # Returns
/// An integer less than, equal to, or greater than zero if `lpstring1` is found, respectively, to be less than, to match,
/// or be greater than `lpstring2`.
/// Null pointers are treated as empty strings, so a null pointer will compare as less than any non-null string, and two null
/// pointers will compare as equal.
///
/// # Notes
/// This function does not currently correctly handle locale-specific string comparison rules, and simply performs a byte-wise comparison.
/// It is intended as a basic implementation of `lstrcmpA` for ASCII strings.
/// This function does not correctly handle 'word-sort' comparison rules such as ensuring that "coop" and "co-op" stay together.
pub fn lstrcmpa(lpstring1: LPCSTR, lpstring2: LPCSTR) -> i32 {
    if lpstring1.is_null() && lpstring2.is_null() {
        return 0;
    } else if lpstring1.is_null() {
        return -1;
    } else if lpstring2.is_null() {
        return 1;
    }

    unsafe {
        libc::strcmp(
            lpstring1.as_ptr() as *const c_char,
            lpstring2.as_ptr() as *const c_char,
        )
    }
}

/// A case-insensitive version of `lstrcmpa` that handles null pointers by treating them as empty strings.
///
/// # Arguments
/// * `lpstring1` - A pointer to the first null-terminated string. Can be null, in which case it is treated as an empty string.
/// * `lpstring2` - A pointer to the second null-terminated string. Can be null, in which case it is treated as an empty string.
///
/// # Safety
/// If a non-null pointer is passed for either argument, it must point to a valid null-terminated string, or the behavior is undefined.
///
/// # Returns
/// An integer less than, equal to, or greater than zero if `lpstring1` is found, respectively, to be less than, to match,
/// or be greater than `lpstring2`, ignoring ASCII case differences.
pub fn lstrcmpia(lpstring1: LPCSTR, lpstring2: LPCSTR) -> i32 {
    if lpstring1.is_null() && lpstring2.is_null() {
        return 0;
    } else if lpstring1.is_null() {
        return -1;
    } else if lpstring2.is_null() {
        return 1;
    }

    unsafe {
        libc::strcasecmp(
            lpstring1.as_ptr() as *const c_char,
            lpstring2.as_ptr() as *const c_char,
        )
    }
}

/// Copies a string to a buffer.
///
/// # Arguments
/// * `lpstring1` - A pointer to the destination buffer. Must be large enough to hold the source string and null terminator.
///   Behavior is undefined if the buffer is too small and may cause a buffer overflow.
///   Can be null, in which case the function does nothing and returns 0.
/// * `lpstring2` - A pointer to the source null-terminated string.
///   Can be null, in which case the destination buffer will be set to an empty string.
///
/// # Safety
/// If a non-null pointer is passed for either argument, it must point to a valid null-terminated string, or the behavior is undefined.
/// The lstrcpya function has an undefined behavior if source and destination buffers overlap,
/// so the caller must ensure that the buffers do not overlap.
pub fn lstrcpya(lpstring1: LPSTR, lpstring2: LPCSTR) -> LPSTR {
    if lpstring1.is_null() {
        return LPSTR::NULL;
    }

    if lpstring2.is_null() {
        return LPSTR::NULL;
    }

    unsafe {
        libc::strcpy(
            lpstring1.as_mut_ptr() as *mut c_char,
            lpstring2.as_ptr() as *const c_char,
        );
    }

    lpstring1
}
