//! MSVCRT locale functions: `___lc_codepage_func`.

use rine_common_msvcrt as common;

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
#[rine_dlls::implemented]
pub unsafe extern "win64" fn ___lc_codepage_func() -> *mut u32 {
    common::___lc_codepage_func()
}
