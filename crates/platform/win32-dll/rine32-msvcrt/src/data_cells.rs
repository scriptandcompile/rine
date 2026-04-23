use rine_common_msvcrt as common;

/// Gets a pointer to the commit mode variable.
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
pub unsafe extern "C" fn _commode() -> *mut i32 {
    common::commode_ptr()
}

/// Gets a pointer to the file mode variable.
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
pub unsafe extern "C" fn _fmode() -> *mut i32 {
    common::fmode_ptr()
}

/// Get a pointer to the CRT's internal array of three FILE structures for stdin, stdout, and stderr.
///
/// # Returns
/// A pointer to an array of three FILE structures expected by the CRT for standard I/O operations.
/// The CRT expects this to be exported as `_iob` and used by functions like `printf` and `fprintf`.
#[rine_dlls::implemented]
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
pub unsafe extern "C" fn __initenv() -> *mut usize {
    common::initenv_ptr()
}
