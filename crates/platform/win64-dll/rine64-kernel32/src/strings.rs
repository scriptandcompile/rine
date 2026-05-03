use rine_common_kernel32::strings as common;
use rine_types::strings::LPCSTR;

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
pub unsafe extern "win64" fn lstrlenA(lpString: LPCSTR) -> u32 {
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
pub unsafe extern "win64" fn lstrcmpa(lpstring1: LPCSTR, lpstring2: LPCSTR) -> i32 {
    common::lstrcmpa(lpstring1, lpstring2)
}
