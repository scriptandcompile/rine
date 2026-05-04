use rine_common_kernel32::strings as common;
use rine_types::strings::{LPCSTR, LPSTR};

/// A safe wrapper around `strlen` that returns 0 for null pointers, matching the behavior of `lstrlenA` in the Windows API.
///
/// # Arguments
/// * `lpString` - A pointer to a null-terminated string. Can be null, in which case the function returns 0.
///
/// # Safety
/// This function is safe to call with a null pointer, as it checks for null before calling `strlen`.
/// However, if a non-null pointer is passed, it must point to a valid null-terminated string, or the behavior is undefined.
///
/// # Returns
/// The length of the string in bytes, not including the null terminator. Returns 0 if `lpString` is null.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn lstrlenA(lpString: LPCSTR) -> u32 {
    common::lstrlena(lpString)
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
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn lstrcmpa(lpstring1: LPCSTR, lpstring2: LPCSTR) -> i32 {
    common::lstrcmpa(lpstring1, lpstring2)
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
///
/// # Notes
/// This function does not currently correctly handle locale-specific string comparison rules, and simply performs a byte-wise comparison.
/// It is intended as a basic implementation of `lstrcmpA` for ASCII strings.
/// This function does not correctly handle 'word-sort' comparison rules such as ensuring that "coop" and "co-op" stay together.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn lstrcmpiA(lpString1: LPCSTR, lpString2: LPCSTR) -> i32 {
    common::lstrcmpia(lpString1, lpString2)
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
/// The lstrcpyA function has an undefined behavior if source and destination buffers overlap,
/// so the caller must ensure that the buffers do not overlap.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn lstrcpyA(lpString1: LPSTR, lpString2: LPCSTR) -> LPSTR {
    common::lstrcpya(lpString1, lpString2)
}
/// Concatenates two strings and stores the result in a buffer.
///
/// # Arguments
/// * `lpstring1` - A pointer to the destination buffer. Must be large enough to hold the resulting string and null terminator.
///   Behavior is undefined if the buffer is too small and may cause a buffer overflow.
///   Can be null, in which case the function does nothing and returns 0.
/// * `lpstring2` - A pointer to the source null-terminated string to append to the destination buffer.
///   Can be null, in which case the destination buffer is left unchanged.
///
/// # Safety
/// If a non-null pointer is passed for either argument, it must point to a valid null-terminated string, or the behavior is undefined.
/// The `lstrcata` function has an undefined behavior if source and destination buffers overlap,
/// so the caller must ensure that the buffers do not overlap.
///
/// # Returns
/// A pointer to the destination buffer, or null if `lpstring1` is null.
/// If `lpstring2` is null, the destination buffer is returned unchanged.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn lstrcatA(lpString1: LPSTR, lpString2: LPCSTR) -> LPSTR {
    common::lstrcata(lpString1, lpString2)
}
