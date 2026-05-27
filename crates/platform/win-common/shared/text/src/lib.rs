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

#[cfg(test)]
mod tests {
    use super::{write_ansi_truncating_nul, write_wide_truncating_nul};

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
}
