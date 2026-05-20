//! Window long operations — shared logic for SetWindowLongA/SetWindowLongPtrA.

use rine_types::windows::{HWND, WINDOW_MANAGER, window_long_index};

/// SetWindowLongA — change an attribute of the specified window.
///
/// This function sets a 32-bit value at the specified offset into the
/// per-window extra memory or into one of the predefined window-long
/// positions (style, ex_style, wnd_proc, user_data, parent, etc.).
///
/// # Arguments
/// * `hwnd` - Handle to the window.
/// * `n_index` - One of the `GWL_*` / `GWLP_*` / `DWLP_*` constants
///   defined in `window_long_index`.
/// * `dw_new_long` - The replacement value (32-bit).
///
/// # Returns
/// The previous value of the specified window long, or 0 if the HWND is
/// not found. Note that 0 can also be a valid previous value; callers
/// should check `GetLastError()` to distinguish success from failure.
/// Currently we do not set `GetLastError`.
///
/// # Notes
/// Unimplemented index values (accepted but return 0):
/// - `GWL_ID` (-12) — not tracked per-window in `WindowState`.
/// - `DWLP_MSGRESULT` (0) — not tracked per-window in `WindowState`.
/// - `DWLP_DLGPROC` (4) — not tracked per-window in `WindowState`.
/// - `DWLP_USER` (8) — not tracked per-window in `WindowState`.
/// - `GWLP_HINSTANCE` (-6) — not tracked per-window in `WindowState`.
///
/// Missing implementation features:
/// - No `SetLastError` is called on failure.
/// - No `WM_STYLECHANGING` / `WM_STYLECHANGED` messages are sent when
///   `GWL_STYLE` or `GWL_EXSTYLE` is changed.
/// - No subclassing bookkeeping is performed for `GWLP_WNDPROC`.
pub fn set_window_long_a(hwnd: HWND, n_index: i32, dw_new_long: i32) -> i32 {
    let state = WINDOW_MANAGER.get_window(hwnd);
    let Some(state) = state else {
        return 0;
    };

    match n_index {
        window_long_index::GWL_STYLE => {
            let old = state.style as i32;
            WINDOW_MANAGER.update_window(hwnd, |s| s.style = dw_new_long as u32);

            old
        }
        window_long_index::GWL_EXSTYLE => {
            let old = state.ex_style as i32;
            WINDOW_MANAGER.update_window(hwnd, |s| s.ex_style = dw_new_long as u32);

            old
        }
        window_long_index::GWLP_WNDPROC => {
            let old = state.wnd_proc as i32;
            WINDOW_MANAGER.update_window(hwnd, |s| s.wnd_proc = dw_new_long as usize);

            old
        }
        window_long_index::GWLP_HWNDPARENT => {
            let old = state.parent.as_raw() as i32;
            WINDOW_MANAGER.update_window(hwnd, |s| s.parent = HWND::from_raw(dw_new_long as usize));

            old
        }
        window_long_index::GWLP_USERDATA => {
            let old = state.user_data as i32;
            WINDOW_MANAGER.update_window(hwnd, |s| s.user_data = dw_new_long as usize);

            old
        }
        // Unimplemented indices — stub returns 0 without modifying state.
        window_long_index::GWL_ID
        | window_long_index::GWLP_HINSTANCE
        | window_long_index::DWLP_MSGRESULT
        | window_long_index::DWLP_DLGPROC
        | window_long_index::DWLP_USER => 0,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rine_types::windows::WindowState;
    use rine_types::windows::window_long_index::*;
    use rine_types::windows::window_style;
    use std::sync::{LazyLock, Mutex};

    static TEST_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    fn test_window_state() -> WindowState {
        WindowState {
            hwnd: HWND::NULL,
            class_name: "test-window-long".to_owned(),
            title: String::new(),
            style: window_style::WS_OVERLAPPEDWINDOW,
            ex_style: 0,
            rect: Default::default(),
            client_rect: Default::default(),
            parent: HWND::NULL,
            visible: true,
            enabled: true,
            wnd_proc: 0x1234_5678,
            user_data: 0xDEAD_BEEF,
            menu: None,
            system_menu: None,
        }
    }

    #[test]
    fn set_gwl_style() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state());
        let prev = set_window_long_a(hwnd, GWL_STYLE, window_style::WS_POPUP as i32);
        assert_eq!(prev, window_style::WS_OVERLAPPEDWINDOW as i32);
        let state = WINDOW_MANAGER.get_window(hwnd).unwrap();
        assert_eq!(state.style, window_style::WS_POPUP);
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn set_gwl_ex_style() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state());
        let prev = set_window_long_a(hwnd, GWL_EXSTYLE, 0x123);
        assert_eq!(prev, 0);
        let state = WINDOW_MANAGER.get_window(hwnd).unwrap();
        assert_eq!(state.ex_style, 0x123);
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn set_gwlp_wndproc() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state());
        let prev = set_window_long_a(hwnd, GWLP_WNDPROC, 0x42);
        assert_eq!(prev, 0x1234_5678i32);
        let state = WINDOW_MANAGER.get_window(hwnd).unwrap();
        assert_eq!(state.wnd_proc, 0x42);
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn set_gwlp_userdata() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state());
        let prev = set_window_long_a(hwnd, GWLP_USERDATA, 0x42);
        assert_eq!(prev, 0xDEAD_BEEFu32 as i32);
        let state = WINDOW_MANAGER.get_window(hwnd).unwrap();
        assert_eq!(state.user_data, 0x42);
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn set_gwlp_hwndparent() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state());
        let parent = HWND::from_raw(0xCAFE);
        let prev = set_window_long_a(hwnd, GWLP_HWNDPARENT, parent.as_raw() as i32);
        assert_eq!(prev, 0);
        let state = WINDOW_MANAGER.get_window(hwnd).unwrap();
        assert_eq!(state.parent, parent);
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn set_unimplemented_index_returns_zero() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state());
        let prev = set_window_long_a(hwnd, GWL_ID, 999);
        assert_eq!(prev, 0);
        let prev = set_window_long_a(hwnd, DWLP_USER, 999);
        assert_eq!(prev, 0);
        let prev = set_window_long_a(hwnd, DWLP_MSGRESULT, 999);
        assert_eq!(prev, 0);
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn set_window_long_invalid_hwnd_returns_zero() {
        let prev = set_window_long_a(HWND::NULL, GWL_STYLE, 0);
        assert_eq!(prev, 0);
    }
}
