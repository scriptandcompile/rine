use rine_types::errors::WinBool;
use rine_types::windows::{Hwnd, WINDOW_MANAGER, window_style_ex};

/// Enables or disables file-drop acceptance for a window.
///
/// # Arguments
/// * `hwnd` - Target window handle.
/// * `accept` - Nonzero enables file drops; zero disables them.
///
/// # Return
/// This function returns no value.
///
/// # Notes
/// This updates the tracked extended style bit (`WS_EX_ACCEPTFILES`) for
/// windows known to the current runtime.
pub fn drag_accept_files(hwnd: Hwnd, accept: WinBool) {
    let _ = WINDOW_MANAGER.update_window(hwnd, |state| {
        if accept.is_true() {
            state.ex_style |= window_style_ex::WS_EX_ACCEPTFILES;
        } else {
            state.ex_style &= !window_style_ex::WS_EX_ACCEPTFILES;
        }
    });
}
