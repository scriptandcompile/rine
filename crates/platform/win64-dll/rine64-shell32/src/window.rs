use rine_common_shell32::window as common;

use rine_types::errors::WinBool;
use rine_types::windows::Hwnd;

/// Enables or disables file-drop acceptance for a window.
///
/// # Arguments
/// * `hwnd` - Handle to the target window.
/// * `f_accept` - Nonzero enables file drops, zero disables them.
///
/// # Safety
/// The caller must ensure `hwnd` refers to a valid window handle for this process context.
///
/// # Return
/// This function returns no value.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn DragAcceptFiles(hwnd: Hwnd, f_accept: WinBool) {
    common::drag_accept_files(hwnd, f_accept);
}
