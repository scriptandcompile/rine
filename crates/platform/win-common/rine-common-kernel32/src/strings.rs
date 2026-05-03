use rine_types::strings::LPCSTR;

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
