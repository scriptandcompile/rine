use rine_common_kernel32::locale as common;
use rine_types::locale::{LCID, LCTYPE};
use rine_types::strings::LPSTR;

/// Get locale-specific information in ANSI form.
///
/// # Arguments
/// * `_locale` - The locale identifier. This implementation currently ignores locale selection and always returns en-US style values.
/// * lc_type - The locale information type (`LCTYPE`) to query.
/// * lc_data - Output buffer for a null-terminated ANSI string.
/// * cch_data - Size of `lc_data` in chars; if zero, the required size (including null terminator) is returned.
///
/// # Safety
/// If `cch_data > 0`, `lc_data` must point to a writable buffer of at least `cch_data` bytes.
///
/// # Return
/// Number of chars written including the null terminator on success; required size when `cch_data == 0`; `0` on failure.
///
/// # Notes
/// Only a subset of `LCTYPE` constants is currently supported.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetLocaleInfoA(
    locale: LCID,
    lc_type: LCTYPE,
    lc_data: LPSTR,
    cch_data: i32,
) -> i32 {
    unsafe { common::get_locale_info_a(locale, lc_type, lc_data, cch_data) }
}
