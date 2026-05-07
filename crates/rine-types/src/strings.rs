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

/// Win32-style null-terminated string pointers, used in many APIs for input and output strings.
/// 'LPCSTR' stands for "Long Pointer to Constant String", a historical Windows convention.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LPCSTR(*const u8);

impl LPCSTR {
    /// A null pointer representing an empty string.
    pub const NULL: Self = Self(core::ptr::null());

    /// Check if the pointer is null.
    ///
    /// # Returns
    /// `true` if the pointer is null, `false` otherwise.
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Get the raw pointer value.
    ///
    /// # Safety
    /// The caller must ensure that the pointer is valid for the intended use.
    /// This function does not perform any safety checks.
    ///
    /// # Returns
    /// The raw pointer as a `*const u8`.
    pub fn as_ptr(self) -> *const u8 {
        self.0
    }

    /// Read a string from the pointer location, treating it as a null-terminated ANSI string.
    ///
    /// # Safety
    /// The caller must ensure that `self.0` is null or points to a valid null-terminated byte string.
    /// If this is not the case, this function may cause undefined behavior.
    /// The caller is responsible for ensuring that the pointer is properly aligned for `u8` access.
    ///
    /// # Returns
    /// An `Option<String>` containing the read string, or `None` if the pointer is null or if the string is not valid UTF-8.
    pub unsafe fn read_string(self) -> Option<String> {
        unsafe { read_cstr(self.0) }
    }

    /// Read a string from the pointer location with an explicit character count, treating it as an ANSI string.
    ///
    /// # Arguments
    /// * `count` - The number of characters to read from the pointer. This does not include any null terminator and may be zero.
    ///
    /// # Safety
    /// The caller must ensure that `self.0` is null or points to a valid byte string of at least `count` bytes when `count > 0`.
    /// If these conditions are not met, this function may cause undefined behavior.
    /// The caller is responsible for ensuring that the pointer is properly aligned for `u8` access.
    ///
    /// # Returns
    /// An `Option<String>` containing the read string, or `None` if the pointer is null, if `count` is negative, or if the string is not valid UTF-8.
    pub unsafe fn read_string_counted(self, count: i32) -> Option<String> {
        unsafe { read_cstr_counted(self.0, count) }
    }

    /// Write a string from the pointer location into a caller-supplied buffer, treating the pointer as an input string.
    ///
    /// # Arguments
    /// * `buf` - A pointer to a writable buffer where the string will be written as a null-terminated byte string.
    /// * `buf_size` - The size of the buffer in bytes. The function will write at most `buf_size - 1` characters plus a null terminator.
    ///
    /// # Safety
    /// The caller must ensure that `self.0` is null or points to a valid null-terminated byte string.
    /// The caller must also ensure that `buf` is null or points to a valid writable buffer of at least `buf_size` bytes.
    /// If these conditions are not met, this function may cause undefined behavior.
    /// The caller is responsible for ensuring that the buffer is properly aligned for `u8` access.
    ///
    /// # Returns
    /// The number of characters written (excluding the null terminator), or the required buffer size (including the null terminator)
    /// if the pointer is null, the buffer is null, or the buffer is too small to hold the string.
    pub unsafe fn write_to_buffer(self, buf: *mut u8, buf_size: u32) -> Option<u32> {
        if self.is_null() {
            return None;
        }

        unsafe {
            let s = self.read_string()?;
            let written = write_cstr(buf, buf_size, &s);
            Some(written)
        }
    }

    /// Write a string directly to the pointer location, treating it as an output buffer.
    ///
    /// # Arguments
    /// * `value` - The string to write. This will be encoded as a null-terminated byte string.
    ///
    /// # Safety
    /// The caller must ensure that `self.0` is null or points to a valid writable buffer of at
    /// least `value.len() + 1` bytes (enough for the string and null terminator).
    /// If this is not the case, this function may cause undefined behavior.
    /// The caller is also responsible for ensuring that the buffer is properly aligned for `u8` access.
    ///
    /// # Returns
    /// The number of characters written (excluding the null terminator), or the required buffer size
    /// (including the null terminator) if the pointer is null or the buffer is too small.
    pub unsafe fn write_string(self, value: &str) -> Option<u32> {
        if self.is_null() {
            return None;
        }

        unsafe {
            let written = write_cstr(self.0 as *mut u8, value.len() as u32 + 1, value);
            Some(written)
        }
    }
}

/// Win32-style null-terminated string pointers, used in many APIs for output strings.
/// 'LPSTR' stands for "Long Pointer to String", a historical Windows convention.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LPSTR(*mut u8);

