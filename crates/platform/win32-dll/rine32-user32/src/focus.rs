use rine_common_user32 as common;
use rine_types::windows::*;

/// GetFocus — retrieves the handle of the window with keyboard focus.
///
/// # Safety
/// Calling this function is safe, but the returned HWND may be invalid if the window has been destroyed.
/// The caller must ensure that the returned HWND is still valid before using it.
///
/// # Returns
/// The HWND of the focused window, or HWND::NULL if no window in the calling thread's message queue has focus.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetFocus() -> HWND {
    common::get_focus()
}

/// SetFocus — sets keyboard focus to the specified window.
///
/// Posts WM_KILLFOCUS to the previously focused window and WM_SETFOCUS
/// to the newly focused window.
///
/// # Arguments
/// * `hwnd` - A handle to the window that will receive the keyboard focus.
///
/// # Safety
/// The caller must ensure that the provided HWND is valid and belongs to the calling thread.
///
/// # Returns
/// The handle of the window that previously had focus, or HWND::NULL.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn SetFocus(hwnd: HWND) -> HWND {
    common::set_focus(hwnd)
}
