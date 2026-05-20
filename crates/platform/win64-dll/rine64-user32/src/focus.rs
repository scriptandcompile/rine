use rine_common_user32 as common;
use rine_types::errors::BOOL;
use rine_types::windows::*;

/// GetFocus — retrieves the handle of the window with keyboard focus.
///
/// # Safety
/// The caller must ensure that the returned HWND is still valid before using it.
///
/// # Returns
/// The HWND of the focused window, or HWND::NULL if no window has focus.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetFocus() -> HWND {
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
pub unsafe extern "win64" fn SetFocus(hwnd: HWND) -> HWND {
    common::set_focus(hwnd)
}

/// GetLastActivePopup — retrieves the handle to the most recent active popup
/// window owned by the specified window.
///
/// # Arguments
/// * `hwnd` - A handle to the owner window.
///
/// # Returns
/// The handle to the most recent active popup window. The return value is the
/// same as `hwnd` if any of the following are true:
/// - The window identified by `hwnd` was not recently active.
/// - The window identified by `hwnd` does not own any popup windows.
/// - The window identified by `hwnd` is itself a popup window.
///
/// # Safety
/// The caller must ensure that the provided HWND is valid (was returned by a
/// previous `CreateWindow` or `CreateWindowEx` call). Passing an invalid HWND
/// will cause the function to return the input `hwnd` unchanged.
///
/// # Notes
/// Partial implementation. Popup activation tracking is not yet wired into
/// `SetFocus`/`SetForegroundWindow`; therefore this function currently returns
/// `hwnd` (the fallback per spec) when no popup has been explicitly recorded.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetLastActivePopup(hwnd: HWND) -> HWND {
    common::get_last_active_popup(hwnd)
}

/// SetForegroundWindow — brings the thread that created the specified window
/// into the foreground and activates the window.
///
/// Posts WM_ACTIVATE (WA_ACTIVE) and WM_SETFOCUS to the newly-foreground
/// window, and WM_ACTIVATE (WA_INACTIVE) and WM_KILLFOCUS to the previously
/// foreground window.
///
/// # Arguments
/// * `hwnd` - A handle to the window that should be brought to the foreground.
///
/// # Safety
/// The caller must ensure that the provided HWND is valid (was returned by a
/// previous `CreateWindow` or `CreateWindowEx` call). Passing an invalid HWND
/// will cause the function to return `BOOL::FALSE`.
///
/// # Returns
/// `BOOL::TRUE` if the window was brought to the foreground, `BOOL::FALSE`
/// if the HWND is NULL or does not refer to a valid window.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn SetForegroundWindow(hwnd: HWND) -> BOOL {
    common::set_foreground_window(hwnd)
}
