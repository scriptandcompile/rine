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

/// Returns the maximum number of bytes in a multibyte character for the current locale.
///
/// # Returns
/// The maximum byte length of a multibyte character:
/// - `1` for single-byte code pages (e.g., 1252, 437)
/// - `2` for DBCS code pages (e.g., 932 Shift-JIS, 936 GBK, 949 Korean, 950 Big5, 1361 Johab)
/// - `4` for UTF-8 (65001)
///
/// # Notes
/// This is a partial/stub implementation derived from `___lc_codepage_func()`.
/// In a full implementation, the return value would directly reflect the per-thread
/// locale's `_locinfo->mb_cur_max` field, and would change when `setlocale()` is called
/// with a multibyte locale. Per-thread locale changes are not yet reflected.
pub fn ___mb_cur_max_func() -> i32 {
    let code_page = unsafe { *___lc_codepage_func() };
    mb_cur_max_for_code_page(code_page)
}

/// Maps a resolved code page identifier to its maximum multibyte character length.
fn mb_cur_max_for_code_page(code_page: u32) -> i32 {
    match code_page {
        65001 => 4,
        932 | 936 | 949 | 950 | 1361 => 2,
        _ => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- ___lc_codepage_func tests --

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

    // -- ___mb_cur_max_func tests --

    #[test]
    fn default_locale_is_single_byte() {
        assert_eq!(___mb_cur_max_func(), 1);
    }

    #[test]
    fn single_byte_code_pages_return_1() {
        assert_eq!(mb_cur_max_for_code_page(1252), 1);
        assert_eq!(mb_cur_max_for_code_page(437), 1);
        assert_eq!(mb_cur_max_for_code_page(28591), 1);
    }

    #[test]
    fn dbcs_code_pages_return_2() {
        assert_eq!(mb_cur_max_for_code_page(932), 2);
        assert_eq!(mb_cur_max_for_code_page(936), 2);
        assert_eq!(mb_cur_max_for_code_page(949), 2);
        assert_eq!(mb_cur_max_for_code_page(950), 2);
        assert_eq!(mb_cur_max_for_code_page(1361), 2);
    }

    #[test]
    fn utf8_code_page_returns_4() {
        assert_eq!(mb_cur_max_for_code_page(65001), 4);
    }
}
