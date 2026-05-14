use rine_types::locale::{LCID, LCTYPE};
use rine_types::strings::{LPSTR, write_cstr};
use tracing::warn;

// Locale values below are the subset currently needed by desktop apps like notepad.
const LOCALE_ILANGUAGE: u32 = 0x0000_0001;
const LOCALE_SLANGUAGE: u32 = 0x0000_0002;
const LOCALE_SABBREVLANGNAME: u32 = 0x0000_0003;
const LOCALE_SNATIVELANGNAME: u32 = 0x0000_0004;
const LOCALE_ICOUNTRY: u32 = 0x0000_0005;
const LOCALE_SCOUNTRY: u32 = 0x0000_0006;
const LOCALE_SABBREVCTRYNAME: u32 = 0x0000_0007;
const LOCALE_SNATIVECTRYNAME: u32 = 0x0000_0008;
const LOCALE_SLIST: u32 = 0x0000_000C;
const LOCALE_SDECIMAL: u32 = 0x0000_000E;
const LOCALE_STHOUSAND: u32 = 0x0000_000F;
const LOCALE_SCURRENCY: u32 = 0x0000_0014;
const LOCALE_SINTLSYMBOL: u32 = 0x0000_0015;
const LOCALE_ICURRDIGITS: u32 = 0x0000_0019;
const LOCALE_SSHORTDATE: u32 = 0x0000_001F;
const LOCALE_IDATE: u32 = 0x0000_0021;
const LOCALE_ITIME: u32 = 0x0000_0023;
const LOCALE_S1159: u32 = 0x0000_0028;
const LOCALE_S2359: u32 = 0x0000_0029;
const LOCALE_SENGLANGUAGE: u32 = 0x0000_1001;
const LOCALE_SENGCOUNTRY: u32 = 0x0000_1002;
const LOCALE_STIMEFORMAT: u32 = 0x0000_1003;
const LOCALE_IDEFAULTANSICODEPAGE: u32 = 0x0000_1004;

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
pub unsafe fn get_locale_info_a(
    _locale: LCID,
    lc_type: LCTYPE,
    lc_data: LPSTR,
    cch_data: i32,
) -> i32 {
    if cch_data < 0 {
        return 0;
    }

    let Some(value) = map_lctype_to_string(lc_type.0) else {
        warn!("GetLocaleInfoA: unsupported LCTYPE 0x{:08X}", lc_type.0);
        return 0;
    };

    let required = (value.len() + 1) as i32;

    if cch_data == 0 {
        return required;
    }

    if lc_data.is_null() || cch_data < required {
        return 0;
    }

    let written = unsafe { write_cstr(lc_data.as_mut_ptr(), cch_data as u32, value) };
    written as i32
}

fn map_lctype_to_string(lc_type: u32) -> Option<&'static str> {
    match lc_type {
        LOCALE_ILANGUAGE => Some("0409"),
        LOCALE_SLANGUAGE => Some("English (United States)"),
        LOCALE_SABBREVLANGNAME => Some("ENU"),
        LOCALE_SNATIVELANGNAME => Some("English"),
        LOCALE_ICOUNTRY => Some("1"),
        LOCALE_SCOUNTRY => Some("United States"),
        LOCALE_SABBREVCTRYNAME => Some("USA"),
        LOCALE_SNATIVECTRYNAME => Some("United States"),
        LOCALE_SLIST => Some(","),
        LOCALE_SDECIMAL => Some("."),
        LOCALE_STHOUSAND => Some(","),
        LOCALE_SCURRENCY => Some("$"),
        LOCALE_SINTLSYMBOL => Some("USD"),
        LOCALE_ICURRDIGITS => Some("2"),
        LOCALE_SSHORTDATE => Some("M/d/yyyy"),
        LOCALE_IDATE => Some("0"),
        LOCALE_ITIME => Some("0"),
        LOCALE_S1159 => Some("AM"),
        LOCALE_S2359 => Some("PM"),
        LOCALE_SENGLANGUAGE => Some("English"),
        LOCALE_SENGCOUNTRY => Some("United States"),
        LOCALE_STIMEFORMAT => Some("h:mm:ss tt"),
        LOCALE_IDEFAULTANSICODEPAGE => Some("1252"),
        _ => None,
    }
}
