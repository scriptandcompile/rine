//! MSVCRT stdio functions: printf, puts, fprintf, etc.
//!
//! These are stub implementations or no-ops for Phase 1.
//! A production implementation would connect to actual file descriptor handling.

/// printf — print formatted output to stdout.
///
/// Stub implementation in Phase 1.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn printf(_format: *const i8) -> i32 {
    tracing::trace!("msvcrt::printf (stub)");
    0
}

/// puts — print a string to stdout followed by a newline.
///
/// Stub implementation in Phase 1.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn puts(_s: *const i8) -> i32 {
    tracing::trace!("msvcrt::puts (stub)");
    0
}

/// fprintf — print formatted output to a file.
///
/// Stub implementation in Phase 1.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn fprintf(_file: *const core::ffi::c_void, _format: *const i8) -> i32 {
    tracing::trace!("msvcrt::fprintf (stub)");
    0
}

/// vfprintf — print formatted output to a file using a va_list.
///
/// Stub implementation in Phase 1.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn vfprintf(
    _file: *const core::ffi::c_void,
    _format: *const i8,
    _args: *const core::ffi::c_void,
) -> i32 {
    tracing::trace!("msvcrt::vfprintf (stub)");
    0
}

/// fwrite — write data to a file.
///
/// Stub implementation in Phase 1.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn fwrite(
    _ptr: *const core::ffi::c_void,
    _size: usize,
    _nmemb: usize,
    _file: *const core::ffi::c_void,
) -> usize {
    tracing::trace!("msvcrt::fwrite (stub)");
    0
}
