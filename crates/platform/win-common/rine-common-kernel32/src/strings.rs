use rine_types::errors::BOOL;
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
pub fn lstrcata(lpstring1: LPSTR, lpstring2: LPCSTR) -> LPSTR {
    if lpstring1.is_null() {
        return LPSTR::NULL;
    }

    if lpstring2.is_null() {
        return lpstring1;
    }

    unsafe {
        libc::strcat(
            lpstring1.as_mut_ptr() as *mut c_char,
            lpstring2.as_ptr() as *const c_char,
        );
    }

    lpstring1
}

/// Copies up to a specified number of characters from a source string to a destination buffer, ensuring null termination.
///
/// # Arguments
/// * `llpstring1` - A pointer to the destination buffer.
///   Must be large enough to hold at least `imaxlength` bytes, including the null terminator.
///   Behavior is undefined if the buffer is too small.
///   Can be null, in which case the function does nothing and returns null.
/// * `lpstring2` - A pointer to the source null-terminated string.
///   Can be null, in which case the destination buffer will be set to an empty string.
///
/// # Safety
/// If a non-null pointer is passed for either argument, it must point to a valid null-terminated string, or the behavior is undefined.
/// The `lstrcpyna` function has an undefined behavior if source and destination buffers overlap,
/// so the caller must ensure that the buffers do not overlap.
///
/// # Returns
/// A pointer to the destination buffer, or null if `llpstring1` is null.
/// If `lpstring2` is null, the destination buffer will be set to an empty string and returned.
pub fn lstrcpyna(llpstring1: LPSTR, lpstring2: LPCSTR, imaxlength: u32) -> LPSTR {
    if llpstring1.is_null() {
        return LPSTR::NULL;
    }

    if lpstring2.is_null() {
        unsafe {
            *llpstring1.as_mut_ptr() = 0;
        }
        return llpstring1;
    }

    unsafe {
        libc::strncpy(
            llpstring1.as_mut_ptr() as *mut c_char,
            lpstring2.as_ptr() as *const c_char,
            imaxlength as usize,
        );
        // Ensure null termination
        *llpstring1.as_mut_ptr().add(imaxlength as usize - 1) = 0;
    }

    llpstring1
}

