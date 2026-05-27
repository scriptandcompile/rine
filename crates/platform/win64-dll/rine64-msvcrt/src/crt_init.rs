//! MSVCRT C runtime initialization: __getmainargs, _initterm, _initterm_e.

use rine_common_msvcrt as common;

/// __wgetmainargs — wide-character MSVCRT CRT argument initialization.
///
/// Populates `*p_argc`, `*p_argv`, and `*p_envp` with the program's
/// command-line arguments and environment as wide (UTF-16) character
/// strings. Called early in the CRT startup sequence before `wmain()`.
///
/// # Arguments
/// * `p_argc` - Out pointer for the argument count. May be null.
/// * `p_argv` - Out pointer for the wide argument vector (`wchar_t***`). May be null.
/// * `p_envp` - Out pointer for the wide environment vector (`wchar_t***`). May be null.
/// * `_do_wildcard` - If non-zero, expand wildcard characters in arguments (currently ignored).
/// * `_start_info` - Pointer to startup info structure (currently ignored).
///
/// # Returns
/// Returns 0 on success. Always returns 0 in the current partial implementation.
///
/// # Safety
/// All pointer arguments must be valid for writes or null. Pointed-to
/// pointers (`*p_argv`, `*p_envp`) must point to valid writable locations
/// or be null.
///
/// # Notes
/// - The `_do_wildcard` parameter is currently ignored; wildcard expansion is not implemented.
/// - The `_start_info` parameter is currently ignored; no special startup information is used.
/// - This is a partial implementation equivalent to the narrow `__getmainargs`.
#[rine_dlls::partial]
pub unsafe extern "win64" fn __wgetmainargs(
    p_argc: *mut i32,
    p_argv: *mut *mut *mut u16,
    p_envp: *mut *mut *mut u16,
    _do_wildcard: i32,
    _start_info: *mut core::ffi::c_void,
) -> i32 {
    tracing::trace!("msvcrt::__wgetmainargs");
    let args = common::cached_wide_main_args();

    if !p_argc.is_null() {
        unsafe { *p_argc = args.argc() };
    }
    if !p_argv.is_null() {
        unsafe { *p_argv = args.argv_ptr() };
    }
    if !p_envp.is_null() {
        unsafe { *p_envp = args.envp_ptr() };
    }

    0 // success
}

/// __getmainargs — MSVCRT CRT argument initialization.
///
/// Populates `*p_argc`, `*p_argv`, and `*p_envp` with the program's
/// command-line arguments and environment. Called early in the CRT
/// startup sequence before `main()`.
///
/// # Safety
/// All pointer arguments must be valid for writes or null.
///
/// # Notes
/// The `_do_wildcard` and `_start_info` parameters are currently ignored, as wildcard expansion
/// is not implemented and no special startup information is needed.
/// They are included in the signature for compatibility with the expected CRT function signature.
#[rine_dlls::partial]
pub unsafe extern "win64" fn __getmainargs(
    p_argc: *mut i32,
    p_argv: *mut *mut *mut i8,
    p_envp: *mut *mut *mut i8,
    _do_wildcard: i32,
    _start_info: *mut core::ffi::c_void,
) -> i32 {
    tracing::trace!("msvcrt::__getmainargs");
    let args = common::cached_main_args();

    if !p_argc.is_null() {
        unsafe { *p_argc = args.argc() };
    }
    if !p_argv.is_null() {
        unsafe { *p_argv = args.argv_ptr() };
    }
    if !p_envp.is_null() {
        unsafe { *p_envp = args.envp_ptr() };
    }

    0 // success
}

