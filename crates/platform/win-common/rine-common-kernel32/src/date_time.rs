use chrono::{Datelike, Local, Timelike};

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
pub unsafe fn get_local_time(mut lp_system_time: LPSYSTEMTIME) {
    if lp_system_time.is_null() {
        return;
    }

    unsafe {
        std::ptr::write_bytes(lp_system_time.as_mut(), 0, 1);

        let local_now = Local::now();

        lp_system_time.as_mut().wDay = local_now.day() as u16;
        lp_system_time.as_mut().wDayOfWeek = local_now.weekday().number_from_sunday() as u16;
        lp_system_time.as_mut().wHour = local_now.hour() as u16;
        // We have to clamp to 999 because timestamp_subsec_millis can go larger than 999 in leap second scenarios,
        // but the win32 api expects a value between 0 and 999.
        lp_system_time.as_mut().wMilliseconds =
            std::cmp::max(local_now.timestamp_subsec_millis(), 999) as u16;
        lp_system_time.as_mut().wMinute = local_now.minute() as u16;
        lp_system_time.as_mut().wMonth = local_now.month() as u16;
        lp_system_time.as_mut().wSecond = local_now.second() as u16;
        lp_system_time.as_mut().wYear = local_now.year() as u16;
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
pub unsafe fn get_time_format(
    _locale: LCID,
    _dw_flags: u32,
    _lp_time: *const SYSTEMTIME,
    _lp_format: LPCSTR,
    _lp_time_str: LPSTR,
    _cch_time: u32,
) -> u32 {
    0
}

/// Formats a date as a string based on the specified locale and format string.
///
/// # Arguments
/// * `locale` - The locale identifier (LCID) for the desired locale.
/// * `dw_flags` - Formatting options.
/// * `lp_date` - A pointer to a SYSTEMTIME structure containing the date to format.
/// * `lp_format` - A pointer to a null-terminated string that specifies the format of the output string.
/// * `lp_date_str` - A pointer to a buffer that receives the formatted date string.
/// * `cch_date` - The size of the buffer pointed to by `lp_date_str`.
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
/// In a complete implementation, this function would format the date according to the specified locale and format string,
/// and write the result to the provided buffer. Further, it would also set the last error code on `GetLastError` appropriately in case of failure.
pub unsafe fn get_date_format_a(
    _locale: LCID,
    _dw_flags: u32,
    _lp_date: *const SYSTEMTIME,
    _lp_format: LPCSTR,
    _lp_date_str: LPSTR,
    _cch_date: u32,
) -> u32 {
    0
}