/// Determine whether a byte is a DBCS lead byte for the given code page.
///
/// # Arguments
/// * `code_page` - The code page identifier to query.
///   `CP_ACP` and `CP_THREAD_ACP` are resolved from the registry ANSI code page.
///   `CP_OEMCP` is resolved from the registry OEM code page.
/// * `test_char` - The byte value to test.
///
/// # Safety
/// This function does not dereference pointers or access caller-provided memory.
///
/// # Returns
/// `BOOL::TRUE` when `test_char` is a lead byte for the resolved code page;
/// otherwise `BOOL::FALSE`.
///
/// # Notes
/// This is a partial implementation that currently uses a small table of known
/// DBCS code pages and lead-byte ranges.
///
/// Unfinished behavior:
/// - It does not query OS `CPINFO`/`CPINFOEX` data, so code pages outside the
///   built-in table always return `FALSE`.
/// - It does not set thread last-error for unsupported or invalid code pages.
pub fn is_dbcs_lead_byte_ex(code_page: u32, test_char: u8) -> BOOL {
    BOOL::from(dbcs::is_dbcs_lead_byte_ex(code_page, test_char))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── lstrlena ──────────────────────────────────────────────────

    #[test]
    fn lstrlena_normal() {
        let bytes = b"hello\0";
        let s: LPCSTR = unsafe { std::mem::transmute(bytes.as_ptr()) };
        assert_eq!(lstrlena(s), 5);
    }

    #[test]
    fn lstrlena_empty() {
        let bytes = b"\0";
        let s: LPCSTR = unsafe { std::mem::transmute(bytes.as_ptr()) };
        assert_eq!(lstrlena(s), 0);
    }

    #[test]
    fn lstrlena_null() {
        assert_eq!(lstrlena(LPCSTR::NULL), 0);
    }

    // ── lstrcmpa ──────────────────────────────────────────────────

    #[test]
    fn lstrcmpa_equal() {
        let a = b"hello\0";
        let b = b"hello\0";
        let sa: LPCSTR = unsafe { std::mem::transmute(a.as_ptr()) };
        let sb: LPCSTR = unsafe { std::mem::transmute(b.as_ptr()) };
        assert_eq!(lstrcmpa(sa, sb), 0);
    }

    #[test]
    fn lstrcmpa_less_than() {
        let a = b"apple\0";
        let b = b"banana\0";
        let sa: LPCSTR = unsafe { std::mem::transmute(a.as_ptr()) };
        let sb: LPCSTR = unsafe { std::mem::transmute(b.as_ptr()) };
        assert!(lstrcmpa(sa, sb) < 0);
    }

    #[test]
    fn lstrcmpa_greater_than() {
        let a = b"zebra\0";
        let b = b"banana\0";
        let sa: LPCSTR = unsafe { std::mem::transmute(a.as_ptr()) };
        let sb: LPCSTR = unsafe { std::mem::transmute(b.as_ptr()) };
        assert!(lstrcmpa(sa, sb) > 0);
    }

    #[test]
    fn lstrcmpa_both_null() {
        assert_eq!(lstrcmpa(LPCSTR::NULL, LPCSTR::NULL), 0);
    }

    #[test]
    fn is_dbcs_lead_byte_ex_shift_jis_lead_returns_true() {
        assert!(is_dbcs_lead_byte_ex(932, 0x81).is_true());
    }

    #[test]
    fn is_dbcs_lead_byte_ex_shift_jis_trailing_returns_false() {
        assert!(is_dbcs_lead_byte_ex(932, 0x40).is_false());
    }

    #[test]
    fn is_dbcs_lead_byte_ex_single_byte_code_page_returns_false() {
        assert!(is_dbcs_lead_byte_ex(1252, 0x81).is_false());
    }

    #[test]
    fn is_dbcs_lead_byte_ex_utf8_returns_false() {
        assert!(is_dbcs_lead_byte_ex(65001, 0xE2).is_false());
    }

    #[test]
    fn is_dbcs_lead_byte_ex_acp_defaults_to_non_dbcs() {
        assert!(is_dbcs_lead_byte_ex(dbcs::CP_ACP, 0x81).is_false());
    }

    #[test]
    fn is_dbcs_lead_byte_ex_thread_acp_defaults_to_non_dbcs() {
        assert!(is_dbcs_lead_byte_ex(dbcs::CP_THREAD_ACP, 0x81).is_false());
    }

    #[test]
    fn lstrcmpa_first_null() {
        let b = b"hello\0";
        let sb: LPCSTR = unsafe { std::mem::transmute(b.as_ptr()) };
        assert_eq!(lstrcmpa(LPCSTR::NULL, sb), -1);
    }

    #[test]
    fn lstrcmpa_second_null() {
        let a = b"hello\0";
        let sa: LPCSTR = unsafe { std::mem::transmute(a.as_ptr()) };
        assert_eq!(lstrcmpa(sa, LPCSTR::NULL), 1);
    }

    // ── lstrcmpia ─────────────────────────────────────────────────

    #[test]
    fn lstrcmpia_equal_case_insensitive() {
        let a = b"Hello\0";
        let b = b"hello\0";
        let sa: LPCSTR = unsafe { std::mem::transmute(a.as_ptr()) };
        let sb: LPCSTR = unsafe { std::mem::transmute(b.as_ptr()) };
        assert_eq!(lstrcmpia(sa, sb), 0);
    }

    #[test]
    fn lstrcmpia_different() {
        let a = b"hello\0";
        let b = b"world\0";
        let sa: LPCSTR = unsafe { std::mem::transmute(a.as_ptr()) };
        let sb: LPCSTR = unsafe { std::mem::transmute(b.as_ptr()) };
        assert!(lstrcmpia(sa, sb) != 0);
    }

    #[test]
    fn lstrcmpia_both_null() {
        assert_eq!(lstrcmpia(LPCSTR::NULL, LPCSTR::NULL), 0);
    }

    #[test]
    fn lstrcmpia_first_null() {
        let b = b"hello\0";
        let sb: LPCSTR = unsafe { std::mem::transmute(b.as_ptr()) };
        assert_eq!(lstrcmpia(LPCSTR::NULL, sb), -1);
    }

    #[test]
    fn lstrcmpia_second_null() {
        let a = b"hello\0";
        let sa: LPCSTR = unsafe { std::mem::transmute(a.as_ptr()) };
        assert_eq!(lstrcmpia(sa, LPCSTR::NULL), 1);
    }

    // ── lstrcpya ──────────────────────────────────────────────────

    #[test]
    fn lstrcpya_copies_string() {
        let src = b"hello\0";
        let mut dst = [0u8; 16];
        let lp_src: LPCSTR = unsafe { std::mem::transmute(src.as_ptr()) };
        let lp_dst: LPSTR = unsafe { std::mem::transmute(dst.as_mut_ptr()) };
        let result = lstrcpya(lp_dst, lp_src);
        assert!(!result.is_null());
        assert_eq!(&dst[..6], b"hello\0");
    }

    #[test]
    fn lstrcpya_null_dest() {
        let src = b"hello\0";
        let lp_src: LPCSTR = unsafe { std::mem::transmute(src.as_ptr()) };
        assert!(lstrcpya(LPSTR::NULL, lp_src).is_null());
    }

    #[test]
    fn lstrcpya_null_src() {
        let mut dst = [0u8; 16];
        dst[0] = 0xFF;
        let lp_dst: LPSTR = unsafe { std::mem::transmute(dst.as_mut_ptr()) };
        let result = lstrcpya(lp_dst, LPCSTR::NULL);
        assert!(result.is_null());
        assert_eq!(dst[0], 0xFF);
    }

    // ── lstrcata ──────────────────────────────────────────────────

    #[test]
    fn lstrcata_concatenates() {
        let mut buf = [0u8; 32];
        buf[..6].copy_from_slice(b"hello\0");
        let append = b" world\0";
        let lp_buf: LPSTR = unsafe { std::mem::transmute(buf.as_mut_ptr()) };
        let lp_append: LPCSTR = unsafe { std::mem::transmute(append.as_ptr()) };
        let result = lstrcata(lp_buf, lp_append);
        assert!(!result.is_null());
        let expected = b"hello world\0";
        assert_eq!(&buf[..expected.len()], expected);
    }

    #[test]
    fn lstrcata_null_dest() {
        let append = b" world\0";
        let lp_append: LPCSTR = unsafe { std::mem::transmute(append.as_ptr()) };
        assert!(lstrcata(LPSTR::NULL, lp_append).is_null());
    }

    #[test]
    fn lstrcata_null_src() {
        let mut buf = b"hello\0\0\0\0\0\0".to_owned();
        let lp_buf: LPSTR = unsafe { std::mem::transmute(buf.as_mut_ptr()) };
        let result = lstrcata(lp_buf, LPCSTR::NULL);
        assert!(!result.is_null());
        assert_eq!(&buf[..6], b"hello\0");
    }

    // ── lstrcpyna ─────────────────────────────────────────────────

    #[test]
    fn lstrcpyna_copies_exact() {
        let src = b"hello\0";
        let mut dst = [0u8; 16];
        let lp_src: LPCSTR = unsafe { std::mem::transmute(src.as_ptr()) };
        let lp_dst: LPSTR = unsafe { std::mem::transmute(dst.as_mut_ptr()) };
        let result = lstrcpyna(lp_dst, lp_src, 6);
        assert!(!result.is_null());
        assert_eq!(&dst[..6], b"hello\0");
    }

    #[test]
    fn lstrcpyna_truncates() {
        let src = b"helloworld\0";
        let mut dst = [0u8; 6];
        let lp_src: LPCSTR = unsafe { std::mem::transmute(src.as_ptr()) };
        let lp_dst: LPSTR = unsafe { std::mem::transmute(dst.as_mut_ptr()) };
        let result = lstrcpyna(lp_dst, lp_src, 6);
        assert!(!result.is_null());
        assert_eq!(&dst[..6], b"hello\0");
    }

    #[test]
    fn lstrcpyna_null_dest() {
        let src = b"hello\0";
        let lp_src: LPCSTR = unsafe { std::mem::transmute(src.as_ptr()) };
        assert!(lstrcpyna(LPSTR::NULL, lp_src, 6).is_null());
    }

    #[test]
    fn lstrcpyna_null_src() {
        let mut dst = [0xFFu8; 16];
        let lp_dst: LPSTR = unsafe { std::mem::transmute(dst.as_mut_ptr()) };
        let result = lstrcpyna(lp_dst, LPCSTR::NULL, 6);
        assert!(!result.is_null());
        assert_eq!(dst[0], 0);
    }
}
