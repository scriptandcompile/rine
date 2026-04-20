//! MSVCRT C string and memory functions: strlen, strcmp, memcpy, memset.

/// strlen — get the length of a null-terminated string.
///
/// Returns 0 if `s` is null (non-standard but safe variant).
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn strlen(s: *const i8) -> usize {
    if s.is_null() {
        return 0;
    }
    unsafe { libc::strlen(s) }
}

/// strcmp — compare two strings lexically.
///
/// Returns: < 0 if lhs < rhs, 0 if equal, > 0 if lhs > rhs.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn strcmp(lhs: *const i8, rhs: *const i8) -> i32 {
    unsafe { libc::strcmp(lhs, rhs) }
}

/// strncmp — compare at most n bytes of two strings.
///
/// Returns: < 0 if lhs < rhs, 0 if equal (within n), > 0 if lhs > rhs (within n).
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn strncmp(lhs: *const i8, rhs: *const i8, n: usize) -> i32 {
    unsafe { libc::strncmp(lhs, rhs, n) }
}