/// _initterm — call a table of `void (*)(void)` initializer pointers.
///
/// Iterates from `start` to `end` (exclusive), calling each non-null
/// function pointer. Used by the CRT to run static constructors and
/// other pre-main initializers.
///
/// # Safety
/// `start` and `end` must delimit a valid array of function pointers
/// (or null entries).
#[rine_dlls::implemented]
pub unsafe extern "win64" fn _initterm(
    start: *const Option<unsafe extern "win64" fn()>,
    end: *const Option<unsafe extern "win64" fn()>,
) {
    tracing::trace!("msvcrt::_initterm");
    unsafe {
        common::run_initterm(start, end, |func| {
            func();
        });
    }
}

/// _initterm_e — like `_initterm`, but callbacks return `int`.
///
/// Stops on the first non-zero return value and propagates it.
/// Returns 0 if all initializers succeeded (or the table is empty).
///
/// # Safety
/// Same as `_initterm`.
#[rine_dlls::implemented]
pub unsafe extern "win64" fn _initterm_e(
    start: *const Option<unsafe extern "win64" fn() -> i32>,
    end: *const Option<unsafe extern "win64" fn() -> i32>,
) -> i32 {
    tracing::trace!("msvcrt::_initterm_e");
    let result = unsafe { common::run_initterm_e(start, end, |func| func()) };
    if result != 0 {
        tracing::warn!(result, "msvcrt::_initterm_e: initializer failed");
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn getmainargs_populates_argc() {
        let mut argc: i32 = 0;
        let mut argv: *mut *mut i8 = std::ptr::null_mut();
        let mut envp: *mut *mut i8 = std::ptr::null_mut();
        let result =
            unsafe { __getmainargs(&mut argc, &mut argv, &mut envp, 0, std::ptr::null_mut()) };
        assert_eq!(result, 0);
        assert!(argc >= 1);
        assert!(!argv.is_null());
        assert!(!envp.is_null());
    }

    #[test]
    fn getmainargs_tolerates_null_pointers() {
        let result = unsafe {
            __getmainargs(
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
            )
        };
        assert_eq!(result, 0);
    }

    #[test]
    fn wgetmainargs_populates_argc() {
        let mut argc: i32 = 0;
        let mut argv: *mut *mut u16 = std::ptr::null_mut();
        let mut envp: *mut *mut u16 = std::ptr::null_mut();
        let result =
            unsafe { __wgetmainargs(&mut argc, &mut argv, &mut envp, 0, std::ptr::null_mut()) };
        assert_eq!(result, 0);
        assert!(argc >= 1);
        assert!(!argv.is_null());
        assert!(!envp.is_null());
    }

    #[test]
    fn wgetmainargs_tolerates_null_pointers() {
        let result = unsafe {
            __wgetmainargs(
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
            )
        };
        assert_eq!(result, 0);
    }

    #[test]
    fn wgetmainargs_argv_entries_are_null_terminated() {
        let mut argc: i32 = 0;
        let mut argv: *mut *mut u16 = std::ptr::null_mut();
        let mut envp: *mut *mut u16 = std::ptr::null_mut();
        let result =
            unsafe { __wgetmainargs(&mut argc, &mut argv, &mut envp, 0, std::ptr::null_mut()) };
        assert_eq!(result, 0);
        assert!(argc >= 1);
        unsafe {
            let argv0 = *argv as *const u16;
            let mut len = 0;
            while *argv0.add(len) != 0 {
                len += 1;
            }
            assert!(len > 0, "argv[0] should be a non-empty wide string");
        }
    }

    #[test]
    fn wgetmainargs_envp_is_non_null() {
        let mut argc: i32 = 0;
        let mut argv: *mut *mut u16 = std::ptr::null_mut();
        let mut envp: *mut *mut u16 = std::ptr::null_mut();
        let result =
            unsafe { __wgetmainargs(&mut argc, &mut argv, &mut envp, 0, std::ptr::null_mut()) };
        assert_eq!(result, 0);
        unsafe {
            assert!(!(*envp).is_null(), "envp[0] should be non-null");
        }
    }

    #[test]
    fn initterm_handles_null_range() {
        unsafe {
            _initterm(std::ptr::null(), std::ptr::null());
        }
    }
}
