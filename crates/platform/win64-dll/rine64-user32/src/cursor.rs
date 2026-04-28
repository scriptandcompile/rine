use rine_common_user32::cursor as common;
use rine_types::strings::read_cstr;

use tracing::warn;

/// Loads the specified cursor resource, returning a handle to the cursor if successful.
///
/// # Arguments
/// * `_hinstance` - A handle to the instance of the module whose executable file contains the cursor to be loaded.
///   If this parameter is `0`, the function loads the cursor from the system's predefined cursors.
/// * `_name` - The ASCII name of the cursor resource to be loaded or the cursor's integer identifier cast to a string.
///
/// # Safety
/// If `_hinstance` is not `0`, it must be a valid handle to an instance of a module that contains the cursor resource specified by `_name`.
/// The caller must ensure that the module is properly loaded and that the cursor resource is correctly defined within the module.
/// If `_hinstance` is `0`, the caller must ensure that `_name` corresponds to a valid predefined cursor name or integer identifier.
/// The caller must also ensure that the returned cursor handle is properly managed and released when no longer needed to avoid resource leaks.
///
/// # Returns
/// An `u32` containing the handle to the loaded cursor if the operation was successful, or `0` if the function
/// fails to load the specified cursor (for example, if the specified cursor resource does not exist or if the module handle is invalid).
///
/// # Notes
/// This function is currently a stub and returns `0` as a placeholder.
/// Currently, this function does not set the `GetLastError` value on failure.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn LoadCursorA(_hinstance: u32, _name: *const u8) -> u32 {
    unsafe {
        let Some(cursor_name) = read_cstr(_name) else {
            warn!(
                "LoadCursorA received an invalid cursor name pointer. Returning 0 as a placeholder."
            );
            return 0;
        };

        common::load_cursor(_hinstance, &cursor_name).unwrap_or_default()
    }
}

/// Loads the specified cursor resource, returning a handle to the cursor if successful.
///
/// # Arguments
/// * `_hinstance` - A handle to the instance of the module whose executable file contains the cursor to be loaded.
///   If this parameter is `0`, the function loads the cursor from the system's predefined cursors.
/// * `_name` - The wide name of the cursor resource to be loaded or the cursor's integer identifier cast to a string.
///
/// # Safety
/// If `_hinstance` is not `0`, it must be a valid handle to an instance of a module that contains the cursor resource specified by `_name`.
/// The caller must ensure that the module is properly loaded and that the cursor resource is correctly defined within the module.
/// If `_hinstance` is `0`, the caller must ensure that `_name` corresponds to a valid predefined cursor name or integer identifier.
/// The caller must also ensure that the returned cursor handle is properly managed and released when no longer needed to avoid resource leaks.
///
/// # Returns
/// An `u32` containing the handle to the loaded cursor if the operation was successful, or `0` if the function
/// fails to load the specified cursor (for example, if the specified cursor resource does not exist or if the module handle is invalid).
///
/// # Notes
/// This function is currently a stub and returns `0` as a placeholder.
/// Currently, this function does not set the `GetLastError` value on failure.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn LoadCursorW(_hinstance: u32, _name: *const u8) -> u32 {
    unsafe {
        let Some(cursor_name) = read_cstr(_name) else {
            warn!(
                "LoadCursorW received an invalid cursor name pointer. Returning 0 as a placeholder."
            );
            return 0;
        };

        common::load_cursor(_hinstance, &cursor_name).unwrap_or_default()
    }
}

/// Sets the cursor shape, returning the handle to the previous cursor if successful.
///
/// # Arguments
/// * `_cursor` - A handle to the cursor to be set.
///   If this parameter is `0`, the function sets the cursor to `None`, which means that the cursor will be hidden until the next mouse movement.
///
/// # Safety
/// The cursor must have been created by either the `CreateCursor` function or the `CreateIconIndirect` function,
/// or loaded by either the `LoadCursor` function or the `LoadImage` function.
///
/// # Returns
/// An `u32` containing the handle to the previous cursor if the operation was successful, or `0` if the function
/// fails to set the specified cursor (for example, if the specified cursor handle is invalid).
///
/// # Notes
/// This function is currently a stub and returns `0` as a placeholder.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn SetCursor(_cursor: u32) -> u32 {
    common::set_cursor(_cursor).unwrap_or_default()
}
