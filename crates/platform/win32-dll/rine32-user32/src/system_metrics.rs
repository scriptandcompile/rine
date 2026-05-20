use rine_common_user32 as common;

/// Retrieves the specified system metric or system configuration setting.
///
/// # Arguments
/// * `n_index` - The system metric or configuration setting to retrieve. See `system_metrics` module for available constants.
///
/// # Safety
/// The caller must ensure that `n_index` is a valid system metric index. Passing an unrecognized index
/// is safe but returns `0` with a warning logged.
///
/// # Returns
/// An `i32` containing the requested system metric value, or `0` if the index is not recognized.
///
/// # Notes
/// This function is currently a stub and returns hardcoded default values (1920x1080 display,
/// 3 mouse buttons, standard UI metrics) rather than querying the actual host system.
/// The following are not yet implemented:
/// - Dynamic display resolution detection
/// - Multi-monitor configuration
/// - Actual mouse button count detection
/// - Real-time metric updates
/// - This function does not set the `GetLastError` value on failure.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetSystemMetrics(n_index: i32) -> i32 {
    common::system_metrics::get_system_metrics(n_index)
}
