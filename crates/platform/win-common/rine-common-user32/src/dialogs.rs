use rine_types::errors::BOOL;
use rine_types::handles::HINSTANCE;
use rine_types::strings::{LPCSTR, LPCWSTR};
use rine_types::windows::{HWND, LPARAM, LRESULT, WINDOW_MANAGER, WPARAM, window_message};
use text::{write_ansi_truncating_nul, write_wide_truncating_nul};

use tracing::warn;

fn find_dialog_item(hdlg: HWND, dlg_item_id: i32) -> HWND {
    if hdlg.is_null() {
        return HWND::NULL;
    }

    WINDOW_MANAGER.find_child_by_control_id(hdlg, dlg_item_id)
}

fn get_window_title(hwnd: HWND) -> Option<String> {
    WINDOW_MANAGER.get_window(hwnd).map(|state| state.title)
}

fn set_window_title(hwnd: HWND, text: &str) -> bool {
    WINDOW_MANAGER.update_window(hwnd, |state| {
        state.title = text.to_owned();
    })
}

unsafe fn write_ansi_text(buffer: *mut u8, max_text_length: i32, text: &str) -> u32 {
    unsafe { write_ansi_truncating_nul(buffer, max_text_length as isize, text.as_bytes()) as u32 }
}

unsafe fn write_wide_text(buffer: *mut u16, max_text_length: i32, text: &str) -> u32 {
    let wide: Vec<u16> = text.encode_utf16().collect();
    unsafe { write_wide_truncating_nul(buffer, max_text_length as isize, &wide) as u32 }
}

/// Sets the title or text of a control in a dialog box.
///
/// # Arguments
/// * `_hdlg` - A handle to the dialog box that contains the control.
/// * `_dlg_item_id` - The identifier of the control.
/// * `_text` - The string to be displayed.
///
/// # Safety
/// `_hdlg` must be a valid handle to a dialog box, `_dlg_item_id` must be a valid control identifier within that dialog box.
///
/// # Returns
/// If the function succeeds, return `BOOL::TRUE`. If the function fails, return `BOOL::FALSE`.
///
/// # Notes
/// Sends a `WM_SETTEXT` message to the specified control.
/// This function Does not support setting the text of a combo box or list box control.
/// To set the text of these controls, use the `CB_SETLBTEXT` or `LB_SETLBTEXT` message, respectively.
pub fn set_dlg_item_text(_hdlg: HWND, _dlg_item_id: i32, _text: &str) -> BOOL {
    let child = find_dialog_item(_hdlg, _dlg_item_id);
    if child.is_null() {
        return BOOL::FALSE;
    }

    if set_window_title(child, _text) {
        BOOL::TRUE
    } else {
        BOOL::FALSE
    }
}

/// Dialog proc is a pointer to a function that processes messages sent to a modal or modeless dialog box.
/// The dialog box procedure is an application-defined function that processes messages sent to a modal or modeless dialog box.
/// Dialog box procedures are similar to window procedures, but they have a different return value and a different set of messages that they process.
///
/// # Arguments
/// * `unnamed_param1` - A handle to the dialog box.
/// * `unnamed_param2` - The message.
/// * `unnamed_param3` - Additional message-specific information.
/// * `unnamed_param4` - Additional message-specific information.
///
/// # Safety
/// The dialog box procedure must be a valid pointer to a function that processes messages sent to a modal or modeless dialog box.
///
/// # Returns
/// The dialog box procedure must return a value of type `isize`.
/// The return value depends on the message being processed.
/// For most messages, the dialog box procedure should return `0` if it processes the message, and a nonzero value if it does not process the message.
/// For the `WM_INITDIALOG` message, the dialog box procedure should return `BOOL::TRUE` if it initializes the dialog box,
/// and `BOOL::FALSE` if it does not initialize the dialog box.
#[cfg(target_pointer_width = "32")]
pub type DLGPROC = *const unsafe extern "stdcall" fn(
    unnamed_param1: HWND,
    unnamed_param2: u32,
    unnamed_param3: WPARAM,
    unnamed_param4: LPARAM,
) -> isize;

