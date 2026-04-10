//! MSVCRT CRT support functions needed during CRT startup.
//!
//! These are called by the MinGW CRT startup code before `main()` runs.
//! Most are no-ops or minimal stubs for Phase 1.

use rine_common_msvcrt::{
    abort_process, amsg_exit, c_specific_handler_result, commode_ptr, errno_location,
    fake_iob_64_ptr, fmode_ptr, initenv_ptr, lock, onexit, set_app_type, set_usermatherr,
    signal_default, unlock,
};

/// __set_app_type — set the application type (console/GUI).
///
/// No-op: rine always runs as a console application.
#[allow(clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn __set_app_type(app_type: i32) {
    tracing::trace!("msvcrt::__set_app_type({app_type})");
    set_app_type(app_type);
}

/// __setusermatherr — register a custom math error handler.
///
/// No-op: we don't support custom math error handlers.
#[allow(clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn __setusermatherr(handler: usize) {
    tracing::trace!("msvcrt::__setusermatherr");
    set_usermatherr(handler);
}

/// __C_specific_handler — SEH personality function for x64 Windows.
///
/// Stub: returns ExceptionContinueSearch (1). Called only if an exception is
/// thrown, which shouldn't happen in a simple hello world.
#[allow(non_snake_case, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn __C_specific_handler(
    _exception_record: usize,
    _establisher_frame: usize,
    _context_record: usize,
    _dispatcher_context: usize,
) -> i32 {
    tracing::warn!("msvcrt::__C_specific_handler called — exceptions not supported");
    c_specific_handler_result()
}

/// _commode — return a pointer to the commit mode variable.
#[allow(clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn _commode() -> *mut i32 {
    commode_ptr()
}

/// Return the raw pointer to the _commode variable for data-export registration.
#[unsafe(no_mangle)]
pub fn commode_data_ptr() -> *mut i32 {
    commode_ptr()
}

/// _fmode — return a pointer to the default file translation mode.
#[allow(clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn _fmode() -> *mut i32 {
    fmode_ptr()
}

/// Return the raw pointer to the _fmode variable for data-export registration.
#[unsafe(no_mangle)]
pub fn fmode_data_ptr() -> *mut i32 {
    fmode_ptr()
}

/// __initenv — return a pointer to the initial environment pointer.
///
/// Returns a pointer to a NULL pointer (empty environment at CRT level;
/// the real environment is provided via `__getmainargs`).
#[allow(clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn __initenv() -> *const *const i8 {
    initenv_ptr() as *const *const i8
}

/// Return the raw pointer to the __initenv variable for data-export registration.
#[unsafe(no_mangle)]
pub fn initenv_data_ptr() -> *mut usize {
    initenv_ptr()
}

/// __iob_func — return pointer to the stdio FILE table.
///
/// Returns a fake FILE table. The first 3 entries represent stdin (0),
/// stdout (1), stderr (2). We store a marker fd in the first field of
/// each entry so fwrite/fprintf can identify the stream.
#[allow(clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn __iob_func() -> *mut u8 {
    fake_iob_64_ptr()
}

/// _onexit — register a function to be called at exit.
///
/// Stub: returns the function pointer (success) but does not actually
/// register it for later calling. Full atexit support in a later phase.
#[allow(clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn _onexit(func: usize) -> usize {
    tracing::trace!("msvcrt::_onexit");
    onexit(func)
}

/// _amsg_exit — display a runtime error message and abort.
#[allow(clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn _amsg_exit(msg_num: i32) {
    amsg_exit(msg_num)
}

/// abort — abnormally terminate the process.
#[allow(clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn abort() {
    tracing::debug!("msvcrt::abort");
    abort_process()
}

/// signal — install a signal handler.
///
/// Stub: returns SIG_DFL (0). Minimal implementation since Windows signals
/// are rarely used in practice.
#[allow(clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn signal(
    sig: i32,
    handler: usize, // void (*)(int)
) -> usize {
    signal_default(sig, handler)
}

/// _lock — acquire an internal CRT lock.
///
/// No-op for single-threaded Phase 1.
#[allow(clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn _lock(locknum: i32) {
    lock(locknum);
}

/// _unlock — release an internal CRT lock.
///
/// No-op for single-threaded Phase 1.
#[allow(clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn _unlock(locknum: i32) {
    unlock(locknum);
}

/// _errno — return a pointer to the per-thread errno value.
///
/// Returns a pointer to libc's errno, which is thread-local.
#[allow(clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn _errno() -> *mut i32 {
    errno_location()
}

/// __p__environ — return a pointer to the environment variable array.
///
/// Returns a pointer to a NULL pointer (minimal stub).
#[allow(non_snake_case, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn __p__environ() -> *const *const *const i8 {
    initenv_ptr() as *const *const *const i8
}

/// __p__fmode — return a pointer to the global file mode variable.
///
/// Returns the same pointer as `_fmode()`.
#[allow(non_snake_case, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn __p__fmode() -> *mut i32 {
    fmode_ptr()
}

/// __p__commode — return a pointer to the global commit mode variable.
///
/// Returns the same pointer as `_commode()`.
#[allow(non_snake_case, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn __p__commode() -> *mut i32 {
    commode_ptr()
}
