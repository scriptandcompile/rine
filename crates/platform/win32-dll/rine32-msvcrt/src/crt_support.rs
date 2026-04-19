//! MSVCRT CRT support functions: exception handling, signal, locks, file descriptors.

use rine_common_msvcrt::{
    abort_process, amsg_exit, c_specific_handler_result, commode_ptr, errno_location,
    fake_iob_32_ptr, fmode_ptr, initenv_ptr, lock, onexit, set_app_type, set_user_math_err,
    signal_default, unlock,
};

/// An internal function used at startup to tell the CRT what type of application we're running (console, GUI, etc).
///
/// # Arguments
/// * `app_type`: An integer representing the application type. The CRT uses this to configure its behavior accordingly.
///   The specific values and their meanings are defined by the CRT, but common values include:
///   0 = _crt_unknown_app
///   1 = _crt_console_app
///   2 = _crt_gui_app
///   3 = _crt_cui_app
///   4 = _crt_app_type_max
///
/// # Safety
/// Called by the CRT intitialization code and unknown values may cause undefined behavior.
///
/// # Note
/// This is called by the CRT initialization code before `main()` runs. We currently ignore the app type since
/// we always run as a console application, but a production implementation would use this to configure CRT behavior accordingly.
/// Currently, this is just a no-op.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __set_app_type(app_type: i32) {
    tracing::trace!(app_type, "msvcrt::__set_app_type");
    set_app_type(app_type);
}

/// Set a custom math error handler.
///
/// # Arguments
/// * `handler`: A pointer to a user-defined math error handler function.
///   The CRT will call this function when a math error occurs (like divide-by-zero or overflow).
///
/// # Safety
/// This is unsafe because the handler must follow the correct calling convention and behavior expected by the CRT.
/// Installing an invalid handler could cause undefined behavior when math errors occur.
///
/// # Notes
/// This is a no-op currently; a production implementation would let the user install a handler for floating-point errors.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __setusermatherr(handler: usize) {
    tracing::trace!(handler, "msvcrt::__setusermatherr");
    set_user_math_err(handler);
}

/// Called by the CRT when a SEH exception is thrown.
///
/// # Arguments
/// * `_exception_record`: A pointer to an EXCEPTION_RECORD structure containing information about the exception.
/// * `_establisher_frame`: A pointer to the frame of the function where the exception occurred.
/// * `_context_record`: A pointer to a CONTEXT structure containing the CPU context at the time of the exception.
///
/// # Safety
/// This is called by the CRT when a SEH exception is thrown.
/// The arguments are pointers to CRT-defined structures with specific layouts,
/// and the function must return a valid handler code expected by the CRT.
/// Incorrect handling could lead to undefined behavior when exceptions occur.
///
/// # Returns
/// This is a stub currently that just returns "continue search" (1).
///
/// # Notes
/// This is called by the CRT when a SEH exception is thrown.
/// We don't support SEH exceptions currently, so this is just a stub that returns "continue search" (1) to
/// indicate that the CRT should call the next handler.
/// In a production implementation, this would analyze the exception record and return the appropriate handler code
/// (1 = continue execution, 0 = call next handler).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __C_specific_handler(
    _exception_record: usize,
    _establisher_frame: usize,
    _context_record: usize,
    _dispatcher_context: usize,
) -> i32 {
    tracing::trace!("msvcrt::__C_specific_handler");
    c_specific_handler_result(
        _exception_record,
        _establisher_frame,
        _context_record,
        _dispatcher_context,
    )
}

/// Gets a pointer to the commit mode variable.
///
/// # Safety
/// This is unsafe because the CRT expects this to return a valid pointer to a global variable with a specific layout.
/// Incorrect handling could lead to undefined behavior in CRT functions that access this variable.
///
/// # Returns
/// A pointer to the commit mode variable, which controls how the CRT handles file buffering and flushing.
///
/// # Notes
/// This is called by CRT implementations to get a pointer to the commit mode variable.
/// We return a pointer to a variable in our data cell module..
/// In a production implementation, this would be a properly implemented variable that controls CRT behavior.
/// Currently, this is just a stub that returns a pointer to a variable that is not actually used.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn _commode() -> *mut i32 {
    commode_ptr()
}

