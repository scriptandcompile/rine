use rine_common_user32::dialogs as common;

use rine_types::errors::WinBool;
use rine_types::handles::HInstance;
use rine_types::strings::{LPCSTR, LPCWSTR};
use rine_types::windows::{Hwnd, LPARAM, LRESULT, WPARAM};

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
    _init_param: LPARAM,
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
    _init_param: LPARAM,
) -> Hwnd {
    let _template = unsafe { _template.read_string().unwrap_or_default() };

    common::create_dialog_param(_hinstance, &_template, _parent, _dlgproc, _init_param)
}

/// Sends a specified message to a control in a dialog box.
///
/// # Arguments
/// * `_hdlg` - A handle to the dialog box that contains the control.
/// * `_dlg_item_id` - The identifier of the control.
/// * `_message` - The message to be sent.
/// * `_wparam` - Additional message-specific information.
/// * `_lparam` - Additional message-specific information.
///
/// # Safety
/// `_hdlg` must be a valid handle to a dialog box.
/// `_dlg_item_id` must be a valid control identifier within that dialog box.
/// `_message` must be a valid message that can be sent to the control.
/// `_wparam` and `_lparam` must be valid additional message-specific information for the message being sent.
/// The function does not perform any validation on the input parameters.
/// It is the caller's responsibility to ensure that they are valid and that the message being sent is appropriate for
/// the control identified by `_dlg_item_id`.
///
/// # Returns
/// The return value is the result of the message processing; it depends on the message sent.
///
/// # Notes
/// The current implementation is a stub and always returns `0`.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn SendDlgItemMessageA(
    _hdlg: Hwnd,
    _dlg_item_id: i32,
    _message: u32,
    _wparam: WPARAM,
    _lparam: LPARAM,
) -> LRESULT {
    common::send_dialog_item_message(_hdlg, _dlg_item_id, _message, _wparam, _lparam)
}

/// Sends a specified message to a control in a dialog box.
///
/// # Arguments
/// * `_hdlg` - A handle to the dialog box that contains the control.
/// * `_dlg_item_id` - The identifier of the control.
/// * `_message` - The message to be sent.
/// * `_wparam` - Additional message-specific information.
/// * `_lparam` - Additional message-specific information.
///
/// # Safety
/// `_hdlg` must be a valid handle to a dialog box.
/// `_dlg_item_id` must be a valid control identifier within that dialog box.
/// `_message` must be a valid message that can be sent to the control.
/// `_wparam` and `_lparam` must be valid additional message-specific information for the message being sent.
/// The function does not perform any validation on the input parameters.
/// It is the caller's responsibility to ensure that they are valid and that the message being sent is appropriate for
/// the control identified by `_dlg_item_id`.
///
/// # Returns
/// The return value is the result of the message processing; it depends on the message sent.
///
/// # Notes
/// The current implementation is a stub and always returns `0`.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn SendDlgItemMessageW(
    _hdlg: Hwnd,
    _dlg_item_id: i32,
    _message: u32,
    _wparam: WPARAM,
    _lparam: LPARAM,
) -> LRESULT {
    common::send_dialog_item_message(_hdlg, _dlg_item_id, _message, _wparam, _lparam)
}

/// Retrieves the identifier of a specified control.
///
/// # Arguments
/// * `_hwnd` - A handle to the control.
///
/// # Safety
/// `_hwnd` must be a valid handle to a control.
/// The function does not perform any validation on the input parameter.
/// It is the caller's responsibility to ensure that it is valid and that it identifies a control.
///
/// # Returns
/// If the function succeeds, the return value is the identifier of the control.
/// If the function fails, the return value is `0`.
/// On failure, to get extended error information, call `GetLastError`.
///
/// # Notes
/// The current implementation is a stub and always returns `0`.
/// We currently do not set the value of `GetLastError` on failure.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetDlgCtrlID(_hwnd: Hwnd) -> i32 {
    common::get_dialog_control_id(_hwnd)
}

/// Retrieves the ANSI title or text of a control in a dialog box.
///
/// # Arguments
/// * `_hdlg` - A handle to the dialog box that contains the control.
/// * `_dlg_item_id` - The identifier of the control.
/// * `_buffer` - A pointer to the buffer that receives the text.
/// * `_max_text_length` - The maximum length, in characters, of the string to be copied to the buffer pointed to by _buffer.
///   If the length of the string, including the null character, exceeds the limit, the string is truncated.
///
/// # Safety
/// `_hdlg` must be a valid handle to a dialog box.
/// `_dlg_item_id` must be a valid control identifier within that dialog box.
/// `_buffer` must be a valid pointer to a buffer that can receive the text.
/// `_max_text_length` must be a valid maximum length for the text to be copied to the buffer.
///
/// # Returns
/// If the function succeeds, the return value is the length of the string copied to the buffer, not including the terminating null character.
/// If the function fails, the return value is `0`.
/// On failure, to get extended error information, call `GetLastError`.
///
/// # Notes
/// The current implementation is a stub and always returns `0`.
/// We currently do not set the value of `GetLastError` on failure.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetDlgItemTextA(
    _hdlg: Hwnd,
    _dlg_item_id: i32,
    _buffer: LPCSTR,
    _max_text_length: i32,
) -> u32 {
    common::get_dialog_item_text_a(_hdlg, _dlg_item_id, _buffer, _max_text_length)
}

/// Retrieves the wide (Unicode) title or text of a control in a dialog box.
///
/// # Arguments
/// * `_hdlg` - A handle to the dialog box that contains the control.
/// * `_dlg_item_id` - The identifier of the control.
/// * `_buffer` - A pointer to the buffer that receives the text.
/// * `_max_text_length` - The maximum length, in characters, of the string to be copied to the buffer pointed to by _buffer.
///   If the length of the string, including the null character, exceeds the limit, the string is truncated.
///
/// # Safety
/// `_hdlg` must be a valid handle to a dialog box.
/// `_dlg_item_id` must be a valid control identifier within that dialog box.
/// `_buffer` must be a valid pointer to a buffer that can receive the text.
/// `_max_text_length` must be a valid maximum length for the text to be copied to the buffer.
///
/// # Returns
/// If the function succeeds, the return value is the length of the string copied to the buffer, not including the terminating null character.
/// If the function fails, the return value is `0`.
/// On failure, to get extended error information, call `GetLastError`.
///
/// # Notes
/// The current implementation is a stub and always returns `0`.
/// We currently do not set the value of `GetLastError` on failure.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetDlgItemTextW(
    _hdlg: Hwnd,
    _dlg_item_id: i32,
    _buffer: LPCWSTR,
    _max_text_length: i32,
) -> u32 {
    common::get_dialog_item_text_w(_hdlg, _dlg_item_id, _buffer, _max_text_length)
}
