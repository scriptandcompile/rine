//! MSVCRT CRT support functions: exception handling, signal, locks, file descriptors.

use rine_common_msvcrt as common;

/// An internal function used at startup to tell the CRT what type of application we're running (console, GUI, etc).
///
/// # Arguments
/// * `app_type`: An integer representing the application type. The CRT uses this to configure its behavior accordingly.
///   The specific values and their meanings are defined by the CRT, but common values include:
///   0 = _crt_unknown_app - the CRT couldn't determine the app type, so it defaults to console behavior.
///   1 = _crt_console_app - a standard console application with stdin/stdout/stderr and a console window.
///   2 = _crt_gui_app - a GUI application without a console window; stdin/stdout/stderr may be redirected to files or pipes.
///   3 = _crt_cui_app - a character-mode application that may or may not have a console window; used for things like Windows Services.
///   4 = _crt_app_type_max - a sentinel value indicating the maximum valid app type.
///
/// # Safety
/// Called by the CRT intitialization code and unknown values may cause undefined behavior.
///
/// # Note
/// This is called by the CRT initialization code before `main()` runs. We currently ignore the app type since
/// we always run as a console application. This now at least stores the app type in a variable, but we don't
/// actually use it for anything yet.
#[rine_dlls::partial]
pub unsafe extern "C" fn __set_app_type(app_type: i32) {
    tracing::trace!(app_type, "msvcrt::__set_app_type");
    let app = app_type.into();
    common::set_app_type(app);
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
#[rine_dlls::stubbed]
pub unsafe extern "C" fn __setusermatherr(handler: usize) {
    tracing::trace!(handler, "msvcrt::__setusermatherr");
    common::set_user_math_err(handler);
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
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
pub unsafe extern "C" fn __C_specific_handler(
    _exception_record: usize,
    _establisher_frame: usize,
    _context_record: usize,
    _dispatcher_context: usize,
) -> i32 {
    tracing::trace!("msvcrt::__C_specific_handler");
    common::c_specific_handler_result(
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
#[rine_dlls::implemented]
#[cfg_attr(not(rust_analyzer), rine_dlls::data_export)]
pub unsafe extern "C" fn _commode() -> *mut i32 {
    common::commode_ptr()
}

/// Gets a pointer to the file mode variable.
///
/// # Safety
/// This is unsafe because the CRT expects this to return a valid pointer to an integer variable that controls CRT behavior.
/// Incorrect handling could lead to undefined behavior in CRT functions that access this variable.
///
/// # Returns
/// A pointer to the file mode variable, which controls how the CRT handles file buffering and flushing.
///
/// # Notes
/// This is called by CRT implementations to get a pointer to the file mode variable.
/// We return a pointer to a variable in our data cell module.
/// In a production implementation, this would be a properly implemented variable that controls CRT behavior.
/// Currently, this is just a stub that returns a pointer to a variable that is not actually used.
#[rine_dlls::implemented]
#[cfg_attr(not(rust_analyzer), rine_dlls::data_export)]
pub unsafe extern "C" fn _fmode() -> *mut i32 {
    common::fmode_ptr()
}

/// Get a pointer to the CRT's internal array of three FILE structures for stdin, stdout, and stderr.
///
/// # Safety
/// This is unsafe because the CRT expects this to return a valid pointer to an integer variable that controls CRT behavior.
/// Incorrect handling could lead to undefined behavior in CRT functions that access this variable.
///
/// # Returns
/// A pointer to an array of three FILE structures expected by the CRT for standard I/O operations.
/// The CRT expects this to be exported as `_iob` and used by functions like `printf` and `fprintf`.
#[rine_dlls::implemented]
#[cfg_attr(not(rust_analyzer), rine_dlls::data_export)]
pub unsafe extern "C" fn _iob() -> *mut u8 {
    common::fake_iob_32_ptr()
}

/// Get a pointer to the environment variable array.
///
/// # Safety
/// This is unsafe because the CRT expects this to return a valid pointer to an array of
/// C strings representing the environment variables.
/// Incorrect handling could lead to undefined behavior in CRT functions that access environment variables.
///
/// # Returns
/// Returns a pointer to the environment variable array, which is an array of C strings (char*).
///
/// # Notes
/// Called by the CRT to get the environment variables. We return a pointer to an empty environment
/// since we provide the real environment via `__getmainargs`.
/// This should return a pointer to the actual environment variables.
#[rine_dlls::implemented]
#[cfg_attr(not(rust_analyzer), rine_dlls::data_export)]
pub unsafe extern "C" fn __initenv() -> *mut usize {
    common::initenv_ptr()
}

/// Gets a pointer to the CRT's internal array of three FILE structures for stdin, stdout, and stderr.
///
/// # Safety
/// This is unsafe because the CRT expects this to return a valid pointer to an array of three FILE structures with a specific layout.
/// Incorrect handling could lead to undefined behavior in CRT functions that perform standard I/O operations.
///
/// # Returns
/// A pointer to an array of three FILE structures expected by the CRT for standard I/O operations.
/// The CRT expects this to be exported as `_iob` and used by functions like `printf` and `fprintf`.
/// The first 3 entries represent stdin (0), stdout (1), stderr (2).
/// We store a marker fd in the first field of each entry so fwrite/fprintf can identify the stream.
#[rine_dlls::implemented]
pub unsafe extern "C" fn __iob_func() -> *mut u8 {
    tracing::trace!("msvcrt::__iob_func");
    common::fake_iob_32_ptr()
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
#[rine_dlls::implemented]
pub unsafe extern "C" fn _onexit(func: usize) -> usize {
    tracing::trace!(func, "msvcrt::_onexit");
    common::onexit(func)
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
#[rine_dlls::implemented]
pub unsafe extern "C" fn _amsg_exit(msg_num: i32) {
    tracing::trace!(msg_num, "msvcrt::_amsg_exit");
    common::_amsg_exit(msg_num)
}

/// Abort the process immediately without unwinding or running exit handlers.
///
/// # Safety
/// This is unsafe because it will terminate the process immediately without running any cleanup code or exit handlers.
/// It should only be called in situations where the process is in an unrecoverable state and cannot continue safely.
///
/// # Notes
/// This is a stub implementation that just calls `std::process::abort()`
#[rine_dlls::implemented]
pub unsafe extern "C" fn abort() {
    tracing::error!("msvcrt::abort");
    common::abort_process()
}

/// Set a signal handler for the specified signal.
///
/// # Arguments
/// * `sig`: The signal number to set the handler for.
/// * `handler`: A pointer to the signal handler function to be called when the signal is raised.
///
/// # Safety
/// This is unsafe because the CRT expects the handler pointer to be valid and follow the correct calling convention.
/// Registering an invalid handler could cause undefined behavior when the signal is raised.
///
/// # Notes
/// Delegates to the common platform signal implementation.
#[rine_dlls::implemented]
pub unsafe extern "C" fn signal(sig: i32, handler: usize) -> usize {
    tracing::trace!(sig, handler, "msvcrt::signal");
    common::signal(sig, handler)
}

/// Acquire a CRT lock for the specified lock number.
///
/// # Arguments
/// * `locknum`: The lock number to acquire. The CRT uses this to synchronize access to internal resources.
///
/// # Safety
/// This is unsafe because the CRT expects locks to be properly acquired and released to avoid deadlocks and ensure thread safety.
/// Incorrect usage could lead to undefined behavior when multiple threads access CRT resources.
#[rine_dlls::implemented]
pub unsafe extern "C" fn _lock(locknum: i32) {
    tracing::trace!(locknum, "msvcrt::_lock");
    common::lock(locknum);
}

/// Release a CRT lock for the specified lock number.
///
/// # Arguments
/// * `locknum`: The lock number to release. This should match a previously acquired lock number.
///
/// # Safety
/// This is unsafe because the CRT expects locks to be properly acquired and released to avoid deadlocks and ensure thread safety.
/// Incorrect usage (like unlocking a lock that wasn't acquired) could lead to undefined behavior when multiple
/// threads access CRT resources.
#[rine_dlls::implemented]
pub unsafe extern "C" fn _unlock(locknum: i32) {
    tracing::trace!(locknum, "msvcrt::_unlock");
    common::unlock(locknum);
}

/// Get a pointer to the thread-local `errno` value.
///
/// # Safety
/// This is unsafe because the CRT expects this to return a valid pointer to a thread-local variable that holds
/// the error code for the last failed system call.
///
/// # Returns
/// A pointer to the thread-local `errno` variable.
/// The CRT and C code will read and write to this variable to get and set the error code for the last failed system call.
///
/// # Notes
/// Called by the `errno` macro or by C code that wants to read/write
/// the error code from the last failed system call.
#[rine_dlls::implemented]
pub unsafe extern "C" fn _errno() -> *mut i32 {
    tracing::trace!("msvcrt::_errno");
    common::errno_location()
}

/// Get a pointer to the environment variable array.
///
/// # Safety
/// This is unsafe because the CRT expects this to return a valid pointer to an array of
/// C strings representing the environment variables.
/// Incorrect handling could lead to undefined behavior in CRT functions that access environment variables.
///
/// # Returns
/// Returns a pointer to the environment variable array, which is an array of C strings (char*).
///
/// # Notes
/// Called by the CRT to get the environment variables. We return a pointer to an empty environment
/// since we provide the real environment via `__getmainargs`.
/// This should return a pointer to the actual environment variables.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
pub unsafe extern "C" fn __p__environ() -> *const *const *const i8 {
    tracing::trace!("msvcrt::__p__environ");
    common::initenv_ptr() as *const *const *const i8
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    static ONEXIT_CALLS: AtomicUsize = AtomicUsize::new(0);

    unsafe extern "C" fn test_onexit_callback() -> i32 {
        ONEXIT_CALLS.fetch_add(1, Ordering::SeqCst);
        0
    }

    #[test]
    fn onexit_registers_and_runs_callback_via_cexit() {
        ONEXIT_CALLS.store(0, Ordering::SeqCst);

        let callback_ptr = test_onexit_callback as *const () as usize;
        let registered = unsafe { super::_onexit(callback_ptr) };
        assert_eq!(registered, callback_ptr);

        unsafe { crate::stdlib::_cexit() };
        assert_eq!(ONEXIT_CALLS.load(Ordering::SeqCst), 1);
    }
}

/// Get a pointer to the file translation mode flag.
///
/// # Safety
/// This is unsafe because the CRT expects this to return a valid pointer to a global variable with a specific layout.
/// Incorrect handling could lead to undefined behavior in CRT functions that access this variable.
///
/// # Returns
/// A pointer to the file mode variable, which controls how the CRT handles file buffering and flushing.
///
/// # Notes
/// Returned pointer points to a mutable `int` that controls whether
/// text files are opened in binary or text mode by default.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
pub unsafe extern "C" fn __p__fmode() -> *mut i32 {
    tracing::trace!("msvcrt::__p__fmode");
    common::fmode_ptr()
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
#[rine_dlls::implemented]
#[allow(non_snake_case)]
pub unsafe extern "C" fn __p__commode() -> *mut i32 {
    tracing::trace!("msvcrt::__p__commode");
    common::commode_ptr()
}
