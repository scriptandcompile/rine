use rine_types::errors::BOOL;
use rine_types::strings::{LPCSTR, LPSTR, LPWSTR};

use core::ffi::c_char;
use libc::strlen;

// ── Code-page constants ──────────────────────────────────────────
const CP_UTF8: u32 = 65001;

// ── MultiByteToWideChar flag constants ───────────────────────────
const MB_PRECOMPOSED: u32 = 0x0001;
const MB_COMPOSITE: u32 = 0x0002;
const MB_USEGLYPHCHARS: u32 = 0x0004;
const MB_ERR_INVALID_CHARS: u32 = 0x0008;

// ── Error codes not yet defined in rine-types ────────────────────
const ERROR_INSUFFICIENT_BUFFER: u32 = 122;
const ERROR_NO_UNICODE_TRANSLATION: u32 = 1113;

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

// ── Internal helpers ─────────────────────────────────────────────

/// Decode UTF-8 bytes to UTF-16 code units.
///
/// When `err_on_invalid` is `false` the conversion is lossy (invalid
/// sequences are replaced with U+FFFD).  When `true` the function
/// fails on the first invalid byte sequence.
fn convert_utf8_to_utf16(bytes: &[u8], err_on_invalid: bool) -> (Vec<u16>, bool) {
    if err_on_invalid {
        match core::str::from_utf8(bytes) {
            Ok(s) => (s.encode_utf16().collect(), false),
            Err(_) => (Vec::new(), true),
        }
    } else {
        (
            String::from_utf8_lossy(bytes).encode_utf16().collect(),
            false,
        )
    }
}

/// Convert an ANSI / OEM / DBCS byte slice to UTF-16.
///
/// # Notes
/// This is a **stub** implementation.  For code pages other than
/// CP_UTF8 each byte is mapped directly to a `u16` with the same
/// numeric value (i.e. ISO-8859-1 / Latin‑1 semantics).  This is
/// correct for ASCII (0x00‑0x7F) but incorrect for:
///
/// * Windows‑1252 mappings in the 0x80‑0x9F range,
/// * OEM code page glyphs,
/// * DBCS / Shift‑JIS / Big5 multi‑byte sequences.
fn convert_code_page_to_utf16(
    bytes: &[u8],
    _code_page: u32,
    _err_on_invalid: bool,
) -> (Vec<u16>, bool) {
    let utf16: Vec<u16> = bytes.iter().map(|&b| b as u16).collect();
    (utf16, false)
}

// ── MultiByteToWideChar ──────────────────────────────────────────

