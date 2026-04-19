//! MSVCRT CRT support functions needed during CRT startup.
//!
//! These are called by the MinGW CRT startup code before `main()` runs.
//! Most are no-ops or minimal stubs.

use rine_common_msvcrt::{
    abort_process, amsg_exit, c_specific_handler_result, commode_ptr, errno_location,
    fake_iob_64_ptr, fmode_ptr, initenv_ptr, lock, onexit, set_app_type, set_user_math_err,
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
pub unsafe extern "win64" fn __set_app_type(app_type: i32) {
    tracing::trace!("msvcrt::__set_app_type({app_type})");
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
pub unsafe extern "win64" fn __setusermatherr(handler: usize) {
    tracing::trace!("msvcrt::__setusermatherr");
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
pub unsafe extern "win64" fn __C_specific_handler(
    _exception_record: usize,
    _establisher_frame: usize,
    _context_record: usize,
    _dispatcher_context: usize,
) -> i32 {
    tracing::warn!("msvcrt::__C_specific_handler called — exceptions not supported");
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
pub unsafe extern "win64" fn _commode() -> *mut i32 {
    commode_ptr()
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
pub unsafe extern "win64" fn __p__commode() -> *mut i32 {
    commode_ptr()
}

/// _fmode — return a pointer to the default file translation mode.
#[allow(clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn _fmode() -> *mut i32 {
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
pub unsafe extern "win64" fn _onexit(func: usize) -> usize {
    tracing::trace!("msvcrt::_onexit");
    onexit(func)
}

/// Prints an error message and exit the process. Called internally by the CRT when fatal errors occur.
///
/// # Arguments
/// * `msg_num`: An integer error code representing the specific error that occurred.
///   The CRT uses this to determine which error message to display.
///   The specific values and their meanings are defined by the CRT, but common values include:
///   1 = _RT_ASSERT
///   2 = _RT_ERROR
///   3 = _RT_INVALID_PARAM
///   4 = _RT_HEAP_ERROR
///   5 = _RT_TERM_SIGNAL
///   6 = _RT_CTRL_CLOSE
///   7 = _RT_CTRL_BREAK
///   8 = _RT_CTRL_CONTROL
///   9 = _RT_CTRL_LOGOFF
///   10 = _RT_CTRL_SHUTDOWN
///   11 = _RT_ASSERT_REPORT_WIDE
///   12 = _RT_ASSERT_REPORT_UNICODE
///   13 = _RT_ASSERT_REPORT_STDERR
///   14 = _RT_ASSERT_REPORT_FILE
///   15 = _RT_ASSERT_REPORT_THREAD
///   16 = _RT_ASSERT_REPORT_THREAD_FILE
///   17 = _RT_ASSERT_REPORT_THREAD_STDERR
///   18 = _RT_ASSERT_REPORT_THREAD_WIDE
///   19 = _RT_ASSERT_REPORT_THREAD_UNICODE
///   20 = _RT_ASSERT_REPORT_FILE_WIDE
///   21 = _RT_ASSERT_REPORT_FILE_UNICODE
///   22 = _RT_ASSERT_REPORT_STDERR_WIDE
///   23 = _RT_ASSERT_REPORT_STDERR_UNICODE
///   24 = _RT_ASSERT_REPORT_THREAD_FILE_WIDE
///   25 = _RT_ASSERT_REPORT_THREAD_FILE_UNICODE
///   26 = _RT_ASSERT_REPORT_THREAD_STDERR_WIDE
///   27 = _RT_ASSERT_REPORT_THREAD_STDERR_UNICODE
///   28 = _RT_ASSERT_REPORT_THREAD_WIDE
///   29 = _RT_ASSERT_REPORT_THREAD_UNICODE
///   30 = _RT_ASSERT_REPORT_FILE_WIDE
///   31 = _RT_ASSERT_REPORT_FILE_UNICODE
///   32 = _RT_ASSERT_REPORT_STDERR_WIDE
///   33 = _RT_ASSERT_REPORT_STDERR_UNICODE
///
/// # Safety
/// This is unsafe because it will terminate the process and should only be called by the CRT when a fatal error occurs.
/// Calling this function will cause the process to exit immediately, so it should be used with caution.
///
/// # Notes
/// This is called by the CRT when fatal errors occur.
/// We currently just print a message and abort the process, but a production implementation would display a message
/// box with the error and possibly allow the user to choose whether to abort or debug.
/// The `msg_num` argument can be used to determine the specific error that occurred and display an appropriate message.
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
/// No-op for single-threaded.
#[allow(clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn _lock(locknum: i32) {
    lock(locknum);
}

/// _unlock — release an internal CRT lock.
///
/// No-op for single-threaded.
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
