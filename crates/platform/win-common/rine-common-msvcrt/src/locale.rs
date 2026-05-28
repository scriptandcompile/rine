use rine_types::locale::lconv;
use std::ffi::CString;
use std::sync::LazyLock;

const DEFAULT_ANSI_CODE_PAGE: u32 = 1252;

/// Value of `CHAR_MAX` (127) used in `lconv` numeric fields to indicate
/// "not available" in the C locale.
const CHAR_MAX: libc::c_char = 127;

/// Leaks a `CString` and returns a mutable C pointer valid for the process
/// lifetime. Used to initialize the static `lconv` strings.
fn make_c_str(s: &str) -> *mut libc::c_char {
    CString::new(s).unwrap().into_raw() as *mut libc::c_char
}

/// Static `lconv` describing the C locale conventions. Leaked string
/// buffers live for the process lifetime.
static LCONV: LazyLock<lconv> = LazyLock::new(|| lconv {
    decimal_point: make_c_str("."),
    thousands_sep: make_c_str(""),
    grouping: make_c_str(""),
    int_curr_symbol: make_c_str(""),
    currency_symbol: make_c_str(""),
    mon_decimal_point: make_c_str(""),
    mon_thousands_sep: make_c_str(""),
    mon_grouping: make_c_str(""),
    positive_sign: make_c_str(""),
    negative_sign: make_c_str(""),
    int_frac_digits: CHAR_MAX,
    frac_digits: CHAR_MAX,
    p_cs_precedes: CHAR_MAX,
    p_sep_by_space: CHAR_MAX,
    n_cs_precedes: CHAR_MAX,
    n_sep_by_space: CHAR_MAX,
    p_sign_posn: CHAR_MAX,
    n_sign_posn: CHAR_MAX,
    int_n_cs_precedes: CHAR_MAX,
    int_n_sep_by_space: CHAR_MAX,
    int_n_sign_posn: CHAR_MAX,
    int_p_cs_precedes: CHAR_MAX,
    int_p_sep_by_space: CHAR_MAX,
    int_p_sign_posn: CHAR_MAX,
});

struct SyncPtr<T>(T);

unsafe impl<T> Sync for SyncPtr<T> {}
unsafe impl<T> Send for SyncPtr<T> {}

static LC_CODEPAGE: LazyLock<SyncPtr<*mut u32>> =
    LazyLock::new(|| SyncPtr(Box::into_raw(Box::new(DEFAULT_ANSI_CODE_PAGE))));

/// Returns a pointer to the current locale's `lconv` structure, which describes
/// numeric and monetary formatting conventions (e.g., decimal point, thousands
/// separator, currency symbol).
///
/// # Returns
/// A pointer to a process-lifetime static `lconv` structure. The caller should
/// not modify the returned structure or the strings it points to; doing so
/// results in undefined behavior.
///
/// # Notes
/// This is a partial/stub implementation that always returns the C locale
/// conventions. In the C locale:
/// - `decimal_point` is `"."`
/// - All monetary fields (`currency_symbol`, `mon_decimal_point`, etc.) are
///   empty strings or `CHAR_MAX` (127), indicating "not available".
/// - No thousands separator or grouping is applied.
///
/// TODO: Reflect per-thread locale changes made via `setlocale()` or
/// `_configthreadlocale()`. Query the system locale for formatting values
/// (decimal separator, digit grouping, currency symbol) instead of returning
/// the minimal C locale defaults.
pub fn localeconv() -> *mut lconv {
    // SAFETY: LCONV is a process-lifetime static; the returned pointer is
    // stable and the struct is never deallocated. The const-to-mut cast
    // matches MSVCRT's `localeconv` signature (`struct lconv *`).
    std::ptr::from_ref(&*LCONV).cast_mut()
}

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

    // -- localeconv tests --

    #[test]
    fn localeconv_returns_non_null() {
        let ptr = localeconv();
        assert!(!ptr.is_null());
    }

    #[test]
    fn localeconv_pointer_is_stable() {
        let ptr1 = localeconv();
        let ptr2 = localeconv();
        assert_eq!(ptr1, ptr2);
    }

    #[test]
    fn localeconv_decimal_point_is_period() {
        let lc = unsafe { &*localeconv() };
        assert!(!lc.decimal_point.is_null());
        let s = unsafe { std::ffi::CStr::from_ptr(lc.decimal_point) };
        assert_eq!(s.to_bytes(), b".");
    }

    #[test]
    fn thousands_sep_is_empty_in_c_locale() {
        let lc = unsafe { &*localeconv() };
        assert!(!lc.thousands_sep.is_null());
        let s = unsafe { std::ffi::CStr::from_ptr(lc.thousands_sep) };
        assert!(s.to_bytes().is_empty());
    }

    #[test]
    fn grouping_is_empty_in_c_locale() {
        let lc = unsafe { &*localeconv() };
        assert!(!lc.grouping.is_null());
        let s = unsafe { std::ffi::CStr::from_ptr(lc.grouping) };
        assert!(s.to_bytes().is_empty());
    }

    #[test]
    fn monetary_strings_are_empty_in_c_locale() {
        let lc = unsafe { &*localeconv() };
        for field in [
            lc.int_curr_symbol,
            lc.currency_symbol,
            lc.mon_decimal_point,
            lc.mon_thousands_sep,
            lc.mon_grouping,
            lc.positive_sign,
            lc.negative_sign,
        ] {
            assert!(!field.is_null());
            let s = unsafe { std::ffi::CStr::from_ptr(field) };
            assert!(s.to_bytes().is_empty(), "expected empty, got {:?}", s);
        }
    }

    #[test]
    fn monetary_char_fields_are_char_max_in_c_locale() {
        let lc = unsafe { &*localeconv() };
        assert_eq!(lc.int_frac_digits, CHAR_MAX);
        assert_eq!(lc.frac_digits, CHAR_MAX);
        assert_eq!(lc.p_cs_precedes, CHAR_MAX);
        assert_eq!(lc.p_sep_by_space, CHAR_MAX);
        assert_eq!(lc.n_cs_precedes, CHAR_MAX);
        assert_eq!(lc.n_sep_by_space, CHAR_MAX);
        assert_eq!(lc.p_sign_posn, CHAR_MAX);
        assert_eq!(lc.n_sign_posn, CHAR_MAX);
        assert_eq!(lc.int_n_cs_precedes, CHAR_MAX);
        assert_eq!(lc.int_n_sep_by_space, CHAR_MAX);
        assert_eq!(lc.int_n_sign_posn, CHAR_MAX);
        assert_eq!(lc.int_p_cs_precedes, CHAR_MAX);
        assert_eq!(lc.int_p_sep_by_space, CHAR_MAX);
        assert_eq!(lc.int_p_sign_posn, CHAR_MAX);
    }
}
