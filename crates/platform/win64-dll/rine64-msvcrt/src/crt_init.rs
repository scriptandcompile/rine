//! MSVCRT C runtime initialization: __getmainargs, _initterm, _initterm_e.

use rine_common_msvcrt::{cached_main_args, run_initterm, run_initterm_e};

/// __getmainargs — MSVCRT CRT argument initialization.
///
/// Populates `*p_argc`, `*p_argv`, and `*p_envp` with the program's
/// command-line arguments and environment. Called early in the CRT
/// startup sequence before `main()`.
///
/// # Safety
/// All pointer arguments must be valid for writes or null.
#[rine_dlls::implemented]
pub unsafe extern "win64" fn __getmainargs(
    p_argc: *mut i32,
    p_argv: *mut *mut *mut i8,
    p_envp: *mut *mut *mut i8,
    _do_wildcard: i32,
    _start_info: *mut core::ffi::c_void,
) -> i32 {
    tracing::trace!("msvcrt::__getmainargs");
    let args = cached_main_args();

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
        run_initterm(start, end, |func| {
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
    let result = unsafe { run_initterm_e(start, end, |func| func()) };
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
        assert!(argc >= 1); // at minimum the program name
        assert!(!argv.is_null());
        assert!(!envp.is_null());
    }

    #[test]
    fn getmainargs_tolerates_null_pointers() {
        // Should not crash when given null out-pointers.
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
    fn initterm_handles_null_range() {
        // Should be a no-op, not crash.
        unsafe {
            _initterm(std::ptr::null(), std::ptr::null());
        }
    }
}