/// Dialog proc is a pointer to a function that processes messages sent to a modal or modeless dialog box.
/// The dialog box procedure is an application-defined function that processes messages sent to a modal or modeless dialog box.
/// Dialog box procedures are similar to window procedures, but they have a different return value and a different set of messages that they process.
///
/// # Arguments
/// * `unnamed_param1` - A handle to the dialog box.
/// * `unnamed_param2` - The message.
/// * `unnamed_param3` - Additional message-specific information.
/// * `unnamed_param4` - Additional message-specific information.
///
/// # Safety
/// The dialog box procedure must be a valid pointer to a function that processes messages sent to a modal or modeless dialog box.
///
/// # Returns
/// The dialog box procedure must return a value of type `isize`.
/// The return value depends on the message being processed.
/// For most messages, the dialog box procedure should return `0` if it processes the message, and a nonzero value if it does not process the message.
/// For the `WM_INITDIALOG` message, the dialog box procedure should return `BOOL::TRUE` if it initializes the dialog box,
/// and `BOOL::FALSE` if it does not initialize the dialog box.
#[cfg(not(target_pointer_width = "32"))]
pub type DLGPROC = *const unsafe extern "win64" fn(
    unnamed_param1: HWND,
    unnamed_param2: u32,
    unnamed_param3: WPARAM,
    unnamed_param4: LPARAM,
) -> isize;

