use rine_common_user32::dialogs as common;

use rine_types::errors::WinBool;
use rine_types::strings::{LPCSTR, LPCWSTR};
use rine_types::windows::Hwnd;

/// Sets the title or text of a control in a dialog box.
///
/// # Arguments
/// * `_hDlg` - A handle to the dialog box that contains the control.
/// * `_nIDDlgItem` - The identifier of the control.
/// * `_lpString` - The string to be displayed.
///
/// # Safety
/// `_hDlg` must be a valid handle to a dialog box, `_nIDDlgItem` must be a valid control identifier within that dialog box.
///
/// # Returns
/// If the function succeeds, return `WinBool::TRUE`. If the function fails, return `WinBool::FALSE`.
///
/// # Notes
/// The `SetDlgItemTextA` function sends a `WM_SETTEXT` message to the specified control.
/// The `SetDlgItemTextA` function does not support setting the text of a combo box or list box control.
/// To set the text of these controls, use the `CB_SETLBTEXT` or `LB_SETLBTEXT` message, respectively.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn SetDlgItemTextA(
    _hDlg: Hwnd,
    _nIDDlgItem: i32,
    _lpString: LPCSTR,
) -> WinBool {
    let text = unsafe { _lpString.read_string().unwrap_or_default() };

    common::set_dlg_item_text(_hDlg, _nIDDlgItem, &text)
}

/// Sets the title or text of a control in a dialog box.
///
/// # Arguments
/// * `_hDlg` - A handle to the dialog box that contains the control.
/// * `_nIDDlgItem` - The identifier of the control.
/// * `_lpString` - The string to be displayed.
///
/// # Safety
/// `_hDlg` must be a valid handle to a dialog box, `_nIDDlgItem` must be a valid control identifier within that dialog box.
///
/// # Returns
/// If the function succeeds, return `WinBool::TRUE`. If the function fails, return `WinBool::FALSE`.
///
/// # Notes
/// The `SetDlgItemTextW` function sends a `WM_SETTEXT` message to the specified control.
/// The `SetDlgItemTextW` function does not support setting the text of a combo box or list box control.
/// To set the text of these controls, use the `CB_SETLBTEXT` or `LB_SETLBTEXT` message, respectively.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn SetDlgItemTextW(
    _hDlg: Hwnd,
    _nIDDlgItem: i32,
    _lpString: LPCWSTR,
) -> WinBool {
    let text = unsafe { _lpString.read_string().unwrap_or_default() };

    common::set_dlg_item_text(_hDlg, _nIDDlgItem, &text)
}
