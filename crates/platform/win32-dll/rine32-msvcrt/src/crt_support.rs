//! MSVCRT CRT support functions: exception handling, signal, locks, file descriptors.

use rine_common_msvcrt::{
    abort_process, amsg_exit, c_specific_handler_result, commode_ptr, errno_location,
    fake_iob_32_ptr, fmode_ptr, initenv_ptr, lock, onexit, set_app_type, set_usermatherr,
    signal_default, unlock,
};

/// __set_app_type — set the application type (subsystem) for the CRT.
///
/// This is a no-op in Phase 1; a production implementation would configure
/// CRT behavior based on whether the app is a console or GUI application.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __set_app_type(app_type: i32) {
    tracing::trace!(app_type, "msvcrt::__set_app_type");
    set_app_type(app_type);
}

/// __setusermatherr — set a custom math error handler.
///
/// This is a no-op in Phase 1; a production implementation would let
/// the user install a handler for floating-point errors.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __setusermatherr(handler: usize) {
    tracing::trace!(handler, "msvcrt::__setusermatherr");
    set_usermatherr(handler);
}

/// __C_specific_handler — handle C SEH (Structured Exception Handling) exceptions.
///
/// Returns a handler code (1 = continue execution, 0 = call next handler).
/// This is a stub in Phase 1.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "C" fn __C_specific_handler(
    _exception_record: usize,
    _establisher_frame: usize,
    _context_record: usize,
    _dispatcher_context: usize,
) -> i32 {
    tracing::trace!("msvcrt::__C_specific_handler");
    c_specific_handler_result()
}

/// __iob_func — get the fake stdio FILE buffer table.
///
/// Returns a pointer to a table of three FILE-like structures for
/// stdin, stdout, stderr. The msvcrt DLL exposes these as `_iob` data.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __iob_func() -> *mut u8 {
    tracing::trace!("msvcrt::__iob_func");
    fake_iob_32_ptr()
}

/// _onexit — register a function to be called at process exit.
///
/// In Phase 1, this just returns the function pointer unchanged.
/// A production implementation would add it to an atexit chain.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn _onexit(func: usize) -> usize {
    tracing::trace!(func, "msvcrt::_onexit");
    onexit(func)
}

/// _amsg_exit — print an error message and exit the process.
///
/// Called internally by the CRT when fatal errors occur.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn _amsg_exit(msg_num: i32) {
    tracing::trace!(msg_num, "msvcrt::_amsg_exit");
    amsg_exit(msg_num)
}

/// abort — raise SIGABRT and terminate the process.
///
/// Does not return.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn abort() {
    tracing::error!("msvcrt::abort");
    abort_process()
}

/// signal — install a signal handler or get the current one.
///
/// Returns the previous handler. This is a stub in Phase 1.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn signal(sig: i32, handler: usize) -> usize {
    tracing::trace!(sig, handler, "msvcrt::signal");
    signal_default(sig, handler)
}

/// _lock — acquire a CRT lock (for thread safety of stdio, malloc, etc.).
///
/// In Phase 1, this is a no-op. A production implementation would use
/// actual OS synchronization primitives.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn _lock(locknum: i32) {
    tracing::trace!(locknum, "msvcrt::_lock");
    lock(locknum);
}

/// _unlock — release a CRT lock.
///
/// In Phase 1, this is a no-op.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn _unlock(locknum: i32) {
    tracing::trace!(locknum, "msvcrt::_unlock");
    unlock(locknum);
}

/// _errno — get a pointer to the thread-local `errno` value.
///
/// Called by the `errno` macro or by C code that wants to read/write
/// the error code from the last failed system call.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn _errno() -> *mut i32 {
    tracing::trace!("msvcrt::_errno");
    errno_location()
}

/// __p__environ — get a pointer to the environment pointer table.
///
/// Returns a triple pointer (for calling context compatibility).
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "C" fn __p__environ() -> *const *const *const i8 {
    tracing::trace!("msvcrt::__p__environ");
    initenv_ptr() as *const *const *const i8
}

/// __p__fmode — get a pointer to the file translation mode flag.
///
/// Returned pointer points to a mutable `int` that controls whether
/// text files are opened in binary or text mode by default.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "C" fn __p__fmode() -> *mut i32 {
    tracing::trace!("msvcrt::__p__fmode");
    fmode_ptr()
}

/// __p__commode — get a pointer to the file commit mode flag.
///
/// Returned pointer points to a mutable `int` that controls whether files
/// are committed to disk before write() returns.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "C" fn __p__commode() -> *mut i32 {
    tracing::trace!("msvcrt::__p__commode");
    commode_ptr()
}
