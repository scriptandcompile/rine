use rine_common_user32::cursor as common;
use rine_types::errors::BOOL;
use rine_types::strings::{LPCSTR, LPCWSTR};
use rine_types::windows::Point;

use tracing::warn;

/// Loads the specified cursor resource, returning a handle to the cursor if successful.
///
/// # Arguments
/// * `hinstance` - A handle to the instance of the module whose executable file contains the
///   cursor to be loaded. If this parameter is `0`, the function loads a system predefined cursor.
/// * `name` - The ASCII name of the cursor resource to be loaded. If this is a
///   `MAKEINTRESOURCE` value (high 16 bits of the pointer are zero), it is treated as an
///   integer resource ID rather than a string pointer.
///
/// # Safety
/// If `hinstance` is not `0`, it must be a valid handle to an instance of a module that
/// contains the cursor resource specified by `name`. The caller must ensure that the module
/// is properly loaded and that the cursor resource is correctly defined within the module.
/// If `hinstance` is `0`, the caller must ensure that `name` corresponds to a valid predefined
/// cursor name or integer identifier (MAKEINTRESOURCE value).
/// When `name` is not an integer resource identifier, it must be a valid pointer to a
/// null-terminated ANSI string.
/// The caller must also ensure that the returned cursor handle is properly managed and released
/// when no longer needed to avoid resource leaks.
///
/// # Returns
/// A `u32` containing the handle to the loaded cursor if the operation was successful, or `0`
/// if the function fails to load the specified cursor (for example, if the specified cursor
/// resource does not exist or if the module handle is invalid).
///
/// # Notes
/// This function is currently a stub and returns `0` as a placeholder.
/// Integer resource identifiers (MAKEINTRESOURCE) are correctly detected and forwarded as
/// `"#<id>"` to the common layer, but no actual cursor loading is performed.
/// This function does not set the `GetLastError` value on failure.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn LoadCursorA(hinstance: u32, name: LPCSTR) -> u32 {
    unsafe {
        if name.is_null() {
            warn!("LoadCursorA received a null cursor name pointer. Returning 0.");
            return 0;
        }

        let cursor_name = if name.is_int_resource() {
            format!("#{}", name.to_int_resource())
        } else {
            let Some(s) = name.read_string() else {
                warn!("LoadCursorA received an invalid cursor name pointer. Returning 0.");
                return 0;
            };
            s
        };

        common::load_cursor(hinstance, &cursor_name).unwrap_or_default()
    }
}

/// Loads the specified cursor resource, returning a handle to the cursor if successful.
///
/// # Arguments
/// * `hinstance` - A handle to the instance of the module whose executable file contains the
///   cursor to be loaded. If this parameter is `0`, the function loads a system predefined cursor.
/// * `name` - The wide (UTF-16) name of the cursor resource to be loaded. If this is a
///   `MAKEINTRESOURCE` value (high 16 bits of the pointer are zero), it is treated as an
///   integer resource ID rather than a string pointer.
///
/// # Safety
/// If `hinstance` is not `0`, it must be a valid handle to an instance of a module that
/// contains the cursor resource specified by `name`. The caller must ensure that the module
/// is properly loaded and that the cursor resource is correctly defined within the module.
/// If `hinstance` is `0`, the caller must ensure that `name` corresponds to a valid predefined
/// cursor name or integer identifier (MAKEINTRESOURCE value).
/// When `name` is not an integer resource identifier, it must be a valid pointer to a
/// null-terminated UTF-16 string.
/// The caller must also ensure that the returned cursor handle is properly managed and released
/// when no longer needed to avoid resource leaks.
///
/// # Returns
/// A `u32` containing the handle to the loaded cursor if the operation was successful, or `0`
/// if the function fails to load the specified cursor (for example, if the specified cursor
/// resource does not exist or if the module handle is invalid).
///
/// # Notes
/// This function is currently a stub and returns `0` as a placeholder.
/// Integer resource identifiers (MAKEINTRESOURCE) are correctly detected and forwarded as
/// `"#<id>"` to the common layer, but no actual cursor loading is performed.
/// This function does not set the `GetLastError` value on failure.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn LoadCursorW(hinstance: u32, name: LPCWSTR) -> u32 {
    unsafe {
        if name.is_null() {
            warn!("LoadCursorW received a null cursor name pointer. Returning 0.");
            return 0;
        }

        let cursor_name = if name.is_int_resource() {
            format!("#{}", name.to_int_resource())
        } else {
            let Some(s) = name.read_string() else {
                warn!("LoadCursorW received an invalid cursor name pointer. Returning 0.");
                return 0;
            };
            s
        };

        common::load_cursor(hinstance, &cursor_name).unwrap_or_default()
    }
}

/// Sets the cursor shape, returning the handle to the previous cursor if successful.
///
/// # Arguments
/// * `cursor` - A handle to the cursor to be set.
///   If this parameter is `0`, the function removes the cursor from the screen
///   (the cursor is hidden until the next mouse movement or a subsequent `SetCursor` call).
///
/// # Safety
/// The cursor must have been created by either the `CreateCursor` function or the
/// `CreateIconIndirect` function, or loaded by either the `LoadCursor` function or
/// the `LoadImage` function.
///
/// # Returns
/// A `u32` containing the handle to the previous cursor if the operation was successful, or `0`
/// if the function fails to set the specified cursor (for example, if the specified cursor
/// handle is invalid).
///
/// # Notes
/// This function is currently a stub and returns `0` as a placeholder.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn SetCursor(cursor: u32) -> u32 {
    common::set_cursor(cursor).unwrap_or_default()
}

/// Retrieves the position of the mouse cursor in screen coordinates.
///
/// # Arguments
/// * `lppt` - A pointer to a `Point` structure that receives the screen coordinates of the cursor.
///   The `x` member receives the x-coordinate, and the `y` member receives the y-coordinate.
///
/// # Safety
/// The `lppt` parameter must be a valid pointer to a `Point` structure that can receive the cursor
/// coordinates. The cursor position is always specified in screen coordinates and is not affected
/// by the mapping mode of the window that contains the cursor. The calling process must have
/// `WINSTA_READATTRIBUTES` access to the window station. The input desktop must be the current
/// desktop when you call this function.
///
/// # Returns
/// `BOOL::TRUE` if the function succeeds, or `BOOL::FALSE` if it fails (for example, if the
/// calling process does not have the required access to the window station or if the input
/// desktop is not the current desktop).
///
/// # Notes
/// This function is currently a stub and returns `BOOL::FALSE` as a placeholder.
/// This implementation does not set the `GetLastError` value on failure.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetCursorPos(lppt: *mut Point) -> BOOL {
    warn!("GetCursorPos is not implemented yet. Returning FALSE as a placeholder.");

    let result = common::get_cursor_pos();

    if result.is_none() {
        return BOOL::FALSE;
    }

    (*lppt).x = result.unwrap().0;
    (*lppt).y = result.unwrap().1;

    BOOL::TRUE
}
