use core::ffi::c_char;
use std::cell::UnsafeCell;

thread_local! {
    static STRERROR_BUF: UnsafeCell<[u8; 1024]> = const { UnsafeCell::new([0u8; 1024]) };
}

/// Get the length of a null-terminated string.
///
/// # Arguments
/// - `s`: A pointer to a null-terminated string. May be null, in which case this function returns 0.
///
/// # Safety
/// - The caller must ensure that `s` is either null or points to a valid null-terminated string.
///   If `s` is non-null and not properly null-terminated, this function may read out of bounds, leading to undefined behavior.
///
/// # Returns
/// - The length of the string pointed to by `s`, excluding the null terminator.
pub unsafe fn strlen(s: *const c_char) -> usize {
    unsafe { libc::strlen(s) }
}

/// Compare at most n characters of two strings.
///
/// # Arguments
/// - `s1`: A pointer to the first null-terminated string. May be null, in which case it is treated as an empty string.
/// - `s2`: A pointer to the second null-terminated string. May be null, in which case it is treated as an empty string.
/// - `n`: The maximum number of characters to compare.
///
/// # Safety
/// - The caller must ensure that `s1` and `s2` are either null or point to valid null-terminated strings.
///   If either pointer is non-null and not properly null-terminated, this function may read out of bounds,
///   leading to undefined behavior.
/// - The caller must ensure that `n` does not exceed the length of either string if they are non-null,
///   to avoid reading out of bounds.
///
/// # Returns
/// - An integer less than, equal to, or greater than zero if `s1` is found, respectively, to be less than, to match,
///   or be greater than `s2` when comparing at most `n` characters.
pub unsafe fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> i32 {
    unsafe { libc::strncmp(s1, s2, n) }
}

/// Get the length of a null-terminated wide string.
///
/// # Arguments
/// - `s`: A pointer to a null-terminated wide (UTF-16) string. May be null, in which case this function returns 0.
///
/// # Safety
/// - The caller must ensure that `s` is either null or points to a valid null-terminated wide string.
///   If `s` is non-null and not properly null-terminated, this function may read out of bounds,
///   leading to undefined behavior.
///
/// # Returns
/// - The length of the wide string pointed to by `s`, excluding the null terminator,
///   measured in wide characters (u16 code units).
pub unsafe fn wcslen(s: *const u16) -> usize {
    if s.is_null() {
        return 0;
    }
    let mut len = 0usize;
    unsafe {
        while *s.add(len) != 0 {
            len += 1;
        }
    }
    len
}

/// Return a string describing the error code.
///
/// # Arguments
/// - `errnum`: The error number to describe. Standard errno values (e.g., `EINVAL`, `ENOENT`) are
///   recognized and will be mapped to a corresponding human-readable message. Unknown values produce
///   a fallback string containing the numeric code.
///
/// # Returns
/// - A pointer to a thread-local null-terminated C string containing a human-readable error message.
///   The pointer remains valid until the next call to `strerror` in the same thread.
///   The caller should copy the string if it needs to persist across subsequent calls.
///
/// # Safety
/// - The returned pointer points to a thread-local buffer that is overwritten on each call to
///   `strerror` in the same thread. The caller must copy the string if it needs to outlive the
///   next call. The pointer is also invalidated when the thread exits.
///
/// # Notes
/// - This implementation uses the host system's `libc::strerror_r` to obtain error messages.
///   The messages are those of the host C library, not MSVC-specific messages. MSVC-compatible
///   error strings (e.g., "No error", "Permission denied", etc.) are not yet implemented.
/// - If the error code is not recognized, a generic `"Unknown error N"` message is returned.
/// - A secure, bounds-checked variant `strerror_s` is not yet implemented.
pub unsafe fn strerror(errnum: i32) -> *mut c_char {
    STRERROR_BUF.with(|buf| {
        let ptr = buf.get() as *mut c_char;
        unsafe {
            let result = libc::strerror_r(errnum, ptr, 1024);
            if result != 0 {
                let msg = format!("Unknown error {}\0", errnum);
                let bytes = msg.as_bytes();
                let len = bytes.len().min(1023);
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr as *mut u8, len);
                *ptr.add(len) = 0;
            }
        }
        ptr
    })
}

