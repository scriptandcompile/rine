use chrono::{Datelike, Local, Timelike};

use rine_types::date_time::LPSYSTEMTIME;

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
