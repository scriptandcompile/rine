//! MSVCRT locale functions: `localeconv`, `___lc_codepage_func`, `___mb_cur_max_func`.

use rine_common_msvcrt as common;

/// Returns a pointer to the current locale's `lconv` structure describing
/// numeric and monetary formatting conventions.
///
/// # Arguments
/// None.
///
/// # Returns
/// A pointer to a process-lifetime static `lconv`. The caller must not modify
/// the returned structure or the strings it points to.
///
/// # Safety
/// The returned pointer is valid for the lifetime of the process. Dereferencing
/// it is safe, but writing to the structure or any pointed-to string induces
/// undefined behavior in subsequent locale-dependent CRT functions.
///
/// # Notes
/// This is a partial/stub implementation that always returns the C locale
/// conventions. Monetary fields are `CHAR_MAX` or empty strings. Per-thread
/// locale changes and system-locale queries are not yet implemented.
#[rine_dlls::partial]
pub unsafe extern "win64" fn localeconv() -> *mut rine_types::locale::lconv {
    common::localeconv()
}

/// Returns a pointer to the current locale's code page.
///
/// # Arguments
/// None.
///
/// # Returns
/// A pointer to an unsigned integer containing the current code page identifier
/// (e.g., 1252 for Western European, 932 for Shift-JIS, 65001 for UTF-8).
///
/// # Safety
/// The returned pointer is valid for the lifetime of the process. Dereferencing
/// the pointer is safe, but writing an arbitrary code page value may cause
/// undefined behavior in CRT locale-dependent functions (e.g., character
/// classification, multibyte-to-wide conversion).
///
/// # Notes
/// This is a partial/stub implementation backed by a process-wide static value.
/// Per-thread locale changes via `setlocale()` or `_configthreadlocale()` are
/// not yet reflected.
#[rine_dlls::partial]
pub unsafe extern "win64" fn ___lc_codepage_func() -> *mut u32 {
    common::___lc_codepage_func()
}

/// Returns the maximum number of bytes in a multibyte character for the current locale.
///
/// # Arguments
/// None.
///
/// # Returns
/// The maximum byte length of a multibyte character:
/// - `1` for single-byte code pages (e.g., 1252, 437)
/// - `2` for DBCS code pages (e.g., 932 Shift-JIS, 936 GBK, 949 Korean, 950 Big5, 1361 Johab)
/// - `4` for UTF-8 (65001)
///
/// # Safety
/// This function reads the current locale's code page from a process-wide static
/// value. The underlying pointer is valid for the lifetime of the process.
///
/// # Notes
/// This is a partial/stub implementation backed by a process-wide static value.
/// Per-thread locale changes via `setlocale()` or `_configthreadlocale()` are
/// not yet reflected.
#[rine_dlls::partial]
pub unsafe extern "win64" fn ___mb_cur_max_func() -> i32 {
    common::___mb_cur_max_func()
}
