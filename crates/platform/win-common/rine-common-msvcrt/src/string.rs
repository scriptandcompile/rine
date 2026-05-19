use core::ffi::c_char;

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
}