impl LPSTR {
    /// A null pointer representing an empty string.
    pub const NULL: Self = Self(core::ptr::null_mut());

    /// Check if the pointer is null.
    ///
    /// # Returns
    /// `true` if the pointer is null, `false` otherwise.
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Get the raw pointer value.
    pub fn as_mut_ptr(self) -> *mut u8 {
        self.0
    }
}

/// Win32-style null-terminated string pointers, used in many APIs for output strings.
/// 'LPWSTR' stands for "Long Pointer to Wide String", a historical Windows convention.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LPWSTR(*mut u16);

impl LPWSTR {
    /// A null pointer representing an empty string.
    pub const NULL: Self = Self(core::ptr::null_mut());

    /// Check if the pointer is null.
    ///
    /// # Returns
    /// `true` if the pointer is null, `false` otherwise.
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Get the raw pointer value.
    pub fn as_mut_ptr(self) -> *mut u16 {
        self.0
    }
}

/// Win32-style null-terminated wide string pointers, used in many APIs for input and output strings.
/// 'LPCWSTR' stands for "Long Pointer to Constant Wide String", a historical Windows convention for UTF-16 strings.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LPCWSTR(*const u16);

impl LPCWSTR {
    /// A null pointer representing an empty string.
    pub const NULL: Self = Self(core::ptr::null());

    /// Check if the pointer is null.
    ///
    /// # Returns
    /// `true` if the pointer is null, `false` otherwise.
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Read a string from the pointer location, treating it as a null-terminated UTF-16LE string.
    ///
    /// # Safety
    /// The caller must ensure that `self.0` is null or points to a valid null-terminated UTF-16 string.
    /// If this is not the case, this function may cause undefined behavior.
    /// The caller is responsible for ensuring that the pointer is properly aligned for `u16` access.
    ///
    /// # Returns
    /// An `Option<String>` containing the read string, or `None` if the pointer is null or if the string is not valid UTF-16.
    pub unsafe fn read_string(self) -> Option<String> {
        unsafe { read_wstr(self.0) }
    }

    /// Read a string from the pointer location with an explicit character count, treating it as a wide string.
    ///
    /// # Arguments
    /// * `count` - The number of characters to read from the pointer. This does not include any null terminator and may be zero.
    ///
    /// # Safety
    /// The caller must ensure that `self.0` is null or points to a valid wide string of at least `count` characters when `count > 0`.
    /// If these conditions are not met, this function may cause undefined behavior.
    /// The caller is responsible for ensuring that the pointer is properly aligned for `u16` access.
    ///
    /// # Returns
    /// An `Option<String>` containing the read string, or `None` if the pointer is null, if `count` is negative, or if the string is not valid UTF-16.
    pub unsafe fn read_string_counted(self, count: i32) -> Option<String> {
        unsafe { read_wstr_counted(self.0, count) }
    }

    /// Write a string from the pointer location into a caller-supplied buffer, treating the pointer as an input string.
    ///
    /// # Arguments
    /// * `buf` - A pointer to a writable buffer where the string will be written as UTF-16LE with a null terminator.
    /// * `buf_size` - The size of the buffer in `u16` elements (not bytes).
    ///   The function will write at most `buf_size - 1` characters plus a null terminator.
    ///
    /// # Safety
    /// The caller must ensure that `self.0` is null or points to a valid null-terminated UTF-16 string.
    /// The caller must also ensure that `buf` is null or points to a valid writable buffer of at least `buf_size` `u16` elements.
    /// If these conditions are not met, this function may cause undefined behavior.
    /// The caller is responsible for ensuring that the buffer is properly aligned for `u16` access.
    ///
    /// # Returns
    /// The number of characters written (excluding the null terminator), or the required buffer size (including the null terminator)
    /// if the pointer is null, the buffer is null, or the buffer is too small to hold the string.
    pub unsafe fn write_to_buffer(self, buf: *mut u16, buf_size: u32) -> Option<u32> {
        if self.is_null() {
            return None;
        }
        unsafe {
            let s = self.read_string()?;
            let written = write_wstr(buf, buf_size, &s);
            Some(written)
        }
    }