/// Creates a modeless dialog box from a dialog box template in memory.
///
/// # Arguments
/// * `_hinstance` - A handle to the module whose executable file contains the dialog box template.
/// * `_template` - A string that contains the dialog box template.
/// * `_parent` - A handle to the window that owns the dialog box.
/// * `_dialog_proc` - A pointer to the dialog box procedure.
/// * `_init_param` - The value to pass to the dialog box in the `lParam` parameter of the `WM_INITDIALOG` message.
///
/// # Safety
/// `_hinstance` must be a valid handle to a module.
/// `_template` must be a valid dialog box template.
/// `_parent` must be a valid handle to a window or `HWND::NULL`.
/// `_dialog_proc` must be a valid pointer to a dialog box procedure.
/// `_init_param` must be a valid value to pass to the dialog box in the `lParam` parameter of the `WM_INITDIALOG` message.
///
/// # Returns
/// If the function succeeds, the return value is a handle to the dialog box.
/// If the function fails, the return value is `HWND::NULL`.
/// On failure, to get extended error information, call `GetLastError`.
///
/// # Notes
/// The current implementation is a stub and always returns `HWND::NULL`.
/// We currently do not set the value of `GetLastError` on failure.
pub fn create_dialog_param(
    _hinstance: HINSTANCE,
    _template: &str,
    _parent: HWND,
    _dialog_proc: DLGPROC,
    _init_param: LPARAM,
) -> HWND {
    warn!(
        "CreateDialogParam is not implemented yet. This is a stub implementation that always returns NULL."
    );
    HWND::NULL
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
pub fn send_dialog_item_message_a(
    _hdlg: HWND,
    _dlg_item_id: i32,
    _message: u32,
    _wparam: WPARAM,
    _lparam: LPARAM,
) -> LRESULT {
    let child = find_dialog_item(_hdlg, _dlg_item_id);
    if child.is_null() {
        return 0;
    }

    match _message {
        window_message::WM_SETTEXT => {
            let src = _lparam as *const i8;
            let text = if src.is_null() {
                String::new()
            } else {
                unsafe { std::ffi::CStr::from_ptr(src) }
                    .to_string_lossy()
                    .into_owned()
            };

            if set_window_title(child, &text) { 1 } else { 0 }
        }
        window_message::WM_GETTEXT => {
            let text = get_window_title(child).unwrap_or_default();
            unsafe { write_ansi_text(_lparam as *mut u8, _wparam as i32, &text) as isize }
        }
        window_message::WM_GETTEXTLENGTH => get_window_title(child)
            .map(|text| text.len() as isize)
            .unwrap_or(0),
        _ => {
            warn!(
                "SendDlgItemMessageA message {:#x} is not implemented yet. Returning 0.",
                _message
            );
            0
        }
    }
}

pub fn send_dialog_item_message_w(
    _hdlg: HWND,
    _dlg_item_id: i32,
    _message: u32,
    _wparam: WPARAM,
    _lparam: LPARAM,
) -> LRESULT {
    let child = find_dialog_item(_hdlg, _dlg_item_id);
    if child.is_null() {
        return 0;
    }

    match _message {
        window_message::WM_SETTEXT => {
            let src = _lparam as *const u16;
            let text = if src.is_null() {
                String::new()
            } else {
                let mut len = 0usize;
                unsafe {
                    while *src.add(len) != 0 {
                        len += 1;
                    }
                    String::from_utf16_lossy(std::slice::from_raw_parts(src, len))
                }
            };

            if set_window_title(child, &text) { 1 } else { 0 }
        }
        window_message::WM_GETTEXT => {
            let text = get_window_title(child).unwrap_or_default();
            unsafe { write_wide_text(_lparam as *mut u16, _wparam as i32, &text) as isize }
        }
        window_message::WM_GETTEXTLENGTH => get_window_title(child)
            .map(|text| text.encode_utf16().count() as isize)
            .unwrap_or(0),
        _ => {
            warn!(
                "SendDlgItemMessageW message {:#x} is not implemented yet. Returning 0.",
                _message
            );
            0
        }
    }
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
pub fn get_dialog_control_id(_hwnd: HWND) -> i32 {
    WINDOW_MANAGER.get_control_id(_hwnd).unwrap_or(0)
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
pub fn get_dialog_item_text_a(
    _hdlg: HWND,
    _dlg_item_id: i32,
    _buffer: LPCSTR,
    _max_text_length: i32,
) -> u32 {
    let child = find_dialog_item(_hdlg, _dlg_item_id);
    if child.is_null() {
        return 0;
    }

    let text = get_window_title(child).unwrap_or_default();
    unsafe { write_ansi_text(_buffer.as_ptr() as *mut u8, _max_text_length, &text) }
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
pub fn get_dialog_item_text_w(
    _hdlg: HWND,
    _dlg_item_id: i32,
    _buffer: LPCWSTR,
    _max_text_length: i32,
) -> u32 {
    let child = find_dialog_item(_hdlg, _dlg_item_id);
    if child.is_null() {
        return 0;
    }

    let text = get_window_title(child).unwrap_or_default();
    unsafe { write_wide_text(_buffer.as_ptr() as *mut u16, _max_text_length, &text) }
}

/// Ends a modal dialog box, causing the system to end any processing for the dialog box.
///
/// # Arguments
/// * `_hdlg` - A handle to the dialog box.
/// * `_result` - The value to be returned to the dialog box's owner window in the `lParam` parameter
///   of the `WM_COMMAND` message when the dialog box is closed.
///
/// # Safety
/// `_hdlg` must be a valid handle to a dialog box.
/// `_result` must be a valid value to be returned to the dialog box's owner window in the `lParam` parameter
///   of the `WM_COMMAND` message when the dialog box is closed.
///
/// # Returns
/// If the function succeeds, the return value is `BOOL::TRUE`.
/// If the function fails, the return value is `BOOL::FALSE`.
/// On failure, to get extended error information, call `GetLastError`.
///
/// # Notes
/// The current implementation is a stub and always returns `FALSE`.
/// We currently do not set the value of `GetLastError` on failure.
pub fn end_dialog(_hdlg: HWND, _result: isize) -> BOOL {
    warn!(
        "EndDialog is not implemented yet. This is a stub implementation that always returns FALSE."
    );
    BOOL::FALSE
}

// ---------------------------------------------------------------------------
// MessageBoxA constants
// ---------------------------------------------------------------------------

// Button type flags
pub const MB_OK: u32 = 0x00000000;
pub const MB_OKCANCEL: u32 = 0x00000001;
pub const MB_ABORTRETRYIGNORE: u32 = 0x00000002;
pub const MB_YESNOCANCEL: u32 = 0x00000003;
pub const MB_YESNO: u32 = 0x00000004;
pub const MB_RETRYCANCEL: u32 = 0x00000005;
pub const MB_CANCELTRYCONTINUE: u32 = 0x00000006;
pub const MB_HELP: u32 = 0x00004000;

// Icon type flags
pub const MB_ICONHAND: u32 = 0x00000010;
pub const MB_ICONSTOP: u32 = MB_ICONHAND;
pub const MB_ICONERROR: u32 = MB_ICONHAND;
pub const MB_ICONQUESTION: u32 = 0x00000020;
pub const MB_ICONEXCLAMATION: u32 = 0x00000030;
pub const MB_ICONWARNING: u32 = MB_ICONEXCLAMATION;
pub const MB_ICONASTERISK: u32 = 0x00000040;
pub const MB_ICONINFORMATION: u32 = MB_ICONASTERISK;

// Default button flags
pub const MB_DEFBUTTON1: u32 = 0x00000000;
pub const MB_DEFBUTTON2: u32 = 0x00000100;
pub const MB_DEFBUTTON3: u32 = 0x00000200;
pub const MB_DEFBUTTON4: u32 = 0x00000300;

// Modality flags
pub const MB_APPLMODAL: u32 = 0x00000000;
pub const MB_SYSTEMMODAL: u32 = 0x00001000;
pub const MB_TASKMODAL: u32 = 0x00002000;

// Misc flags
pub const MB_SETFOREGROUND: u32 = 0x00010000;
pub const MB_TOPMOST: u32 = 0x00040000;
pub const MB_SERVICE_NOTIFICATION: u32 = 0x00200000;

// Return values
pub const IDOK: i32 = 1;
pub const IDCANCEL: i32 = 2;
pub const IDABORT: i32 = 3;
pub const IDRETRY: i32 = 4;
pub const IDIGNORE: i32 = 5;
pub const IDYES: i32 = 6;
pub const IDNO: i32 = 7;
pub const IDTRYAGAIN: i32 = 10;
pub const IDCONTINUE: i32 = 11;

/// Displays a modal dialog box that contains a system icon, a set of buttons, and a brief
/// application-specific message, such as status or error information. The message box returns
/// an integer value that indicates which button the user clicked.
///
/// # Arguments
/// * `_hwnd` - A handle to the owner window of the message box. If this parameter is `HWND::NULL`,
///   the message box has no owner window.
/// * `text` - The message to be displayed.
/// * `caption` - The dialog box title. If this parameter is `None`, the default title is used.
/// * `u_type` - The contents and behavior of the dialog box. This parameter can be a combination
///   of flags from the MB_* constants.
///
/// # Returns
/// If the function succeeds, the return value is one of the ID* constants indicating which button
/// was pressed. If there is not enough memory to create the message box, the return value is `0`.
///
/// # Notes
/// This implementation uses `rfd` for native message dialogs. Not all button combinations
/// supported by Windows are exposed by `rfd`; unsupported combinations fall back to a simple
/// OK dialog and log a warning.
pub fn message_box(_hwnd: HWND, text: &str, caption: &str, u_type: u32) -> i32 {
    use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};

    let buttons = u_type & 0x0F;
    let icon = u_type & 0xF0;

    let level = match icon {
        MB_ICONHAND => MessageLevel::Error,
        MB_ICONQUESTION => MessageLevel::Info,
        MB_ICONEXCLAMATION => MessageLevel::Warning,
        MB_ICONASTERISK => MessageLevel::Info,
        _ => MessageLevel::Info,
    };

    let rfd_buttons = match buttons {
        MB_OK => MessageButtons::Ok,
        MB_OKCANCEL => MessageButtons::OkCancel,
        MB_YESNO => MessageButtons::YesNo,
        MB_YESNOCANCEL => MessageButtons::YesNoCancel,
        _ => {
            warn!(
                "MessageBoxA: unsupported button combination (u_type={:#x}), falling back to OK.",
                u_type
            );
            MessageButtons::Ok
        }
    };

    let result = MessageDialog::new()
        .set_level(level)
        .set_title(caption)
        .set_description(text)
        .set_buttons(rfd_buttons)
        .show();

    match result {
        MessageDialogResult::Ok => IDOK,
        MessageDialogResult::Cancel => IDCANCEL,
        MessageDialogResult::Yes => IDYES,
        MessageDialogResult::No => IDNO,
        MessageDialogResult::Custom(_) => IDOK,
    }
}
