use std::sync::LazyLock;

struct SyncPtr<T>(T);

unsafe impl<T> Sync for SyncPtr<T> {}
unsafe impl<T> Send for SyncPtr<T> {}

fn leaked_i32(initial: i32) -> *mut i32 {
    Box::into_raw(Box::new(initial))
}

fn leaked_usize(initial: usize) -> *mut usize {
    Box::into_raw(Box::new(initial))
}

static COMMODE_PTR: LazyLock<SyncPtr<*mut i32>> = LazyLock::new(|| SyncPtr(leaked_i32(0)));
static FMODE_PTR: LazyLock<SyncPtr<*mut i32>> = LazyLock::new(|| SyncPtr(leaked_i32(0)));
static INITENV_PTR: LazyLock<SyncPtr<*mut usize>> = LazyLock::new(|| SyncPtr(leaked_usize(0)));

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
pub fn commode_ptr() -> *mut i32 {
    COMMODE_PTR.0
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
pub fn fmode_ptr() -> *mut i32 {
    FMODE_PTR.0
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
pub fn initenv_ptr() -> *mut usize {
    INITENV_PTR.0
}
