use std::sync::LazyLock;

const DEFAULT_ANSI_CODE_PAGE: u32 = 1252;

struct SyncPtr<T>(T);

unsafe impl<T> Sync for SyncPtr<T> {}
unsafe impl<T> Send for SyncPtr<T> {}

static LC_CODEPAGE: LazyLock<SyncPtr<*mut u32>> =
    LazyLock::new(|| SyncPtr(Box::into_raw(Box::new(DEFAULT_ANSI_CODE_PAGE))));

/// Returns a pointer to the current locale's code page.
///
/// # Returns
/// A pointer to an unsigned integer containing the current code page identifier
/// (e.g., 1252 for Western European, 932 for Shift-JIS, 65001 for UTF-8).
/// The value is initially the system default ANSI code page (1252).
///
/// # Notes
/// This is a partial/stub implementation that returns a pointer to a process-wide
/// static value. In a full implementation, the returned pointer would point to
/// per-thread locale data that changes when `setlocale()` is called. Currently,
/// the code page value is fixed and cannot be modified via locale changes.
pub fn ___lc_codepage_func() -> *mut u32 {
    LC_CODEPAGE.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_non_null_pointer() {
        let ptr = ___lc_codepage_func();
        assert!(!ptr.is_null());
    }

    #[test]
    fn default_value_is_ansi_1252() {
        let ptr = ___lc_codepage_func();
        unsafe {
            assert_eq!(*ptr, DEFAULT_ANSI_CODE_PAGE);
        }
    }

    #[test]
    fn pointer_is_stable_across_calls() {
        let ptr1 = ___lc_codepage_func();
        let ptr2 = ___lc_codepage_func();
        assert_eq!(ptr1, ptr2);
    }
}
