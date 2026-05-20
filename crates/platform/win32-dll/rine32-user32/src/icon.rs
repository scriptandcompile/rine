use rine_common_user32::icon as common;
use rine_types::strings::{LPCSTR, LPCWSTR};

use tracing::warn;

/// Loads the specified icon resource, returning a handle to the icon if successful.
///
/// # Arguments
/// * `_hinstance` - A handle to the instance of the module whose executable file contains the icon to be loaded.
///   If this parameter is `0`, the function loads a predefined icon.
/// * `_name` - The ASCII name of the icon resource to be loaded or the icon's integer identifier cast to a string.
///
/// # Safety
/// If `_hinstance` is not `0`, it must be a valid handle to an instance of a module that contains the icon resource specified by `_name`.
/// The caller must ensure that the module is properly loaded and that the icon resource is correctly defined within the module.
/// If `_hinstance` is `0`, the caller must ensure that `_name` corresponds to a valid predefined icon name or integer identifier.
/// The caller must also ensure that the returned icon handle is properly managed and released when no longer needed to avoid resource leaks.
///
/// # Returns
/// An `u32` containing the handle to the loaded icon if the operation was successful, or `0` if the function
/// fails to load the specified icon (for example, if the specified icon resource does not exist or if the module handle is invalid).
///
/// # Notes
/// This function is currently a stub and returns `0` as a placeholder.
/// Currently, this function does not set the `GetLastError` value on failure.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn LoadIconA(_hinstance: u32, _name: LPCSTR) -> u32 {
    unsafe {
        let Some(icon_name) = _name.read_string() else {
            warn!("LoadIconA received an invalid icon name pointer. Returning 0 as a placeholder.");
            return 0;
        };

        common::load_icon(_hinstance, &icon_name).unwrap_or_default()
    }
}

/// Loads the specified icon resource, returning a handle to the icon if successful.
///
/// # Arguments
/// * `_hinstance` - A handle to the instance of the module whose executable file contains the icon to be loaded.
///   If this parameter is `0`, the function loads a predefined icon.
/// * `_name` - The wide (UTF-16) name of the icon resource to be loaded or the icon's integer identifier cast to a string.
///
/// # Safety
/// If `_hinstance` is not `0`, it must be a valid handle to an instance of a module that contains the icon resource specified by `_name`.
/// The caller must ensure that the module is properly loaded and that the icon resource is correctly defined within the module.
/// If `_hinstance` is `0`, the caller must ensure that `_name` corresponds to a valid predefined icon name or integer identifier.
/// The caller must also ensure that the returned icon handle is properly managed and released when no longer needed to avoid resource leaks.
///
/// # Returns
/// An `u32` containing the handle to the loaded icon if the operation was successful, or `0` if the function
/// fails to load the specified icon (for example, if the specified icon resource does not exist or if the module handle is invalid).
///
/// # Notes
/// This function is currently a stub and returns `0` as a placeholder.
/// Currently, this function does not set the `GetLastError` value on failure.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn LoadIconW(_hinstance: u32, _name: LPCWSTR) -> u32 {
    unsafe {
        let Some(icon_name) = _name.read_string() else {
            warn!("LoadIconW received an invalid icon name pointer. Returning 0 as a placeholder.");
            return 0;
        };

        common::load_icon(_hinstance, &icon_name).unwrap_or_default()
    }
}
