use rine_common_user32::dialogs as common;

use rine_types::errors::WinBool;
use rine_types::handles::HInstance;
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
pub unsafe extern "win64" fn SetDlgItemTextA(
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
pub unsafe extern "win64" fn SetDlgItemTextW(
    _hDlg: Hwnd,
    _nIDDlgItem: i32,
    _lpString: LPCWSTR,
) -> WinBool {
    let text = unsafe { _lpString.read_string().unwrap_or_default() };

    common::set_dlg_item_text(_hDlg, _nIDDlgItem, &text)
}

/// Creates a modeless dialog box from a dialog box template in memory.
///
/// # Arguments
/// * `_hinstance` - A handle to the module whose executable file contains the dialog box template.
/// * `_template` - A pointer to a null-terminated string that specifies the dialog box template.
///   Alternatively, this parameter can be an integer value that specifies the dialog box template.
///   In this case, the parameter must be cast to `LPCSTR` or `LPCWSTR` and the `MAKEINTRESOURCE` macro must be used to create the value.
/// * `_parent` - A handle to the window that owns the dialog box. If this parameter is `Hwnd::NULL`, the dialog box has no owner window.
/// * `_dialog_proc` - A pointer to a dialog box procedure that processes messages sent to the dialog box.
/// * `_init_param` - The value to pass to the dialog box in the `lParam` parameter of the `WM_INITDIALOG` message.
///   This parameter can be used to pass any value to the dialog box, such as a pointer to a data structure that contains
///   initialization data for the dialog box.
///
/// # Safety
/// `_hinstance` must be a valid handle to a module.
/// `_template` must be a valid dialog box template.
/// `_parent` must be a valid handle to a window or `Hwnd::NULL`.
/// `_dialog_proc` must be NULL or a valid pointer to a dialog box procedure.
///   The dialog box procedure must be a valid pointer to a function that processes messages sent to a modal or modeless dialog box.
///   The return value of the dialog box procedure depends on the message being processed.
///   For most messages, the dialog box procedure should return `0` if it processes the message, and a nonzero value if it does not process the message.
///   For the `WM_INITDIALOG` message, the dialog box procedure should return `WinBool::TRUE` if it initializes the dialog box, and `WinBool::FALSE`
///   if it does not initialize the dialog box.
/// `_init_param` must be a valid value to pass to the dialog box in the `lParam` parameter of the `WM_INITDIALOG` message.
///
/// # Returns
/// If the function succeeds, the return value is a handle to the dialog box.
/// If the function fails, the return value is `Hwnd::NULL`.
/// To get extended error information, call `GetLastError`.
///
/// # Notes
/// The current implementation is a stub and always returns `Hwnd::NULL`.
/// We currently do not set the value of `GetLastError` on failure.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn CreateDialogParamA(
    _hinstance: HInstance,
    _template: LPCSTR,
    _parent: Hwnd,
    _dlgproc: common::DLGPROC,
    _init_param: isize,
) -> Hwnd {
    let _template = unsafe { _template.read_string().unwrap_or_default() };

    common::create_dialog_param(_hinstance, &_template, _parent, _dlgproc, _init_param)
}

/// Creates a modeless dialog box from a dialog box template in memory.
///
/// # Arguments
/// * `_hinstance` - A handle to the module whose executable file contains the dialog box template.
/// * `_template` - A pointer to a null-terminated string that specifies the dialog box template.
///   Alternatively, this parameter can be an integer value that specifies the dialog box template.
///   In this case, the parameter must be cast to `LPCSTR` or `LPCWSTR` and the `MAKEINTRESOURCE` macro must be used to create the value.
/// * `_parent` - A handle to the window that owns the dialog box. If this parameter is `Hwnd::NULL`, the dialog box has no owner window.
/// * `_dialog_proc` - A pointer to a dialog box procedure that processes messages sent to the dialog box.
/// * `_init_param` - The value to pass to the dialog box in the `lParam` parameter of the `WM_INITDIALOG` message.
///   This parameter can be used to pass any value to the dialog box, such as a pointer to a data structure that contains
///   initialization data for the dialog box.
///
/// # Safety
/// `_hinstance` must be a valid handle to a module.
/// `_template` must be a valid dialog box template.
/// `_parent` must be a valid handle to a window or `Hwnd::NULL`.
/// `_dialog_proc` must be NULL or a valid pointer to a dialog box procedure.
///   The dialog box procedure must be a valid pointer to a function that processes messages sent to a modal or modeless dialog box.
///   The return value of the dialog box procedure depends on the message being processed.
///   For most messages, the dialog box procedure should return `0` if it processes the message, and a nonzero value if it does not process the message.
///   For the `WM_INITDIALOG` message, the dialog box procedure should return `WinBool::TRUE` if it initializes the dialog box, and `WinBool::FALSE`
///   if it does not initialize the dialog box.
/// `_init_param` must be a valid value to pass to the dialog box in the `lParam` parameter of the `WM_INITDIALOG` message.
///
/// # Returns
/// If the function succeeds, the return value is a handle to the dialog box.
/// If the function fails, the return value is `Hwnd::NULL`.
/// To get extended error information, call `GetLastError`.
///
/// # Notes
/// The current implementation is a stub and always returns `Hwnd::NULL`.
/// We currently do not set the value of `GetLastError` on failure.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn CreateDialogParamW(
    _hinstance: HInstance,
    _template: LPCWSTR,
    _parent: Hwnd,
    _dlgproc: common::DLGPROC,
    _init_param: isize,
) -> Hwnd {
    let _template = unsafe { _template.read_string().unwrap_or_default() };

    common::create_dialog_param(_hinstance, &_template, _parent, _dlgproc, _init_param)
}