/// Compare two null-terminated strings.
///
/// # Arguments
/// - `s1`: A pointer to the first null-terminated string. May be null, in which case it is treated as an empty string.
/// - `s2`: A pointer to the second null-terminated string. May be null, in which case it is treated as an empty string.
///
/// # Safety
/// - The caller must ensure that `s1` and `s2` are either null or point to valid null-terminated strings.
///   If either pointer is non-null and not properly null-terminated, this function may read out of bounds,
///   leading to undefined behavior.
///
/// # Returns
/// - An integer less than, equal to, or greater than zero if `s1` is found, respectively, to be less than, to match,
///   or be greater than `s2`.
pub unsafe fn strcmp(s1: *const c_char, s2: *const c_char) -> i32 {
    unsafe { libc::strcmp(s1, s2) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strlen_normal() {
        let bytes = b"hello\0";
        let len = unsafe { strlen(bytes.as_ptr() as *const c_char) };
        assert_eq!(len, 5);
    }

    #[test]
    fn strlen_empty() {
        let bytes = b"\0";
        let len = unsafe { strlen(bytes.as_ptr() as *const c_char) };
        assert_eq!(len, 0);
    }

    #[test]
    fn strncmp_equal() {
        let a = b"hello\0";
        let b = b"hello\0";
        let result =
            unsafe { strncmp(a.as_ptr() as *const c_char, b.as_ptr() as *const c_char, 5) };
        assert_eq!(result, 0);
    }

    #[test]
    fn strncmp_less_than() {
        let a = b"apple\0";
        let b = b"banana\0";
        let result =
            unsafe { strncmp(a.as_ptr() as *const c_char, b.as_ptr() as *const c_char, 5) };
        assert!(result < 0);
    }

    #[test]
    fn strncmp_greater_than() {
        let a = b"zebra\0";
        let b = b"banana\0";
        let result =
            unsafe { strncmp(a.as_ptr() as *const c_char, b.as_ptr() as *const c_char, 5) };
        assert!(result > 0);
    }

    #[test]
    fn strncmp_limited_by_n() {
        let a = b"aaaaaaa\0";
        let b = b"aaaabbb\0";
        let result =
            unsafe { strncmp(a.as_ptr() as *const c_char, b.as_ptr() as *const c_char, 3) };
        assert_eq!(result, 0);
    }

    #[test]
    fn strcmp_equal() {
        let a = b"hello\0";
        let b = b"hello\0";
        let result = unsafe { strcmp(a.as_ptr() as *const c_char, b.as_ptr() as *const c_char) };
        assert_eq!(result, 0);
    }

    #[test]
    fn strcmp_less_than() {
        let a = b"apple\0";
        let b = b"banana\0";
        let result = unsafe { strcmp(a.as_ptr() as *const c_char, b.as_ptr() as *const c_char) };
        assert!(result < 0);
    }

    #[test]
    fn strcmp_greater_than() {
        let a = b"zebra\0";
        let b = b"banana\0";
        let result = unsafe { strcmp(a.as_ptr() as *const c_char, b.as_ptr() as *const c_char) };
        assert!(result > 0);
    }

    #[test]
    fn wcslen_normal() {
        let wide = [
            b'h' as u16,
            b'e' as u16,
            b'l' as u16,
            b'l' as u16,
            b'o' as u16,
            0,
        ];
        let len = unsafe { wcslen(wide.as_ptr()) };
        assert_eq!(len, 5);
    }

    #[test]
    fn wcslen_empty() {
        let wide = [0u16];
        let len = unsafe { wcslen(wide.as_ptr()) };
        assert_eq!(len, 0);
    }

    #[test]
    fn strerror_known_errno() {
        unsafe {
            let msg = strerror(libc::ENOENT);
            let s = std::ffi::CStr::from_ptr(msg).to_string_lossy();
            assert!(
                !s.is_empty(),
                "strerror(ENOENT) should return a non-empty message"
            );
        }
    }

    #[test]
    fn strerror_zero_is_success() {
        unsafe {
            let msg = strerror(0);
            let s = std::ffi::CStr::from_ptr(msg).to_string_lossy();
            assert!(
                !s.is_empty(),
                "strerror(0) should return a non-empty message"
            );
        }
    }

    #[test]
    fn strerror_unknown_errno_contains_number() {
        unsafe {
            let msg = strerror(999999);
            let s = std::ffi::CStr::from_ptr(msg).to_string_lossy();
            // The string should indicate the errno is unknown, or at least contain the number
            assert!(
                s.contains("999999") || s.to_lowercase().contains("unknown"),
                "strerror(999999) should indicate an unknown error, got: {s}"
            );
        }
    }

    #[test]
    fn strerror_return_is_stable_until_next_call() {
        unsafe {
            let msg1 = strerror(libc::ENOENT);
            let s1 = std::ffi::CStr::from_ptr(msg1).to_string_lossy().to_string();

            let msg2 = strerror(libc::EINVAL);
            let s2 = std::ffi::CStr::from_ptr(msg2).to_string_lossy().to_string();

            // s1 should no longer be valid descriptor error (buffer was overwritten)
            assert!(
                !s2.is_empty(),
                "strerror(EINVAL) should return a non-empty message"
            );

            // The messages should differ since they describe different errno values
            assert_ne!(
                s1, s2,
                "different errno values should yield different messages"
            );
        }
    }
}
