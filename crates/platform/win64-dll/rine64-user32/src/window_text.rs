use rine_types::strings::{read_cstr, read_wstr};
use rine_types::windows::*;

use crate::backend::set_native_title;

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn set_window_text_a(hwnd: usize, text: *const u8) -> i32 {
    let hwnd = Hwnd::from_raw(hwnd);
    let text_str = read_cstr(text).unwrap_or_default();
    let backend_title = text_str.clone();

    WINDOW_MANAGER.update_window(hwnd, |state| {
        state.title = text_str;
    });

    set_native_title(hwnd, &backend_title);

    1
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn set_window_text_w(hwnd: usize, text: *const u16) -> i32 {
    let hwnd = Hwnd::from_raw(hwnd);
    let text_str = read_wstr(text).unwrap_or_default();
    let backend_title = text_str.clone();

    WINDOW_MANAGER.update_window(hwnd, |state| {
        state.title = text_str;
    });

    set_native_title(hwnd, &backend_title);

    1
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn get_window_text_a(
    hwnd: usize,
    buffer: *mut u8,
    max_count: i32,
) -> i32 {
    if buffer.is_null() || max_count <= 0 {
        return 0;
    }

    let hwnd = Hwnd::from_raw(hwnd);

    let title = match WINDOW_MANAGER.get_window(hwnd) {
        Some(state) => state.title,
        None => return 0,
    };

    let bytes = title.as_bytes();
    let copy_len = bytes.len().min((max_count - 1) as usize);

    std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer, copy_len);
    *buffer.add(copy_len) = 0;

    copy_len as i32
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn get_window_text_w(
    hwnd: usize,
    buffer: *mut u16,
    max_count: i32,
) -> i32 {
    if buffer.is_null() || max_count <= 0 {
        return 0;
    }

    let hwnd = Hwnd::from_raw(hwnd);

    let title = match WINDOW_MANAGER.get_window(hwnd) {
        Some(state) => state.title,
        None => return 0,
    };

    let wide: Vec<u16> = title.encode_utf16().collect();
    let copy_len = wide.len().min((max_count - 1) as usize);

    std::ptr::copy_nonoverlapping(wide.as_ptr(), buffer, copy_len);
    *buffer.add(copy_len) = 0;

    copy_len as i32
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn get_window_text_length_a(hwnd: usize) -> i32 {
    let hwnd = Hwnd::from_raw(hwnd);

    match WINDOW_MANAGER.get_window(hwnd) {
        Some(state) => state.title.len() as i32,
        None => 0,
    }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn get_window_text_length_w(hwnd: usize) -> i32 {
    let hwnd = Hwnd::from_raw(hwnd);

    match WINDOW_MANAGER.get_window(hwnd) {
        Some(state) => state.title.encode_utf16().count() as i32,
        None => 0,
    }
}
