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

pub fn rtl_get_version() -> u32 {
    tracing::warn!(
        api = "RtlGetVersion",
        dll = "ntdll",
        "RtlGetVersion stub called. Returned success"
    );
    0
}
