//! Focus management — GetFocus / SetFocus shared logic.

use rine_types::windows::*;

/// GetFocus — retrieves the handle of the window with keyboard focus.
///
/// Returns the HWND of the focused window, or HWND::NULL if no window
/// in the calling thread's message queue has focus.
pub fn get_focus() -> HWND {
    WINDOW_MANAGER.get_focus()
}

/// SetFocus — sets keyboard focus to the specified window.
///
/// Posts WM_KILLFOCUS to the previously focused window and WM_SETFOCUS
/// to the newly focused window.
///
/// Returns the handle of the window that previously had focus, or
/// HWND::NULL if there was no previously focused window.
pub fn set_focus(hwnd: HWND) -> HWND {
    let prev = WINDOW_MANAGER.set_focus(hwnd);

    THREAD_MESSAGE_QUEUE.with(|queue| {
        if !prev.is_null() {
            queue.post_message(Msg {
                hwnd: prev,
                message: window_message::WM_KILLFOCUS,
                w_param: hwnd.as_raw(),
                l_param: 0,
                time: 0,
                pt: Point::default(),
            });
        }
        if !hwnd.is_null() {
            queue.post_message(Msg {
                hwnd,
                message: window_message::WM_SETFOCUS,
                w_param: prev.as_raw(),
                l_param: 0,
                time: 0,
                pt: Point::default(),
            });
        }
    });

    prev
}

#[cfg(test)]
mod tests {
    use super::*;

    use serial_test::serial;
    use std::sync::{LazyLock, Mutex};

    static TEST_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    fn test_window_state() -> WindowState {
        WindowState {
            hwnd: HWND::NULL,
            class_name: "test-focus".to_owned(),
            title: "".to_owned(),
            style: 0,
            ex_style: 0,
            rect: Rect::default(),
            client_rect: Rect::default(),
            parent: HWND::NULL,
            visible: true,
            enabled: true,
            wnd_proc: 0,
            user_data: 0,
        }
    }

    fn drain_message_queue() {
        THREAD_MESSAGE_QUEUE.with(|queue| {
            let mut msg = Msg {
                hwnd: HWND::NULL,
                message: 0,
                w_param: 0,
                l_param: 0,
                time: 0,
                pt: Point::default(),
            };
            while queue.peek_message(&mut msg, true) {}
        });
    }

    fn reset_state() {
        set_focus(HWND::NULL);
        drain_message_queue();
    }

    #[test]
    #[serial]
    fn get_focus_returns_null_initially() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();
        assert_eq!(get_focus(), HWND::NULL);
    }

    #[test]
    #[serial]
    fn set_focus_returns_null_when_no_previous_focus() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state());
        assert_eq!(set_focus(hwnd), HWND::NULL);
        reset_state();
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    #[serial]
    fn set_focus_and_get_focus_roundtrip() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state());
        set_focus(hwnd);
        assert_eq!(get_focus(), hwnd);
        reset_state();
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    #[serial]
    fn set_focus_returns_previous_focus() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();
        let a = WINDOW_MANAGER.create_window(test_window_state());
        let b = WINDOW_MANAGER.create_window(test_window_state());
        set_focus(a);
        let prev = set_focus(b);
        assert_eq!(prev, a);
        reset_state();
        WINDOW_MANAGER.destroy_window(a);
        WINDOW_MANAGER.destroy_window(b);
    }

    #[test]
    #[serial]
    fn set_focus_null_clears_focus() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state());
        set_focus(hwnd);
        assert_eq!(get_focus(), hwnd);
        let prev = set_focus(HWND::NULL);
        assert_eq!(prev, hwnd);
        assert_eq!(get_focus(), HWND::NULL);
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    #[serial]
    fn set_focus_null_when_already_null_returns_null() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();
        assert_eq!(set_focus(HWND::NULL), HWND::NULL);
        assert_eq!(get_focus(), HWND::NULL);
    }

    #[test]
    #[serial]
    fn set_focus_posts_wm_setfocus_to_new_window() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state());
        set_focus(hwnd);
        let mut msg = Msg {
            hwnd: HWND::NULL,
            message: 0,
            w_param: 0,
            l_param: 0,
            time: 0,
            pt: Point::default(),
        };
        let found = THREAD_MESSAGE_QUEUE.with(|queue| queue.peek_message(&mut msg, false));
        assert!(found);
        assert_eq!(msg.hwnd, hwnd);
        assert_eq!(msg.message, window_message::WM_SETFOCUS);
        reset_state();
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    #[serial]
    fn set_focus_posts_wm_killfocus_to_previous_window() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();
        let a = WINDOW_MANAGER.create_window(test_window_state());
        let b = WINDOW_MANAGER.create_window(test_window_state());
        set_focus(a);
        set_focus(b);
        let mut msgs: Vec<Msg> = Vec::new();
        THREAD_MESSAGE_QUEUE.with(|queue| {
            let mut msg = Msg {
                hwnd: HWND::NULL,
                message: 0,
                w_param: 0,
                l_param: 0,
                time: 0,
                pt: Point::default(),
            };
            while queue.peek_message(&mut msg, true) {
                msgs.push(msg);
                msg = Msg {
                    hwnd: HWND::NULL,
                    message: 0,
                    w_param: 0,
                    l_param: 0,
                    time: 0,
                    pt: Point::default(),
                };
            }
        });
        let killfocus = msgs
            .iter()
            .find(|m| m.message == window_message::WM_KILLFOCUS);
        assert!(killfocus.is_some());
        assert_eq!(killfocus.unwrap().hwnd, a);
        reset_state();
        WINDOW_MANAGER.destroy_window(a);
        WINDOW_MANAGER.destroy_window(b);
    }
}
