use rine_common_kernel32::date_time as common;
use rine_types::date_time::LPSYSTEMTIME;

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
