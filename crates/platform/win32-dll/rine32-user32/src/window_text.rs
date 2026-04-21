use rine_common_user32 as common;
use rine_types::errors::WinBool;
use rine_types::strings::{read_cstr, read_wstr};

/// Set the title bar text for a window.
///
/// # Arguments
/// * `hwnd` - Handle to the window.
/// * `text` - New title text as a C-style string (null-terminated).
///
/// # Safety
/// * `text` must be a valid pointer to a null-terminated string.
/// * The function assumes the caller has the right to modify the window's title.
///
/// # Returns
/// `WinBool::TRUE` on success, `WinBool::FALSE` if the HWND is not found.
///
/// # Notes
/// Currently, this function does not perform any access checks on the window handle (HWND).
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn SetWindowTextA(hwnd: usize, text: *const u8) -> WinBool {
    common::set_window_text(hwnd, read_cstr(text).unwrap_or_default())
}

/// Set the title bar text for a window.
///
/// # Arguments
/// * `hwnd` - Handle to the window.
/// * `text` - New title text as a C-style string (null-terminated).
///
/// # Safety
/// * `text` must be a valid pointer to a null-terminated string.
/// * The function assumes the caller has the right to modify the window's title.
///
/// # Returns
/// `WinBool::TRUE` on success, `WinBool::FALSE` if the HWND is not found.
///
/// # Notes
/// Currently, this function does not perform any access checks on the window handle (HWND).
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn SetWindowTextW(hwnd: usize, text: *const u16) -> WinBool {
    common::set_window_text(hwnd, read_wstr(text).unwrap_or_default())
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn get_window_text_a(
    hwnd: usize,
    buffer: *mut u8,
    max_count: i32,
) -> i32 {
    unsafe { common::get_window_text_a(hwnd, buffer, max_count) }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn get_window_text_w(
    hwnd: usize,
    buffer: *mut u16,
    max_count: i32,
) -> i32 {
    unsafe { common::get_window_text_w(hwnd, buffer, max_count) }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn get_window_text_length_a(hwnd: usize) -> i32 {
    common::get_window_text_length(hwnd)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn get_window_text_length_w(hwnd: usize) -> i32 {
    common::get_window_text_length(hwnd)
}
