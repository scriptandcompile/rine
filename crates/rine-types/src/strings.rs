//! Windows string types and conversion helpers.

use core::fmt;

/// NT UNICODE_STRING — a counted (not null-terminated) wide-character string
/// used throughout the NT kernel API.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct UnicodeString {
    /// Length in **bytes** (not characters), excluding any trailing null.
    pub length: u16,
    /// Maximum length in **bytes** of the buffer.
    pub maximum_length: u16,
    /// Pointer to a UTF-16LE buffer.
    pub buffer: *mut u16,
}

impl UnicodeString {
    pub const fn empty() -> Self {
        Self {
            length: 0,
            maximum_length: 0,
            buffer: core::ptr::null_mut(),
        }
    }
}

impl fmt::Debug for UnicodeString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "UNICODE_STRING(len={}, max={}, buf={:?})",
            self.length, self.maximum_length, self.buffer,
        )
    }
}

// ---------------------------------------------------------------------------
// Null-terminated string readers
// ---------------------------------------------------------------------------

/// Read a null-terminated ANSI string from a raw pointer.
///
/// Returns `None` if `ptr` is null.
///
/// # Safety
/// `ptr` must be null or point to a valid null-terminated byte string.
pub unsafe fn read_cstr(ptr: *const u8) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    Some(
        unsafe { std::ffi::CStr::from_ptr(ptr.cast()) }
            .to_string_lossy()
            .into_owned(),
    )
}

/// Read a null-terminated UTF-16LE string from a raw pointer.
///
/// Returns `None` if `ptr` is null.
///
/// # Safety
/// `ptr` must be null or point to a valid null-terminated UTF-16 string.
pub unsafe fn read_wstr(ptr: *const u16) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    let mut len = 0;
    unsafe {
        while *ptr.add(len) != 0 {
            len += 1;
        }
    }
    let slice = unsafe { core::slice::from_raw_parts(ptr, len) };
    Some(String::from_utf16_lossy(slice))
}

// ---------------------------------------------------------------------------
// Win32 buffer writers
// ---------------------------------------------------------------------------

/// Write an ANSI string (with null terminator) into a caller-supplied buffer.
///
/// Returns the number of characters written (excluding the null terminator),
/// or the required buffer size (including the null terminator) if the buffer
/// is too small or null.
///
/// # Safety
/// `buf` must be null or point to at least `buf_size` writable bytes.
pub unsafe fn write_cstr(buf: *mut u8, buf_size: u32, value: &str) -> u32 {
    let needed = value.len() as u32 + 1; // +1 for null terminator
    if buf.is_null() || buf_size < needed {
        return needed;
    }
    unsafe {
        core::ptr::copy_nonoverlapping(value.as_ptr(), buf, value.len());
        *buf.add(value.len()) = 0;
    }
    value.len() as u32
}

/// Write a UTF-16LE string (with null terminator) into a caller-supplied buffer.
///
/// Returns the number of characters written (excluding the null terminator),
/// or the required buffer size (including the null terminator) if the buffer
/// is too small or null.
///
/// # Safety
/// `buf` must be null or point to at least `buf_size` writable u16 elements.
pub unsafe fn write_wstr(buf: *mut u16, buf_size: u32, value: &str) -> u32 {
    let encoded: Vec<u16> = value.encode_utf16().collect();
    let needed = encoded.len() as u32 + 1;
    if buf.is_null() || buf_size < needed {
        return needed;
    }
    unsafe {
        core::ptr::copy_nonoverlapping(encoded.as_ptr(), buf, encoded.len());
        *buf.add(encoded.len()) = 0;
    }
    encoded.len() as u32
}

/// Escape a string for safe embedding in JSON string values.
///
/// This escapes backslash, double quote, and common control characters.
pub fn json_escape(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── read_cstr ────────────────────────────────────────────────

    #[test]
    fn read_cstr_null_returns_none() {
        assert_eq!(unsafe { read_cstr(std::ptr::null()) }, None);
    }

    #[test]
    fn read_cstr_valid() {
        let s = b"hello\0";
        assert_eq!(unsafe { read_cstr(s.as_ptr()) }, Some("hello".into()));
    }

    // ── read_wstr ────────────────────────────────────────────────

    #[test]
    fn read_wstr_null_returns_none() {
        assert_eq!(unsafe { read_wstr(std::ptr::null()) }, None);
    }

    #[test]
    fn read_wstr_valid() {
        let s: Vec<u16> = "hello\0".encode_utf16().collect();
        assert_eq!(unsafe { read_wstr(s.as_ptr()) }, Some("hello".into()));
    }

    // ── write_cstr ───────────────────────────────────────────────

    #[test]
    fn write_cstr_fits() {
        let mut buf = [0u8; 16];
        let n = unsafe { write_cstr(buf.as_mut_ptr(), 16, "hello") };
        assert_eq!(n, 5);
        assert_eq!(&buf[..6], b"hello\0");
    }

    #[test]
    fn write_cstr_too_small() {
        let mut buf = [0u8; 4];
        let n = unsafe { write_cstr(buf.as_mut_ptr(), 4, "hello") };
        assert_eq!(n, 6); // required size including null
    }

    #[test]
    fn write_cstr_null_buf() {
        let n = unsafe { write_cstr(std::ptr::null_mut(), 0, "hello") };
        assert_eq!(n, 6);
    }

    // ── write_wstr ───────────────────────────────────────────────

    #[test]
    fn write_wstr_fits() {
        let mut buf = [0u16; 16];
        let n = unsafe { write_wstr(buf.as_mut_ptr(), 16, "hello") };
        assert_eq!(n, 5);
        let expected: Vec<u16> = "hello".encode_utf16().chain(std::iter::once(0)).collect();
        assert_eq!(&buf[..6], &expected[..]);
    }

    #[test]
    fn write_wstr_too_small() {
        let mut buf = [0u16; 4];
        let n = unsafe { write_wstr(buf.as_mut_ptr(), 4, "hello") };
        assert_eq!(n, 6);
    }

    #[test]
    fn write_wstr_null_buf() {
        let n = unsafe { write_wstr(std::ptr::null_mut(), 0, "hello") };
        assert_eq!(n, 6);
    }

    // ── json_escape ─────────────────────────────────────────────

    #[test]
    fn json_escape_escapes_specials() {
        let s = "a\\\"\n\r\tb";
        assert_eq!(json_escape(s), "a\\\\\\\"\\n\\r\\tb");
    }
}
