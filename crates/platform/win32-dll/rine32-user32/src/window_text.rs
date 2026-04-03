use rine_common_user32 as common;
use rine_types::strings::{read_cstr, read_wstr};

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn set_window_text_a(hwnd: usize, text: *const u8) -> i32 {
    common::set_window_text(hwnd, read_cstr(text).unwrap_or_default())
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn set_window_text_w(hwnd: usize, text: *const u16) -> i32 {
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
