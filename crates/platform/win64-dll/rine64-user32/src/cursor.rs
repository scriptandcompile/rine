use rine_common_user32::cursor as common;
use rine_types::errors::WinBool;
use rine_types::strings::read_cstr;
use rine_types::windows::Point;

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

/// Retrieves the position of the mouse cursor in screen coordinates.
///
/// # Arguments
/// * `_lppt` - A pointer to a `Point` structure that receives the screen coordinates of the cursor.
///   The `x` member receives the x-coordinate of the cursor, and the `y` member receives the y-coordinate of the cursor.
///
/// # Safety
/// The `_lppt` parameter must be a valid pointer to a `Point` structure that can receive the cursor coordinates.
/// The cursor position is always specified in screen coordinates and is not affected by the mapping mode of the window that contains the cursor.
/// The calling process must have `WINSTA_READATTRIBUTES` access to the window station.
/// The input desktop must be the current desktop when you call this function.
/// If the input desktop is not the current desktop, the function fails and returns `None`.
///
/// # Returns
/// An `Option<(i32, i32)>` containing the x and y coordinates of the cursor in screen coordinates if the operation was successful,
/// or `None` if the function fails to retrieve the cursor position (for example, if the calling process does not have the required
/// access to the window station or if the input desktop is not the current desktop).
///
/// # Notes
/// This function is currently a stub and returns `None` as a placeholder.
/// This implementation does not set the `GetLastError` value on failure.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetCursorPos(_lppt: *mut Point) -> WinBool {
    warn!("GetCursorPos is not implemented yet. Returning None as a placeholder.");

    let result = common::get_cursor_pos();

    if result.is_none() {
        return WinBool::FALSE;
    }

    (*_lppt).x = result.unwrap().0;
    (*_lppt).y = result.unwrap().1;

    WinBool::TRUE
}
