//! Window text operations — shared logic for SetWindowText, GetWindowText(Length).

use rine_types::errors::BOOL;
use rine_types::windows::*;

use crate::backend::set_native_title;

/// SetWindowText — set the title bar text for a window.
///
/// Returns `BOOL::TRUE` on success, `BOOL::FALSE` if the HWND is not found.
pub fn set_window_text(hwnd: HWND, text: String) -> BOOL {
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
pub unsafe fn get_window_text_a(hwnd: HWND, buffer: *mut u8, max_count: i32) -> i32 {
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
pub unsafe fn get_window_text_w(hwnd: HWND, buffer: *mut u16, max_count: i32) -> i32 {
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
pub fn get_window_text_length(hwnd: HWND) -> i32 {
    match WINDOW_MANAGER.get_window(hwnd) {
        Some(state) => state.title.len() as i32,
        None => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{LazyLock, Mutex};

    static TEST_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    fn test_window_state(title: &str) -> WindowState {
        WindowState {
            hwnd: HWND::NULL,
            class_name: "test-window-text".to_owned(),
            title: title.to_owned(),
            style: 0,
            ex_style: 0,
            rect: Rect::default(),
            client_rect: Rect::default(),
            parent: HWND::NULL,
            visible: true,
            enabled: true,
            wnd_proc: 0,
            user_data: 0,
            menu: None,
            system_menu: None,
        }
    }

    #[test]
    fn set_window_text_updates_title() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state(""));
        assert_eq!(set_window_text(hwnd, "new title".to_owned()), BOOL::TRUE);
        let state = WINDOW_MANAGER.get_window(hwnd).unwrap();
        assert_eq!(state.title, "new title");
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn set_window_text_replaces_existing() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state("old"));
        assert_eq!(set_window_text(hwnd, "updated".to_owned()), BOOL::TRUE);
        let state = WINDOW_MANAGER.get_window(hwnd).unwrap();
        assert_eq!(state.title, "updated");
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn get_window_text_a_copies_ansi_title() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state("Hello"));
        let mut buf = [0u8; 16];
        let len = unsafe { get_window_text_a(hwnd, buf.as_mut_ptr(), 16) };
        assert_eq!(len, 5);
        assert_eq!(&buf[..6], b"Hello\0");
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn get_window_text_a_truncates_to_buffer() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state("Hello World"));
        let mut buf = [0u8; 6];
        let len = unsafe { get_window_text_a(hwnd, buf.as_mut_ptr(), 6) };
        assert_eq!(len, 5);
        assert_eq!(&buf[..6], b"Hello\0");
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn get_window_text_a_null_buffer_returns_zero() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state("test"));
        let len = unsafe { get_window_text_a(hwnd, std::ptr::null_mut(), 16) };
        assert_eq!(len, 0);
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn get_window_text_a_zero_max_count_returns_zero() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state("test"));
        let mut buf = [0u8; 16];
        let len = unsafe { get_window_text_a(hwnd, buf.as_mut_ptr(), 0) };
        assert_eq!(len, 0);
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn get_window_text_a_negative_max_count_returns_zero() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state("test"));
        let mut buf = [0u8; 16];
        let len = unsafe { get_window_text_a(hwnd, buf.as_mut_ptr(), -1) };
        assert_eq!(len, 0);
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn get_window_text_a_invalid_hwnd_returns_zero() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let mut buf = [0u8; 16];
        let len = unsafe { get_window_text_a(HWND::NULL, buf.as_mut_ptr(), 16) };
        assert_eq!(len, 0);
    }

    #[test]
    fn get_window_text_w_copies_wide_title() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state("Hi"));
        let mut buf = [0u16; 16];
        let len = unsafe { get_window_text_w(hwnd, buf.as_mut_ptr(), 16) };
        assert_eq!(len, 2);
        assert_eq!(buf[0], b'H' as u16);
        assert_eq!(buf[1], b'i' as u16);
        assert_eq!(buf[2], 0);
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn get_window_text_w_truncates_to_buffer() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state("ABCDEFGHIJ"));
        let mut buf = [0u16; 4];
        let len = unsafe { get_window_text_w(hwnd, buf.as_mut_ptr(), 4) };
        assert_eq!(len, 3);
        assert_eq!(buf[0], b'A' as u16);
        assert_eq!(buf[1], b'B' as u16);
        assert_eq!(buf[2], b'C' as u16);
        assert_eq!(buf[3], 0);
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn get_window_text_w_null_buffer_returns_zero() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state("test"));
        let len = unsafe { get_window_text_w(hwnd, std::ptr::null_mut(), 16) };
        assert_eq!(len, 0);
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn get_window_text_w_zero_max_count_returns_zero() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state("test"));
        let mut buf = [0u16; 16];
        let len = unsafe { get_window_text_w(hwnd, buf.as_mut_ptr(), 0) };
        assert_eq!(len, 0);
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn get_window_text_w_invalid_hwnd_returns_zero() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let mut buf = [0u16; 16];
        let len = unsafe { get_window_text_w(HWND::NULL, buf.as_mut_ptr(), 16) };
        assert_eq!(len, 0);
    }

    #[test]
    fn get_window_text_length_valid_window() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state("Hello"));
        assert_eq!(get_window_text_length(hwnd), 5);
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn get_window_text_length_empty_title() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state(""));
        assert_eq!(get_window_text_length(hwnd), 0);
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn get_window_text_length_invalid_hwnd_returns_zero() {
        assert_eq!(get_window_text_length(HWND::NULL), 0);
    }

    #[test]
    fn set_window_text_updates_length() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state(""));
        assert_eq!(get_window_text_length(hwnd), 0);
        set_window_text(hwnd, "updated".to_owned());
        assert_eq!(get_window_text_length(hwnd), 7);
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn get_window_text_a_and_w_match() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state("Hello"));
        let mut a_buf = [0u8; 16];
        let mut w_buf = [0u16; 16];
        let a_len = unsafe { get_window_text_a(hwnd, a_buf.as_mut_ptr(), 16) };
        let w_len = unsafe { get_window_text_w(hwnd, w_buf.as_mut_ptr(), 16) };
        assert_eq!(a_len as u32, w_len as u32);
        assert_eq!(a_len, 5);
        for i in 0..a_len as usize {
            assert_eq!(a_buf[i] as u16, w_buf[i]);
        }
        WINDOW_MANAGER.destroy_window(hwnd);
    }
}