/// __iob_func — get the fake stdio FILE buffer table.
///
/// Returns a pointer to a table of three FILE-like structures for
/// stdin, stdout, stderr. The msvcrt DLL exposes these as `_iob` data.
#[allow(clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __iob_func() -> *mut u8 {
    tracing::trace!("msvcrt::__iob_func");
    fake_iob_32_ptr()
}

/// Register a function to be called at process exit.
///
/// # Arguments
/// * `func`: A pointer to a function that takes no arguments and returns void.
///   This function will be called when the process exits, either normally or via `exit()`.
///
/// # Safety
/// This is unsafe because the CRT expects the function pointer to be valid and follow the correct calling convention.
/// Registering an invalid function could cause undefined behavior when the process exits.
///
/// # Notes
/// This is currently a no-op that just returns the function pointer unchanged.
/// A production implementation would add it to an atexit chain and call it when the process exits.
/// Currently, this just returns the function pointer unchanged.
/// A production implementation would add it to an atexit chain.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn _onexit(func: usize) -> usize {
    tracing::trace!(func, "msvcrt::_onexit");
    onexit(func)
}

/// _amsg_exit — print an error message and exit the process.
///
/// Called internally by the CRT when fatal errors occur.
#[allow(clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn _amsg_exit(msg_num: i32) {
    tracing::trace!(msg_num, "msvcrt::_amsg_exit");
    amsg_exit(msg_num)
}

/// abort — raise SIGABRT and terminate the process.
///
/// Does not return.
#[allow(clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn abort() {
    tracing::error!("msvcrt::abort");
    abort_process()
}

/// signal — install a signal handler or get the current one.
///
/// Currently returns the previous handler.
#[allow(clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn signal(sig: i32, handler: usize) -> usize {
    tracing::trace!(sig, handler, "msvcrt::signal");
    signal_default(sig, handler)
}

/// _lock — acquire a CRT lock (for thread safety of stdio, malloc, etc.).
///
/// Currently, this is a no-op. A production implementation would use
/// actual OS synchronization primitives.
#[allow(clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn _lock(locknum: i32) {
    tracing::trace!(locknum, "msvcrt::_lock");
    lock(locknum);
}

/// _unlock — release a CRT lock.
///
/// Currently, this is a no-op.
#[allow(clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn _unlock(locknum: i32) {
    tracing::trace!(locknum, "msvcrt::_unlock");
    unlock(locknum);
}

/// _errno — get a pointer to the thread-local `errno` value.
///
/// Called by the `errno` macro or by C code that wants to read/write
/// the error code from the last failed system call.
#[allow(clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn _errno() -> *mut i32 {
    tracing::trace!("msvcrt::_errno");
    errno_location()
}

/// __p__environ — get a pointer to the environment pointer table.
///
/// Returns a triple pointer (for calling context compatibility).
#[allow(non_snake_case, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __p__environ() -> *const *const *const i8 {
    tracing::trace!("msvcrt::__p__environ");
    initenv_ptr() as *const *const *const i8
}

/// __p__fmode — get a pointer to the file translation mode flag.
///
/// Returned pointer points to a mutable `int` that controls whether
/// text files are opened in binary or text mode by default.
#[allow(non_snake_case, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __p__fmode() -> *mut i32 {
    tracing::trace!("msvcrt::__p__fmode");
    fmode_ptr()
}

/// Gets a pointer to the commit mode variable.
///
/// # Safety
/// This is unsafe because the CRT expects this to return a valid pointer to a global variable with a specific layout.
/// Incorrect handling could lead to undefined behavior in CRT functions that access this variable.
///
/// # Returns
/// A pointer to the commit mode variable, which controls how the CRT handles file buffering and flushing.
///
/// # Notes
/// This is called by CRT implementations to get a pointer to the commit mode variable.
/// We return a pointer to a variable in our data cell module..
/// In a production implementation, this would be a properly implemented variable that controls CRT behavior.
/// Currently, this is just a stub that returns a pointer to a variable that is not actually used.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __p__commode() -> *mut i32 {
    tracing::trace!("msvcrt::__p__commode");
    commode_ptr()
}
