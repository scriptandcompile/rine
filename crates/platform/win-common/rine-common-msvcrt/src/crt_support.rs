//! Common CRT support functions and data used by both 32-bit and 64-bit msvcrt implementations.
//! This includes functions that are expected by the CRT but not provided by the Windows API,
//! as well as shared data exports like `_commode` and `_fmode`.

use std::sync::LazyLock;

static FAKE_IOB_32: LazyLock<Box<[u8; 96]>> = LazyLock::new(build_fake_iob::<96, 32>);
static FAKE_IOB_64: LazyLock<Box<[u8; 144]>> = LazyLock::new(build_fake_iob::<144, 48>);

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
/// # Note
/// This is called by the CRT initialization code before `main()` runs. We currently ignore the app type since
/// we always run as a console application, but a production implementation would use this to configure CRT behavior accordingly.
/// Currently, this is just a no-op.
pub fn set_app_type(_app_type: i32) {}

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
pub fn set_user_math_err(_handler: usize) {}

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
pub fn c_specific_handler_result(
    _exception_record: usize,
    _establisher_frame: usize,
    _context_record: usize,
    _dispatcher_context: usize,
) -> i32 {
    1
}

/// Get a pointer to the CRT's internal array of three FILE structures for stdin, stdout, and stderr.
///
/// # Returns
/// A pointer to an array of three FILE structures expected by the CRT for standard I/O operations.
/// The CRT expects this to be exported as `_iob` and used by functions like `printf` and `fprintf`.
pub fn fake_iob_32_ptr() -> *mut u8 {
    FAKE_IOB_32.as_ptr() as *mut u8
}

/// Get a pointer to the CRT's internal array of three FILE structures for stdin, stdout, and stderr.
///
/// # Returns
/// A pointer to an array of three FILE structures expected by the CRT for standard I/O operations.
/// The CRT expects this to be exported as `_iob` and used by functions like `printf` and `fprintf`.
pub fn fake_iob_64_ptr() -> *mut u8 {
    FAKE_IOB_64.as_ptr() as *mut u8
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
pub fn onexit(func: usize) -> usize {
    func
}

/// Display a CRT error message and abort the process.
///
/// # Arguments
/// * `msg_num`: An integer representing the error message number. The CRT uses this to determine which error message to display.
///
/// # Notes
/// This is a stub implementation that just prints the message number and aborts the process.
pub fn amsg_exit(msg_num: i32) -> ! {
    eprintln!("rine: msvcrt runtime error (msg_num={msg_num})");
    std::process::abort();
}

/// Abort the process immediately without unwinding or running exit handlers.
///
/// # Safety
/// This is unsafe because it will terminate the process immediately without running any cleanup code or exit handlers.
/// It should only be called in situations where the process is in an unrecoverable state and cannot continue safely.
///
/// # Notes
/// This is a stub implementation that just calls `std::process::abort()`.
pub fn abort_process() -> ! {
    std::process::abort();
}

/// Set a signal handler for the specified signal.
///
/// # Arguments
/// * `_sig`: The signal number to set the handler for.
/// * `_handler`: A pointer to the signal handler function to be called when the signal is raised.
///
/// # Safety
/// This is unsafe because the CRT expects the handler pointer to be valid and follow the correct calling convention.
/// Registering an invalid handler could cause undefined behavior when the signal is raised.
///
/// # Notes
/// This is a stub implementation that does nothing and returns 0.
pub fn signal_default(_sig: i32, _handler: usize) -> usize {
    0
}

pub fn lock(_locknum: i32) {}

pub fn unlock(_locknum: i32) {}

pub fn errno_location() -> *mut i32 {
    unsafe { libc::__errno_location() }
}

/// Creates fake stdio control structures expected by the CRT,
/// since some CRT functions expect a pointer to an array of three FILE structures for stdin, stdout, and stderr.
fn build_fake_iob<const SIZE: usize, const ENTRY_SIZE: usize>() -> Box<[u8; SIZE]> {
    let mut buf = Box::new([0u8; SIZE]);
    buf[0..4].copy_from_slice(&0i32.to_ne_bytes());
    buf[ENTRY_SIZE..ENTRY_SIZE + 4].copy_from_slice(&1i32.to_ne_bytes());
    buf[ENTRY_SIZE * 2..ENTRY_SIZE * 2 + 4].copy_from_slice(&2i32.to_ne_bytes());
    buf
}

#[cfg(test)]
mod tests {
    use super::{fake_iob_32_ptr, fake_iob_64_ptr};

    #[test]
    fn fake_iob_32_has_expected_markers() {
        let ptr = fake_iob_32_ptr() as *const u8;
        let bytes = unsafe { std::slice::from_raw_parts(ptr, 96) };
        assert_eq!(&bytes[0..4], &0i32.to_ne_bytes());
        assert_eq!(&bytes[32..36], &1i32.to_ne_bytes());
        assert_eq!(&bytes[64..68], &2i32.to_ne_bytes());
    }

    #[test]
    fn fake_iob_64_has_expected_markers() {
        let ptr = fake_iob_64_ptr() as *const u8;
        let bytes = unsafe { std::slice::from_raw_parts(ptr, 144) };
        assert_eq!(&bytes[0..4], &0i32.to_ne_bytes());
        assert_eq!(&bytes[48..52], &1i32.to_ne_bytes());
        assert_eq!(&bytes[96..100], &2i32.to_ne_bytes());
    }
}
