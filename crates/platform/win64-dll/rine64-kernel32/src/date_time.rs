use rine_common_kernel32::date_time as common;
use rine_types::date_time::{LPSYSTEMTIME, SYSTEMTIME};
use rine_types::locale::LCID;
use rine_types::strings::{LPCSTR, LPSTR};

/// Retrieves the current local date and time
///
/// # Arguments
/// * `lp_system_time` - A pointer to a SYSTEMTIME structure to receive the current local date and time.
///
/// # Safety
/// This function is unsafe because it dereferences a raw pointer.
/// The caller must ensure that the pointer is valid and points to a properly initialized SYSTEMTIME structure.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetLocalTime(lp_system_time: LPSYSTEMTIME) {
    unsafe {
        common::get_local_time(lp_system_time);
    }
}

/// Formats a time as a string based on the specified locale and format string
///
/// # Arguments
/// * `locale` - The locale identifier (LCID) for the desired locale.
/// * `dw_flags` - Formatting options.
/// * `lp_time` - A pointer to a SYSTEMTIME structure containing the time to format.
/// * `lp_format` - A pointer to a null-terminated string that specifies the format of the output string.
/// * `lp_time_str` - A pointer to a buffer that receives the formatted time string.
/// * `cch_time` - The size of the buffer pointed to by `lp_time_str`.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that the pointers are valid and point to properly initialized structures and buffers.
///
/// # Returns
/// The function returns the number of characters written to the buffer, not including the null terminator, or 0 if the function fails.
///
/// # Notes
/// This is a stub implementation and does not perform actual formatting.
/// It always returns 0.
/// In a complete implementation, this function would format the time according to the specified locale and format string,
/// and write the result to the provided buffer. Further, it would also set the last error code on `GetLastError` appropriately in case of failure.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetTimeFormatA(
    _locale: LCID,
    _dw_flags: u32,
    _lp_time: *const SYSTEMTIME,
    _lp_format: LPCSTR,
    _lp_time_str: LPSTR,
    _cch_time: u32,
) -> u32 {
    unsafe {
        common::get_time_format(
            _locale,
            _dw_flags,
            _lp_time,
            _lp_format,
            _lp_time_str,
            _cch_time,
        )
    }
}
