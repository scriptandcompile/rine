use rine_types::os::{
    OsVersionInfoExW, OsVersionInfoW, SIZEOF_OSVERSIONINFOEXW, SIZEOF_OSVERSIONINFOW, get_version,
};
use rine_types::strings::{UnicodeString, wstr_unit_len};

/// Initialize a `UNICODE_STRING` structure with the given source string.
///
/// # Arguments
/// * `destination_string`: pointer to the `UNICODE_STRING` structure to initialize.
/// * `source_string`: pointer to a null-terminated wide string (PCWSTR) referenced by the `UNICODE_STRING`.
///
/// # Safety
/// All pointer parameters must be valid.
/// The `destination_string` must point to a valid `UNICODE_STRING` structure, and `source_string` must point
/// to a valid null-terminated wide string.
pub unsafe fn rtl_init_unicode_string(
    destination_string: *mut UnicodeString,
    source_string: *const u16,
) {
    if destination_string.is_null() {
        return;
    }

    if source_string.is_null() {
        unsafe {
            (*destination_string).length = 0;
            (*destination_string).maximum_length = 0;
            (*destination_string).buffer = core::ptr::null_mut();
        }
        return;
    }

    let source_units = unsafe { wstr_unit_len(source_string).unwrap_or(0) };
    let unit_size = core::mem::size_of::<u16>();

    let byte_len = source_units
        .saturating_mul(unit_size)
        .min(u16::MAX as usize) as u16;
    let max_byte_len = (usize::from(byte_len) + unit_size).min(u16::MAX as usize) as u16;

    unsafe {
        (*destination_string).length = byte_len;
        (*destination_string).maximum_length = max_byte_len;
        (*destination_string).buffer = source_string.cast_mut();
    }
}

/// Fill an `OSVERSIONINFOEXW` with the current (spoofed) Windows version.
/// Unlike `GetVersionEx` this function is not subject to application compatibility shims.
///
/// # Arguments
/// * `info`: pointer to a writable `OSVERSIONINFOW` or `OSVERSIONINFOEXW` structure with
///   `dwOSVersionInfoSize` set correctly.
///
/// # Safety
/// `info` must point to a writable `OSVERSIONINFOW` or `OSVERSIONINFOEXW`
/// with `dwOSVersionInfoSize` set correctly.
///
/// # Returns
/// `STATUS_SUCCESS` (0) on success. `STATUS_INVALID_PARAMETER` (0xC000_000D) if `info` is null or has an unexpected size.
///
/// # Notes
/// This function fills the provided structure with a spoofed Windows version, which can be configured
/// via environment variables. The version information is logged for debugging purposes.
pub unsafe fn rtl_get_version(info: *mut OsVersionInfoW) -> u32 {
    if info.is_null() {
        return 0xC000_000D; // STATUS_INVALID_PARAMETER
    }

    let ver = get_version();
    let size = unsafe { (*info).os_version_info_size };

    match size {
        SIZEOF_OSVERSIONINFOW => {
            tracing::debug!(
                "RtlGetVersion: {}.{}.{} (OSVERSIONINFOW)",
                ver.major,
                ver.minor,
                ver.build
            );
            unsafe { ver.fill_w(info) };
        }
        SIZEOF_OSVERSIONINFOEXW | 0 => {
            // Size 0 is accepted by some callers; treat as Ex.
            tracing::debug!(
                "RtlGetVersion: {}.{}.{} SP{}.{} (OSVERSIONINFOEXW)",
                ver.major,
                ver.minor,
                ver.build,
                ver.service_pack_major,
                ver.service_pack_minor
            );
            let ex = info.cast::<OsVersionInfoExW>();
            unsafe {
                (*ex).os_version_info_size = SIZEOF_OSVERSIONINFOEXW;
                ver.fill_ex_w(ex);
            }
        }
        _ => {
            tracing::warn!("RtlGetVersion: unexpected size {size}");
            return 0xC000_000D; // STATUS_INVALID_PARAMETER
        }
    }

    0 // STATUS_SUCCESS
}