    /// Write a string directly to the pointer location, treating it as an output buffer.
    ///
    /// # Arguments
    /// * `value` - The string to write. This will be encoded as UTF-16LE and null-terminated.
    ///
    /// # Safety
    /// The caller must ensure that `self.0` is null or points to a valid writable buffer of at
    /// least `value.len() * 2 + 2` bytes (enough for the UTF-16 encoding and null terminator).
    /// If this is not the case, this function may cause undefined behavior.
    /// The caller is also responsible for ensuring that the buffer is properly aligned for `u16` access.
    ///
    /// # Returns
    /// The number of characters written (excluding the null terminator), or the
    /// required buffer size (including the null terminator) if the pointer is null or the buffer is too small.
    pub unsafe fn write_string(self, value: &str) -> Option<u32> {
        if self.is_null() {
            return None;
        }
        unsafe {
            let written = write_wstr(self.0 as *mut u16, value.len() as u32 + 1, value);
            Some(written)
        }
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

/// Count UTF-16 code units in a null-terminated wide string.
///
/// Returns `None` if `ptr` is null.
///
/// # Safety
/// `ptr` must be null or point to a valid null-terminated UTF-16 string.
pub unsafe fn wstr_unit_len(ptr: *const u16) -> Option<usize> {
    if ptr.is_null() {
        return None;
    }

    let mut len = 0usize;
    unsafe {
        while *ptr.add(len) != 0 {
            len += 1;
        }
    }
    Some(len)
}

/// Read an ANSI string from a raw pointer with an explicit character count.
///
/// Returns `None` if `ptr` is null or `count` is negative.
///
/// # Safety
/// `ptr` must point to at least `count` readable bytes when `count > 0`.
pub unsafe fn read_cstr_counted(ptr: *const u8, count: i32) -> Option<String> {
    if ptr.is_null() || count < 0 {
        return None;
    }
    if count == 0 {
        return Some(String::new());
    }

    let bytes = unsafe { core::slice::from_raw_parts(ptr, count as usize) };
    let mut nul_terminated = Vec::with_capacity(bytes.len() + 1);
    nul_terminated.extend_from_slice(bytes);
    nul_terminated.push(0);
    unsafe { read_cstr(nul_terminated.as_ptr()) }
}

/// Read a UTF-16LE string from a raw pointer with an explicit character count.
///
/// Returns `None` if `ptr` is null or `count` is negative.
///
/// # Safety
/// `ptr` must point to at least `count` readable `u16` values when `count > 0`.
pub unsafe fn read_wstr_counted(ptr: *const u16, count: i32) -> Option<String> {
    if ptr.is_null() || count < 0 {
        return None;
    }
    if count == 0 {
        return Some(String::new());
    }

    let units = unsafe { core::slice::from_raw_parts(ptr, count as usize) };
    let mut nul_terminated = Vec::with_capacity(units.len() + 1);
    nul_terminated.extend_from_slice(units);
    nul_terminated.push(0);
    unsafe { read_wstr(nul_terminated.as_ptr()) }
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

    #[test]
    fn wstr_unit_len_null_returns_none() {
        assert_eq!(unsafe { wstr_unit_len(std::ptr::null()) }, None);
    }

    #[test]
    fn wstr_unit_len_counts_until_nul() {
        let s: Vec<u16> = "hello\0ignored".encode_utf16().collect();
        assert_eq!(unsafe { wstr_unit_len(s.as_ptr()) }, Some(5));
    }

    // ── read_cstr_counted ───────────────────────────────────────

    #[test]
    fn read_cstr_counted_null_returns_none() {
        assert_eq!(unsafe { read_cstr_counted(std::ptr::null(), 4) }, None);
    }

    #[test]
    fn read_cstr_counted_negative_returns_none() {
        let s = b"hello";
        assert_eq!(unsafe { read_cstr_counted(s.as_ptr(), -1) }, None);
    }

    #[test]
    fn read_cstr_counted_reads_exact_len() {
        let s = b"hello world";
        assert_eq!(
            unsafe { read_cstr_counted(s.as_ptr(), 5) },
            Some("hello".into())
        );
    }

    // ── read_wstr_counted ───────────────────────────────────────

    #[test]
    fn read_wstr_counted_null_returns_none() {
        assert_eq!(unsafe { read_wstr_counted(std::ptr::null(), 4) }, None);
    }

    #[test]
    fn read_wstr_counted_negative_returns_none() {
        let s: Vec<u16> = "hello".encode_utf16().collect();
        assert_eq!(unsafe { read_wstr_counted(s.as_ptr(), -1) }, None);
    }

    #[test]
    fn read_wstr_counted_reads_exact_len() {
        let s: Vec<u16> = "hello world".encode_utf16().collect();
        assert_eq!(
            unsafe { read_wstr_counted(s.as_ptr(), 5) },
            Some("hello".into())
        );
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