/// Convert a multi-byte (ANSI) string to its UTF-16LE (wide)
/// representation, implementing the Windows
/// [`MultiByteToWideChar`](https://learn.microsoft.com/en-us/windows/win32/api/stringapiset/nf-stringapiset-multibytetowidechar)
/// function.
///
/// # Arguments
/// * `code_page` - Code page used to interpret the source bytes.
///   `CP_ACP` (0), `CP_OEMCP` (1) and `CP_THREAD_ACP` (3) are
///   resolved from the emulated registry with Windows-default
///   fallbacks.
/// * `dw_flags` - A bitwise combination of `MB_PRECOMPOSED` (0x01),
///   `MB_COMPOSITE` (0x02), `MB_USEGLYPHCHARS` (0x04) and
///   `MB_ERR_INVALID_CHARS` (0x08).  Unrecognised flag bits cause
///   the function to fail with `ERROR_INVALID_FLAGS`.
/// * `lp_multi_byte_str` - Pointer to the source multi-byte string.
/// * `cb_multi_byte` - Size of the source buffer in bytes, or `-1`
///   if the source is null-terminated.
/// * `lp_wide_char_str` - Pointer to the destination wide-character
///   buffer.  May be `NULL` when `cch_wide_char` is `0` (query
///   size mode).
/// * `cch_wide_char` - Size of the destination buffer in wide
///   characters (`u16` units).  Pass `0` to query the required
///   buffer size without writing.
///
/// # Safety
/// * `lp_multi_byte_str` must be non-null and point to a readable
///   buffer of at least `cb_multi_byte` bytes (or a valid
///   null-terminated byte string when `cb_multi_byte == -1`).
/// * When `cch_wide_char > 0`, `lp_wide_char_str` must be non-null
///   and point to a writable buffer of at least `cch_wide_char`
///   `u16` elements.  The buffer may overlap with the source buffer;
///   this implementation copies through an intermediate `Vec` so it
///   is safe even with overlapping buffers.
///
/// # Returns
/// * On success: the number of wide characters written to
///   `lp_wide_char_str`, **excluding** any terminating null
///   character.
/// * When `cch_wide_char == 0`: the required buffer size in wide
///   characters (including the terminator when
///   `cb_multi_byte == -1`).
/// * On failure: `0`.  `SetLastError` is called with one of
///   `ERROR_INVALID_PARAMETER`, `ERROR_INSUFFICIENT_BUFFER`,
///   `ERROR_INVALID_FLAGS`, or `ERROR_NO_UNICODE_TRANSLATION`.
///
/// # Notes
/// **Partial implementation.**
///
/// Fully supported:
/// * `CP_UTF8` (65001) — correct UTF-8 → UTF-16 conversion with
///   `MB_ERR_INVALID_CHARS` error detection.
///
/// Stub behaviour for all other code pages:
/// * Each byte is cast directly to a `u16` (Latin‑1 / ISO-8859-1
///   semantics).  This is correct for plain ASCII (0x00‑0x7F) but
///   wrong for Windows‑1252 0x80‑0x9F, OEM code page glyphs, and
///   every DBCS / CJK multi-byte encoding.
/// * `MB_ERR_INVALID_CHARS` has no effect for non-UTF-8 code pages.
/// * `MB_PRECOMPOSED`, `MB_COMPOSITE` and `MB_USEGLYPHCHARS` are
///   accepted but ignored; no Unicode normalisation is performed.
pub unsafe fn multi_byte_to_wide_char(
    code_page: u32,
    dw_flags: u32,
    lp_multi_byte_str: LPCSTR,
    cb_multi_byte: i32,
    lp_wide_char_str: LPWSTR,
    cch_wide_char: i32,
) -> i32 {
    // ── Parameter validation ─────────────────────────────────────
    if lp_multi_byte_str.is_null() {
        crate::process::set_last_error(rine_types::errors::ERROR_INVALID_PARAMETER);
        return 0;
    }

    if cb_multi_byte == 0 {
        crate::process::set_last_error(rine_types::errors::ERROR_INVALID_PARAMETER);
        return 0;
    }

    if cch_wide_char < 0 {
        crate::process::set_last_error(rine_types::errors::ERROR_INVALID_PARAMETER);
        return 0;
    }

    if cch_wide_char > 0 && lp_wide_char_str.is_null() {
        crate::process::set_last_error(rine_types::errors::ERROR_INVALID_PARAMETER);
        return 0;
    }

    // Reject unrecognised flag bits.
    let known_flags: u32 = MB_PRECOMPOSED | MB_COMPOSITE | MB_USEGLYPHCHARS | MB_ERR_INVALID_CHARS;
    if dw_flags & !known_flags != 0 {
        crate::process::set_last_error(rine_types::errors::ERROR_INVALID_PARAMETER);
        return 0;
    }

    // ── Resolve ACP / OEMCP aliases ──────────────────────────────
    let resolved_cp = dbcs::resolve_code_page(code_page);

    // ── Obtain the source byte slice ─────────────────────────────
    let src_bytes: &[u8] = if cb_multi_byte == -1 {
        let len = lstrlena(lp_multi_byte_str) as usize;
        unsafe { core::slice::from_raw_parts(lp_multi_byte_str.as_ptr(), len) }
    } else {
        unsafe { core::slice::from_raw_parts(lp_multi_byte_str.as_ptr(), cb_multi_byte as usize) }
    };

    // ── Convert to UTF-16 ────────────────────────────────────────
    let err_on_invalid = (dw_flags & MB_ERR_INVALID_CHARS) != 0;

    let (wide_chars, had_errors) = if resolved_cp == CP_UTF8 {
        convert_utf8_to_utf16(src_bytes, err_on_invalid)
    } else {
        convert_code_page_to_utf16(src_bytes, resolved_cp, err_on_invalid)
    };

    if had_errors && err_on_invalid {
        crate::process::set_last_error(ERROR_NO_UNICODE_TRANSLATION);
        return 0;
    }

    // ── Determine the required size ──────────────────────────────
    // When cbMultiByte is -1 the null terminator counts toward the
    // required buffer size.
    let required = if cb_multi_byte == -1 {
        wide_chars.len() as i32 + 1
    } else {
        wide_chars.len() as i32
    };

    // Query-size mode — just return what is needed.
    if cch_wide_char == 0 {
        return required;
    }

    if cch_wide_char < required {
        crate::process::set_last_error(ERROR_INSUFFICIENT_BUFFER);
        return 0;
    }

    // ── Write to the output buffer ───────────────────────────────
    unsafe {
        core::ptr::copy_nonoverlapping(
            wide_chars.as_ptr(),
            lp_wide_char_str.as_mut_ptr(),
            wide_chars.len(),
        );
        if cb_multi_byte == -1 {
            *lp_wide_char_str.as_mut_ptr().add(wide_chars.len()) = 0;
        }
    }

    wide_chars.len() as i32
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

    // ── multi_byte_to_wide_char ─────────────────────────────────────

    #[test]
    fn multi_byte_to_wide_char_null_src_returns_zero() {
        unsafe {
            assert_eq!(
                multi_byte_to_wide_char(65001, 0, LPCSTR::NULL, -1, LPWSTR::NULL, 0),
                0
            );
        }
    }

    #[test]
    fn multi_byte_to_wide_char_cb_multi_byte_zero_returns_zero() {
        let bytes = b"hello\0";
        let lp_src: LPCSTR = unsafe { std::mem::transmute(bytes.as_ptr()) };
        unsafe {
            assert_eq!(
                multi_byte_to_wide_char(65001, 0, lp_src, 0, LPWSTR::NULL, 0),
                0
            );
        }
    }

    #[test]
    fn multi_byte_to_wide_char_utf8_query_size() {
        let bytes = b"hello\0";
        let lp_src: LPCSTR = unsafe { std::mem::transmute(bytes.as_ptr()) };
        unsafe {
            // Null-terminated: required includes terminator → 5 + 1 = 6
            let needed = multi_byte_to_wide_char(65001, 0, lp_src, -1, LPWSTR::NULL, 0);
            assert_eq!(needed, 6);
        }
    }

    #[test]
    fn multi_byte_to_wide_char_utf8_query_size_counted() {
        let bytes = b"hello";
        let lp_src: LPCSTR = unsafe { std::mem::transmute(bytes.as_ptr()) };
        unsafe {
            // Explicit length (no auto-terminator) → 5
            let needed = multi_byte_to_wide_char(65001, 0, lp_src, 5, LPWSTR::NULL, 0);
            assert_eq!(needed, 5);
        }
    }

    #[test]
    fn multi_byte_to_wide_char_utf8_roundtrip() {
        // "hello" followed by the UTF-8 encoding of U+2713 (✓)
        let bytes = b"hello\xE2\x9C\x93\0";
        let lp_src: LPCSTR = unsafe { std::mem::transmute(bytes.as_ptr()) };
        let mut wide_buf = [0u16; 16];
        let lp_dst: LPWSTR = unsafe { std::mem::transmute(wide_buf.as_mut_ptr()) };
        unsafe {
            let written = multi_byte_to_wide_char(65001, 0, lp_src, -1, lp_dst, 16);
            assert_eq!(written, 6); // "hello✓" → 6 UTF-16 code units
            assert_eq!(
                wide_buf[..7],
                [
                    'h' as u16, 'e' as u16, 'l' as u16, 'l' as u16, 'o' as u16, 0x2713, 0
                ]
            );
        }
    }

    #[test]
    fn multi_byte_to_wide_char_utf8_err_on_invalid_fails() {
        // 0xFF is never valid UTF-8
        let bytes = b"abc\xFF\0";
        let lp_src: LPCSTR = unsafe { std::mem::transmute(bytes.as_ptr()) };
        unsafe {
            let result =
                multi_byte_to_wide_char(65001, MB_ERR_INVALID_CHARS, lp_src, -1, LPWSTR::NULL, 0);
            assert_eq!(result, 0);
        }
    }

    #[test]
    fn multi_byte_to_wide_char_utf8_lossy_no_err_flag() {
        // 0xFF is invalid, but without MB_ERR_INVALID_CHARS it should
        // be replaced with U+FFFD.
        let bytes = b"abc\xFF\0";
        let lp_src: LPCSTR = unsafe { std::mem::transmute(bytes.as_ptr()) };
        let mut wide_buf = [0u16; 16];
        let lp_dst: LPWSTR = unsafe { std::mem::transmute(wide_buf.as_mut_ptr()) };
        unsafe {
            let written = multi_byte_to_wide_char(65001, 0, lp_src, -1, lp_dst, 16);
            assert_eq!(written, 4); // 'a', 'b', 'c', U+FFFD
            assert_eq!(wide_buf[3], 0xFFFD);
        }
    }

    #[test]
    fn multi_byte_to_wide_char_insufficient_buffer() {
        let bytes = b"hello\0";
        let lp_src: LPCSTR = unsafe { std::mem::transmute(bytes.as_ptr()) };
        let mut wide_buf = [0u16; 2]; // too small
        let lp_dst: LPWSTR = unsafe { std::mem::transmute(wide_buf.as_mut_ptr()) };
        unsafe {
            let result = multi_byte_to_wide_char(65001, 0, lp_src, -1, lp_dst, 2);
            assert_eq!(result, 0);
        }
    }

    #[test]
    fn multi_byte_to_wide_char_unknown_flag_returns_zero() {
        let bytes = b"hello\0";
        let lp_src: LPCSTR = unsafe { std::mem::transmute(bytes.as_ptr()) };
        unsafe {
            let result = multi_byte_to_wide_char(65001, 0x8000, lp_src, -1, LPWSTR::NULL, 0);
            assert_eq!(result, 0);
        }
    }

    #[test]
    fn multi_byte_to_wide_char_ansi_pass_through() {
        // ANSI code page: each byte maps to u16 with same value.
        let bytes = b"hello\0";
        let lp_src: LPCSTR = unsafe { std::mem::transmute(bytes.as_ptr()) };
        let mut wide_buf = [0u16; 16];
        let lp_dst: LPWSTR = unsafe { std::mem::transmute(wide_buf.as_mut_ptr()) };
        unsafe {
            let written = multi_byte_to_wide_char(0, 0, lp_src, -1, lp_dst, 16);
            assert_eq!(written, 5);
            assert_eq!(
                wide_buf[..6],
                [
                    b'h' as u16,
                    b'e' as u16,
                    b'l' as u16,
                    b'l' as u16,
                    b'o' as u16,
                    0
                ]
            );
        }
    }

    #[test]
    fn multi_byte_to_wide_char_cch_wide_char_negative() {
        let bytes = b"hello\0";
        let lp_src: LPCSTR = unsafe { std::mem::transmute(bytes.as_ptr()) };
        unsafe {
            assert_eq!(
                multi_byte_to_wide_char(65001, 0, lp_src, -1, LPWSTR::NULL, -1),
                0
            );
        }
    }
}
