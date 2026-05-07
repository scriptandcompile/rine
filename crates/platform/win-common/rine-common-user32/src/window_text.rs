//! Window text operations — shared logic for SetWindowText, GetWindowText(Length).

use rine_types::errors::BOOL;
use rine_types::windows::*;

use crate::backend::set_native_title;

/// SetWindowText — set the title bar text for a window.
///
/// Returns `BOOL::TRUE` on success, `BOOL::FALSE` if the HWND is not found.
pub fn set_window_text(hwnd: Hwnd, text: String) -> BOOL {
    let backend_title = text.clone();

    WINDOW_MANAGER.update_window(hwnd, |state| {
        state.title = text;
    });

    set_native_title(hwnd, &backend_title);

    BOOL::TRUE
}

/// GetWindowTextA — copy the window title into an ANSI buffer.
///
/// Returns the number of characters copied, excluding the null terminator.
///
/// # Safety
/// `buffer` must point to at least `max_count` bytes of writable memory.
pub unsafe fn get_window_text_a(hwnd: Hwnd, buffer: *mut u8, max_count: i32) -> i32 {
    if buffer.is_null() || max_count <= 0 {
        return 0;
    }

    let title = match WINDOW_MANAGER.get_window(hwnd) {
        Some(state) => state.title,
        None => return 0,
    };

    let bytes = title.as_bytes();
    let copy_len = bytes.len().min((max_count - 1) as usize);

    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer, copy_len);
        *buffer.add(copy_len) = 0;
    }

    copy_len as i32
}

/// GetWindowTextW — copy the window title into a UTF-16 buffer.
///
/// Returns the number of code units copied, excluding the null terminator.
///
/// # Safety
/// `buffer` must point to at least `max_count` u16s of writable memory.
pub unsafe fn get_window_text_w(hwnd: Hwnd, buffer: *mut u16, max_count: i32) -> i32 {
    if buffer.is_null() || max_count <= 0 {
        return 0;
    }

    let title = match WINDOW_MANAGER.get_window(hwnd) {
        Some(state) => state.title,
        None => return 0,
    };

    let wide: Vec<u16> = title.encode_utf16().collect();
    let copy_len = wide.len().min((max_count - 1) as usize);

    unsafe {
        std::ptr::copy_nonoverlapping(wide.as_ptr(), buffer, copy_len);
        *buffer.add(copy_len) = 0;
    }

    copy_len as i32
}

/// GetWindowTextLength(A/W) — return the number of characters in the window title.
///
/// Returns the character count, not counting the null terminator.
pub fn get_window_text_length(hwnd: Hwnd) -> i32 {
    match WINDOW_MANAGER.get_window(hwnd) {
        Some(state) => state.title.len() as i32,
        None => 0,
    }
}
