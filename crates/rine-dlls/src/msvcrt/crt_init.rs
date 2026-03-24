//! MSVCRT C runtime initialization: __getmainargs, _initterm, _initterm_e.

use std::ffi::CString;
use std::sync::OnceLock;

/// Cached C-style argc/argv/envp built once from `std::env`.
struct MainArgs {
    argc: i32,
    argv_ptrs: Vec<*mut i8>,
    envp_ptrs: Vec<*mut i8>,
    // The CStrings own the backing memory; the *mut i8 pointers above
    // borrow into them. Because this lives in a OnceLock<> the storage
    // is valid for the lifetime of the process.
    _argv_strings: Vec<CString>,
    _envp_strings: Vec<CString>,
}

// SAFETY: All raw pointers in MainArgs point into the CString vecs that
// are co-located in the same struct and never moved or freed (OnceLock).
unsafe impl Send for MainArgs {}
unsafe impl Sync for MainArgs {}

static MAIN_ARGS: OnceLock<MainArgs> = OnceLock::new();

fn cached_main_args() -> &'static MainArgs {
    MAIN_ARGS.get_or_init(|| {
        // Build argv from process arguments.
        let args: Vec<String> = std::env::args().collect();
        let argv_strings: Vec<CString> = args
            .iter()
            .map(|a| CString::new(a.as_str()).unwrap_or_default())
            .collect();
        let mut argv_ptrs: Vec<*mut i8> = argv_strings
            .iter()
            .map(|cs| cs.as_ptr() as *mut i8)
            .collect();
        argv_ptrs.push(std::ptr::null_mut()); // NULL sentinel

        // Build envp from process environment.
        let envp_strings: Vec<CString> = std::env::vars()
            .map(|(k, v)| CString::new(format!("{k}={v}")).unwrap_or_default())
            .collect();
        let mut envp_ptrs: Vec<*mut i8> = envp_strings
            .iter()
            .map(|cs| cs.as_ptr() as *mut i8)
            .collect();
        envp_ptrs.push(std::ptr::null_mut()); // NULL sentinel

        MainArgs {
            argc: args.len() as i32,
            argv_ptrs,
            envp_ptrs,
            _argv_strings: argv_strings,
            _envp_strings: envp_strings,
        }
    })
}

/// __getmainargs — MSVCRT CRT argument initialization.
///
/// Populates `*p_argc`, `*p_argv`, and `*p_envp` with the program's
/// command-line arguments and environment. Called early in the CRT
/// startup sequence before `main()`.
///
/// # Safety
/// All pointer arguments must be valid for writes or null.
pub unsafe extern "C" fn __getmainargs(
    p_argc: *mut i32,
    p_argv: *mut *mut *mut i8,
    p_envp: *mut *mut *mut i8,
    _do_wildcard: i32,
    _start_info: *mut core::ffi::c_void,
) -> i32 {
    tracing::trace!("msvcrt::__getmainargs");
    let args = cached_main_args();

    if !p_argc.is_null() {
        unsafe { *p_argc = args.argc };
    }
    if !p_argv.is_null() {
        unsafe { *p_argv = args.argv_ptrs.as_ptr() as *mut *mut i8 };
    }
    if !p_envp.is_null() {
        unsafe { *p_envp = args.envp_ptrs.as_ptr() as *mut *mut i8 };
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
pub unsafe extern "C" fn _initterm(
    start: *const Option<unsafe extern "C" fn()>,
    end: *const Option<unsafe extern "C" fn()>,
) {
    tracing::trace!("msvcrt::_initterm");
    if start.is_null() || end.is_null() || start >= end {
        return;
    }
    let count = unsafe { end.offset_from(start) } as usize;
    for i in 0..count {
        if let Some(func) = unsafe { *start.add(i) } {
            unsafe { func() };
        }
    }
}

/// _initterm_e — like `_initterm`, but callbacks return `int`.
///
/// Stops on the first non-zero return value and propagates it.
/// Returns 0 if all initializers succeeded (or the table is empty).
///
/// # Safety
/// Same as `_initterm`.
pub unsafe extern "C" fn _initterm_e(
    start: *const Option<unsafe extern "C" fn() -> i32>,
    end: *const Option<unsafe extern "C" fn() -> i32>,
) -> i32 {
    tracing::trace!("msvcrt::_initterm_e");
    if start.is_null() || end.is_null() || start >= end {
        return 0;
    }
    let count = unsafe { end.offset_from(start) } as usize;
    for i in 0..count {
        if let Some(func) = unsafe { *start.add(i) } {
            let result = unsafe { func() };
            if result != 0 {
                tracing::warn!(result, index = i, "msvcrt::_initterm_e: initializer failed");
                return result;
            }
        }
    }
    0
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
    fn initterm_calls_entries() {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);

        unsafe extern "C" fn inc() {
            COUNTER.fetch_add(1, Ordering::Relaxed);
        }

        let table: [Option<unsafe extern "C" fn()>; 3] = [Some(inc), None, Some(inc)];

        COUNTER.store(0, Ordering::Relaxed);
        unsafe {
            _initterm(table.as_ptr(), table.as_ptr().add(table.len()));
        }
        assert_eq!(COUNTER.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn initterm_handles_null_range() {
        // Should be a no-op, not crash.
        unsafe {
            _initterm(std::ptr::null(), std::ptr::null());
        }
    }

    #[test]
    fn initterm_e_stops_on_error() {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);

        unsafe extern "C" fn ok() -> i32 {
            COUNTER.fetch_add(1, Ordering::Relaxed);
            0
        }
        unsafe extern "C" fn fail() -> i32 {
            COUNTER.fetch_add(1, Ordering::Relaxed);
            42
        }
        unsafe extern "C" fn unreachable_init() -> i32 {
            COUNTER.fetch_add(100, Ordering::Relaxed);
            0
        }

        let table: [Option<unsafe extern "C" fn() -> i32>; 3] =
            [Some(ok), Some(fail), Some(unreachable_init)];

        COUNTER.store(0, Ordering::Relaxed);
        let result = unsafe { _initterm_e(table.as_ptr(), table.as_ptr().add(table.len())) };
        assert_eq!(result, 42);
        // Only the first two should have been called.
        assert_eq!(COUNTER.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn initterm_e_returns_zero_on_success() {
        unsafe extern "C" fn ok() -> i32 {
            0
        }

        let table: [Option<unsafe extern "C" fn() -> i32>; 2] = [Some(ok), Some(ok)];

        let result = unsafe { _initterm_e(table.as_ptr(), table.as_ptr().add(table.len())) };
        assert_eq!(result, 0);
    }
}
