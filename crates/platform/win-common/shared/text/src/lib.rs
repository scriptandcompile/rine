/// Write UTF-8 bytes into a caller-supplied ANSI buffer, truncating to fit
/// and always writing a trailing NUL when capacity is positive.
///
/// Returns the number of bytes written, excluding the trailing NUL.
/// Returns `0` when `buffer` is null or `capacity <= 0`.
///
/// # Safety
/// `buffer` must point to at least `capacity` writable bytes when non-null.
pub unsafe fn write_ansi_truncating_nul(buffer: *mut u8, capacity: isize, value: &[u8]) -> usize {
    if buffer.is_null() || capacity <= 0 {
        return 0;
    }

    let cap = capacity as usize;
    let write_len = value.len().min(cap.saturating_sub(1));

    unsafe {
        core::ptr::copy_nonoverlapping(value.as_ptr(), buffer, write_len);
        *buffer.add(write_len) = 0;
    }

    write_len
}

/// Write UTF-16 code units into a caller-supplied wide buffer, truncating to
/// fit and always writing a trailing NUL when capacity is positive.
///
/// Returns the number of UTF-16 code units written, excluding the trailing NUL.
/// Returns `0` when `buffer` is null or `capacity <= 0`.
///
/// # Safety
/// `buffer` must point to at least `capacity` writable `u16` values when non-null.
pub unsafe fn write_wide_truncating_nul(buffer: *mut u16, capacity: isize, value: &[u16]) -> usize {
    if buffer.is_null() || capacity <= 0 {
        return 0;
    }

    let cap = capacity as usize;
    let write_len = value.len().min(cap.saturating_sub(1));

    unsafe {
        core::ptr::copy_nonoverlapping(value.as_ptr(), buffer, write_len);
        *buffer.add(write_len) = 0;
    }

    write_len
}

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

/// Write an ANSI string (with null terminator) into a caller-supplied buffer.
///
/// Returns the number of characters written (excluding the null terminator),
/// or the required buffer size (including the null terminator) if the buffer
/// is too small or null.
///
/// # Safety
/// `buf` must be null or point to at least `buf_size` writable bytes.
pub unsafe fn write_cstr(buf: *mut u8, buf_size: u32, value: &str) -> u32 {
    let needed = value.len() as u32 + 1;
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

#[cfg(test)]
mod tests {
    use super::{
        json_escape, read_cstr, read_cstr_counted, read_wstr, read_wstr_counted,
        write_ansi_truncating_nul, write_cstr, write_wide_truncating_nul, write_wstr,
        wstr_unit_len,
    };

    #[test]
    fn ansi_null_buffer_returns_zero() {
        let written = unsafe { write_ansi_truncating_nul(core::ptr::null_mut(), 8, b"abc") };
        assert_eq!(written, 0);
    }

    #[test]
    fn ansi_zero_capacity_returns_zero() {
        let mut out = [0u8; 4];
        let written = unsafe { write_ansi_truncating_nul(out.as_mut_ptr(), 0, b"abc") };
        assert_eq!(written, 0);
        assert_eq!(out, [0, 0, 0, 0]);
    }

    #[test]
    fn ansi_negative_capacity_returns_zero() {
        let mut out = [0u8; 4];
        let written = unsafe { write_ansi_truncating_nul(out.as_mut_ptr(), -1, b"abc") };
        assert_eq!(written, 0);
        assert_eq!(out, [0, 0, 0, 0]);
    }

    #[test]
    fn ansi_exact_fit_writes_full_string_and_nul() {
        let mut out = [0u8; 4];
        let written =
            unsafe { write_ansi_truncating_nul(out.as_mut_ptr(), out.len() as isize, b"abc") };
        assert_eq!(written, 3);
        assert_eq!(&out, b"abc\0");
    }

    #[test]
    fn ansi_truncates_to_capacity_minus_one() {
        let mut out = [0u8; 4];
        let written =
            unsafe { write_ansi_truncating_nul(out.as_mut_ptr(), out.len() as isize, b"abcdef") };
        assert_eq!(written, 3);
        assert_eq!(&out, b"abc\0");
    }

    #[test]
    fn wide_null_buffer_returns_zero() {
        let source = [b'a' as u16, b'b' as u16, b'c' as u16];
        let written = unsafe { write_wide_truncating_nul(core::ptr::null_mut(), 8, &source) };
        assert_eq!(written, 0);
    }

    #[test]
    fn wide_zero_capacity_returns_zero() {
        let mut out = [0u16; 4];
        let source = [b'a' as u16, b'b' as u16, b'c' as u16];
        let written = unsafe { write_wide_truncating_nul(out.as_mut_ptr(), 0, &source) };
        assert_eq!(written, 0);
        assert_eq!(out, [0, 0, 0, 0]);
    }

    #[test]
    fn wide_negative_capacity_returns_zero() {
        let mut out = [0u16; 4];
        let source = [b'a' as u16, b'b' as u16, b'c' as u16];
        let written = unsafe { write_wide_truncating_nul(out.as_mut_ptr(), -1, &source) };
        assert_eq!(written, 0);
        assert_eq!(out, [0, 0, 0, 0]);
    }

    #[test]
    fn wide_exact_fit_writes_full_string_and_nul() {
        let mut out = [0u16; 4];
        let source = [b'a' as u16, b'b' as u16, b'c' as u16];
        let written =
            unsafe { write_wide_truncating_nul(out.as_mut_ptr(), out.len() as isize, &source) };
        assert_eq!(written, 3);
        assert_eq!(out, [b'a' as u16, b'b' as u16, b'c' as u16, 0]);
    }

    #[test]
    fn wide_truncates_to_capacity_minus_one() {
        let mut out = [0u16; 4];
        let source = [
            b'a' as u16,
            b'b' as u16,
            b'c' as u16,
            b'd' as u16,
            b'e' as u16,
            b'f' as u16,
        ];
        let written =
            unsafe { write_wide_truncating_nul(out.as_mut_ptr(), out.len() as isize, &source) };
        assert_eq!(written, 3);
        assert_eq!(out, [b'a' as u16, b'b' as u16, b'c' as u16, 0]);
    }

    #[test]
    fn read_cstr_null_returns_none() {
        assert_eq!(unsafe { read_cstr(std::ptr::null()) }, None);
    }

    #[test]
    fn read_cstr_valid() {
        let s = b"hello\0";
        assert_eq!(unsafe { read_cstr(s.as_ptr()) }, Some("hello".into()));
    }

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
        assert_eq!(n, 6);
    }

    #[test]
    fn write_cstr_null_buf() {
        let n = unsafe { write_cstr(std::ptr::null_mut(), 0, "hello") };
        assert_eq!(n, 6);
    }

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

    #[test]
    fn json_escape_escapes_specials() {
        let s = "a\\\"\n\r\tb";
        assert_eq!(json_escape(s), "a\\\\\\\"\\n\\r\\tb");
    }
}
