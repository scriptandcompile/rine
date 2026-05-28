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

static WINITENV_PTR: LazyLock<SyncPtr<*mut usize>> = LazyLock::new(|| SyncPtr(leaked_usize(0)));

/// Get a pointer to the wide-character environment variable array.
///
/// # Safety
/// This is unsafe because the CRT expects this to return a valid pointer to an array of
/// wide C strings representing the environment variables.
/// Incorrect handling could lead to undefined behavior in CRT functions that access environment variables.
///
/// # Returns
/// Returns a pointer to the wide-character environment variable array, which is an array of
/// wide C strings (wchar_t*).
///
/// # Notes
/// Called by the CRT to get the wide environment variables via `__winitenv`. We return a pointer to an
/// empty environment since we provide the real wide environment via `__wgetmainargs`.
/// This should return a pointer to the actual wide environment variables populated from `_wenviron`.
pub fn winitenv_ptr() -> *mut usize {
    WINITENV_PTR.0
}

static WCMDLN_STRING: LazyLock<Vec<u16>> = LazyLock::new(|| {
    let args: Vec<String> = std::env::args().collect();
    let joined = args
        .iter()
        .map(|a| {
            if a.contains(' ') {
                format!("\"{a}\"")
            } else {
                a.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    joined.encode_utf16().chain(std::iter::once(0)).collect()
});

static WCMDLN_PTR: LazyLock<SyncPtr<*mut usize>> = LazyLock::new(|| {
    let ptr = wcmdln_str() as usize;
    SyncPtr(leaked_usize(ptr))
});

/// Get a pointer to the wide-character command-line string.
///
/// # Returns
/// A pointer to a null-terminated UTF-16LE command line string,
/// reconstructed from `std::env::args()` with space-containing arguments
/// quoted. The returned pointer is valid for the lifetime of the process.
///
/// # Notes
/// This provides the backing data for the `_wcmdln` export in msvcrt.dll.
/// The command line is built once on first access and cached.
/// The reconstruction from argv may differ from the original command line
/// provided by the OS (e.g., quoting, whitespace normalization).
/// Wildcard expansion is not performed.
pub fn wcmdln_str() -> *const u16 {
    WCMDLN_STRING.as_ptr()
}

/// Get the address of the `_wcmdln` data cell.
///
/// `_wcmdln` in msvcrt is a data export with type `wchar_t*`, so importers
/// expect the IAT slot to point at storage that contains a wide-string pointer.
pub fn wcmdln_ptr() -> *mut usize {
    WCMDLN_PTR.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wcmdln_str_returns_non_null() {
        let ptr = wcmdln_str();
        assert!(!ptr.is_null());
    }

    #[test]
    fn wcmdln_str_is_null_terminated() {
        let ptr = wcmdln_str();
        assert!(!ptr.is_null());
        unsafe {
            let mut len = 0;
            while *ptr.add(len) != 0 {
                len += 1;
            }
            assert!(len > 0, "command line should not be empty");
        }
    }
}
