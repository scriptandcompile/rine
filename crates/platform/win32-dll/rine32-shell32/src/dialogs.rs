use rine_common_shell32::dialogs as common;

use rine_types::handles::Handle;
use rine_types::strings::{LPCSTR, LPCWSTR};
use rine_types::windows::HWND;

/// Displays a ShellAbout dialog box.
///
/// # Arguments
/// * `_hwnd` - A handle to the parent window. This parameter can be `HWND::NULL`.
/// * `_sz_app` - App/title text.
/// * `_sz_other_stuff` - Optional extra text shown in the dialog body.
/// * `_h_icon` - Optional icon handle.
///
/// # Safety
/// `_sz_app` and `_sz_other_stuff` must be valid null-terminated ANSI strings when non-null.
///
/// # Return
/// Nonzero on success, zero on failure.
/// # Notes
/// This implementation applies the documented text-layout split between
/// Windows 2000/XP/Server 2003 and Windows Vista/Server 2008+.
/// The actual dialog display is not implemented, but the text layout logic is applied and can be observed in debug logs.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn ShellAboutA(
    _hwnd: HWND,
    _sz_app: LPCSTR,
    _sz_other_stuff: LPCSTR,
    _h_icon: Handle,
) -> i32 {
    let app_text = if _sz_app.is_null() {
        None
    } else {
        unsafe { _sz_app.read_string() }
    };

    let other_stuff = if _sz_other_stuff.is_null() {
        None
    } else {
        unsafe { _sz_other_stuff.read_string() }
    };

    common::shell_about(_hwnd, app_text.as_deref(), other_stuff.as_deref(), _h_icon).as_i32()
}

/// Displays a ShellAbout dialog box.
///
/// # Arguments
/// * `_hwnd` - A handle to the parent window. This parameter can be `HWND::NULL`.
/// * `_sz_app` - App/title text.
/// * `_sz_other_stuff` - Optional extra text shown in the dialog body.
/// * `_h_icon` - Optional icon handle.
///
/// # Safety
/// `_sz_app` and `_sz_other_stuff` must be valid null-terminated ANSI strings when non-null.
///
/// # Return
/// Nonzero on success, zero on failure.
///
/// # Notes
/// This implementation applies the documented text-layout split between
/// Windows 2000/XP/Server 2003 and Windows Vista/Server 2008+.
/// The actual dialog display is not implemented, but the text layout logic is applied and can be observed in debug logs.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn ShellAboutW(
    _hwnd: HWND,
    _sz_app: LPCWSTR,
    _sz_other_stuff: LPCWSTR,
    _h_icon: Handle,
) -> i32 {
    let app_text = if _sz_app.is_null() {
        None
    } else {
        unsafe { _sz_app.read_string() }
    };

    let other_stuff = if _sz_other_stuff.is_null() {
        None
    } else {
        unsafe { _sz_other_stuff.read_string() }
    };

    common::shell_about(_hwnd, app_text.as_deref(), other_stuff.as_deref(), _h_icon).as_i32()
}
