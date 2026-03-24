//! msvcrt stdio functions: printf, puts.

use core::ffi::{c_char, c_int};

unsafe extern "C" {
    /// printf — formatted output to stdout (provided by the host C library).
    ///
    /// Re-exported directly so the PE binary calls the real libc printf
    /// through the IAT. This avoids needing `c_variadic` for va_list
    /// forwarding while giving full format-string support.
    #[link_name = "printf"]
    pub safe fn printf(format: *const c_char, ...) -> c_int;
}

/// puts — write a string followed by a newline to stdout.
///
/// # Safety
/// `s` must be a valid, null-terminated C string.
pub unsafe extern "C" fn puts(s: *const c_char) -> c_int {
    if s.is_null() {
        return libc::EOF;
    }
    tracing::trace!("msvcrt::puts");
    unsafe { libc::puts(s) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn puts_rejects_null() {
        // Should return EOF, not crash.
        let result = unsafe { puts(core::ptr::null()) };
        assert_eq!(result, libc::EOF);
    }

    #[test]
    fn puts_writes_string() {
        let s = c"hello from puts";
        let result = unsafe { puts(s.as_ptr()) };
        // puts returns a non-negative value on success.
        assert!(result >= 0);
    }
}
