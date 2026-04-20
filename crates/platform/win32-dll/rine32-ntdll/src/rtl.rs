use rine_common_ntdll as common;
use rine_types::strings::UnicodeString;

/// Initialize a UNICODE_STRING structure with the given source string.
///
/// # Arguments
/// * `destination_string`: pointer to the UNICODE_STRING structure to initialize.
/// * `source_string`: pointer to a null-terminated wide string (PCWSTR) to copy into the UNICODE_STRING.
///
/// # Safety
/// All pointer parameters must be valid.
/// The `destination_string` must point to a valid UNICODE_STRING structure, and `source_string` must point
/// to a valid null-terminated wide string.
///
/// # Notes
/// This is a stub implementation that does not perform any actual initialization.
/// It simply logs a warning and does not modify the destination string.
#[allow(non_snake_case)]
pub unsafe extern "stdcall" fn RtlInitUnicodeString(
    destination_string: *mut UnicodeString, // PUNICODE_STRING
    source_string: *const u16,              // PCWSTR
) {
    unsafe { common::rtl::rtl_init_unicode_string(destination_string, source_string) };
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn RtlGetVersion() -> u32 {
    tracing::warn!(api = "RtlGetVersion", dll = "ntdll", "win32 stub called");
    0
}
