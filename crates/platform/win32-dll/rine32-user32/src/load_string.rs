use rine_common_user32::string_resource as common;
use rine_types::strings::LPSTR;

/// Loads a string resource from the specified module into a caller-provided ANSI buffer.
///
/// # Arguments
/// * `hinstance` - Handle to the module instance that owns the string resource.
///   When this is `0`, the main executable module is used by default.
/// * `u_id` - String resource identifier to load.
/// * `lp_buffer` - Destination ANSI buffer for the loaded string.
/// * `cch_buffer_max` - Capacity of `lp_buffer` in characters, including the null terminator.
///
/// # Returns
/// The number of characters copied to `lp_buffer`, excluding the null terminator.
/// Returns `0` when the resource ID is unsupported or not found, when `lp_buffer` is null,
/// or when `cch_buffer_max` is non-positive.
///
/// # Safety
/// * If `lp_buffer` is non-null, it must point to writable memory for at least `cch_buffer_max` bytes.
/// * The caller is responsible for ensuring the provided module/resource identifiers are valid.
///
/// # Notes
/// This implementation performs a deterministic no-cache lookup in PE string resources.
/// For resource entries with multiple language variants, the lowest numeric LANGID is selected.
/// UTF-16 string data is converted to ANSI with deterministic lossy rules:
/// ASCII code points (`0x00..=0x7F`) are preserved and all other characters are replaced with `?`.
/// On unsupported or missing IDs, the function writes a null terminator to `lp_buffer[0]`
/// when `lp_buffer` is non-null and `cch_buffer_max > 0`, then returns `0`.
/// Malformed resource structures are treated as not found and also return `0` with the same
/// output-buffer clearing behavior.
/// This function does not set `GetLastError`.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn LoadStringA(
    hinstance: u32,
    u_id: u32,
    lp_buffer: LPSTR,
    cch_buffer_max: i32,
) -> i32 {
    unsafe { common::load_string_a(hinstance, u_id, lp_buffer, cch_buffer_max) }
}
