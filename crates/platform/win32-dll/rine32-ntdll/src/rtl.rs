use rine_common_ntdll as common;
use rine_types::os::OsVersionInfoW;
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
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn RtlInitUnicodeString(
    destination_string: *mut UnicodeString, // PUNICODE_STRING
    source_string: *const u16,              // PCWSTR
) {
    unsafe { common::rtl::rtl_init_unicode_string(destination_string, source_string) };
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
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn RtlGetVersion(info: *mut OsVersionInfoW) -> u32 {
    unsafe { common::rtl::rtl_get_version(info) }
}
