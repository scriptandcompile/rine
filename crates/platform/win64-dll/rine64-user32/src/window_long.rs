use rine_common_user32 as common;
use rine_types::windows::*;

/// SetWindowLongA — change an attribute of the specified window.
///
/// # Arguments
/// * `hwnd` - Handle to the window.
/// * `n_index` - One of the `GWL_*` / `GWLP_*` / `DWLP_*` constants.
/// * `dw_new_long` - The replacement value (32-bit).
///
/// # Safety
/// * `hwnd` must be a valid window handle or `HWND::NULL`.
/// * The caller must have appropriate access rights to modify the window's attributes.
///
/// # Returns
/// The previous value of the specified window long, or 0 if the HWND is not found.
/// Note that 0 can be a valid previous value; callers should check `GetLastError()`
/// to distinguish success from failure.
///
/// # Notes
/// Currently, this function does not set `GetLastError` on failure.
/// The following index values are accepted but return 0 without modifying state:
/// `GWLP_HINSTANCE`, `DWLP_MSGRESULT`, `DWLP_DLGPROC`, `DWLP_USER`.
/// `GWL_ID` is tracked and can be queried through dialog-control APIs such as `GetDlgCtrlID`.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn SetWindowLongA(hwnd: HWND, n_index: i32, dw_new_long: i32) -> i32 {
    common::set_window_long_a(hwnd, n_index, dw_new_long)
}

/// SetWindowLongW — change an attribute of the specified window.
///
/// For non-string-based indices this behaves identically to `SetWindowLongA`.
///
/// # Arguments
/// * `hwnd` - Handle to the window.
/// * `n_index` - One of the `GWL_*` / `GWLP_*` / `DWLP_*` constants.
/// * `dw_new_long` - The replacement value (32-bit).
///
/// # Safety
/// * `hwnd` must be a valid window handle or `HWND::NULL`.
/// * The caller must have appropriate access rights to modify the window's attributes.
///
/// # Returns
/// The previous value of the specified window long, or 0 if the HWND is not found.
///
/// # Notes
/// Currently, this function does not set `GetLastError` on failure.
/// The following index values are accepted but return 0 without modifying state:
/// `GWLP_HINSTANCE`, `DWLP_MSGRESULT`, `DWLP_DLGPROC`, `DWLP_USER`.
/// `GWL_ID` is tracked and can be queried through dialog-control APIs such as `GetDlgCtrlID`.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn SetWindowLongW(hwnd: HWND, n_index: i32, dw_new_long: i32) -> i32 {
    common::set_window_long_a(hwnd, n_index, dw_new_long)
}
