//! Window lifecycle — shared logic for CreateWindowEx, DestroyWindow, ShowWindow, UpdateWindow.

use rine_types::windows::*;
use rine_types::{errors::BOOL, strings::json_escape};

use crate::backend::{
    create_native_window, destroy_native_window, request_native_redraw, set_native_visibility,
};

/// Create a new window from pre-parsed Rust string arguments.
///
/// Returns the raw HWND on success, 0 if the class name is unregistered.
pub fn create_window(
    ex_style: u32,
    class_name: String,
    title: String,
    style: u32,
    rect: Rect,
    parent: HWND,
) -> HWND {
    let escaped_class_name = json_escape(&class_name);
    let escaped_title = json_escape(&title);

    let class = match WINDOW_CLASS_REGISTRY.get(&class_name) {
        Some(c) => c,
        None => return HWND::NULL,
    };

    let state = WindowState {
        hwnd: HWND::NULL,
        class_name: class_name.clone(),
        title: title.clone(),
        style,
        ex_style,
        rect,
        client_rect: Rect {
            left: 0,
            top: 0,
            right: rect.right - rect.left,
            bottom: rect.bottom - rect.top,
        },
        parent,
        visible: (style & window_style::WS_VISIBLE) != 0,
        enabled: (style & window_style::WS_DISABLED) == 0,
        wnd_proc: class.wnd_proc,
        user_data: 0,
    };

    let hwnd = WINDOW_MANAGER.create_window(state);

    if let Some(state) = WINDOW_MANAGER.get_window(hwnd) {
        create_native_window(hwnd, &state);
    }

    let detail = format!(
        r#"{{"hwnd":{},"title":"{}","class_name":"{}","parent":{}}}"#,
        hwnd.as_raw(),
        escaped_title,
        escaped_class_name,
        parent.as_raw(),
    );
    rine_types::dev_notify!(on_handle_created(hwnd.as_raw() as i64, "Window", &detail));

    THREAD_MESSAGE_QUEUE.with(|queue| {
        queue.post_message(Msg {
            hwnd,
            message: window_message::WM_CREATE,
            w_param: 0,
            l_param: 0,
            time: 0,
            pt: Point::default(),
        });
    });

    hwnd
}

/// Destroy a window.
///
/// Calls the window procedure with WM_DESTROY (using the provided callback
/// to respect the caller's ABI), tears down the native window, and removes
/// the window from the manager.
///
/// `call_wnd_proc` receives `(proc_fn, hwnd, msg, w_param, l_param)`.
///
/// # Safety
/// The caller must pass a valid window handle that belongs to this runtime and
/// provide a callback that can safely invoke the target window procedure.
///
/// Returns 1 on success, 0 if the HWND was not found.
pub unsafe fn destroy_window(
    hwnd: HWND,
    call_wnd_proc: impl Fn(usize, HWND, u32, usize, isize) -> isize,
) -> BOOL {
    let Some(state) = WINDOW_MANAGER.get_window(hwnd) else {
        return BOOL::FALSE;
    };

    // Deliver WM_DESTROY synchronously while the window state is still valid.
    // Apps commonly call PostQuitMessage from this handler.
    let _ = call_wnd_proc(state.wnd_proc, hwnd, window_message::WM_DESTROY, 0, 0);

    destroy_native_window(hwnd);

    if WINDOW_MANAGER.destroy_window(hwnd) {
        rine_types::dev_notify!(on_handle_closed(hwnd.as_raw() as i64));
        BOOL::TRUE
    } else {
        BOOL::FALSE
    }
}

/// Show or hide a window according to `cmd_show` (SW_* constants).
///
/// Returns `BOOL::TRUE` if the window was previously visible, `BOOL::FALSE` otherwise.
pub fn show_window(hwnd: HWND, cmd_show: i32) -> BOOL {
    let was_visible = WINDOW_MANAGER
        .get_window(hwnd)
        .map(|state| state.visible)
        .unwrap_or(false);

    WINDOW_MANAGER.update_window(hwnd, |state| match cmd_show {
        show_window::SW_HIDE => state.visible = false,
        show_window::SW_SHOWNORMAL | show_window::SW_SHOW | show_window::SW_SHOWDEFAULT => {
            state.visible = true;
            state.style &= !(window_style::WS_MINIMIZE | window_style::WS_MAXIMIZE);
        }
        show_window::SW_SHOWMINIMIZED => {
            state.visible = true;
            state.style |= window_style::WS_MINIMIZE;
            state.style &= !window_style::WS_MAXIMIZE;
        }
        show_window::SW_SHOWMAXIMIZED => {
            state.visible = true;
            state.style |= window_style::WS_MAXIMIZE;
            state.style &= !window_style::WS_MINIMIZE;
        }
        show_window::SW_RESTORE => {
            state.visible = true;
            state.style &= !(window_style::WS_MINIMIZE | window_style::WS_MAXIMIZE);
        }
        _ => {}
    });

    if let Some(state) = WINDOW_MANAGER.get_window(hwnd) {
        set_native_visibility(hwnd, state.visible);
        request_native_redraw(hwnd);
    }

    THREAD_MESSAGE_QUEUE.with(|queue| {
        queue.post_message(Msg {
            hwnd,
            message: window_message::WM_SHOWWINDOW,
            w_param: usize::from(!was_visible),
            l_param: cmd_show as isize,
            time: 0,
            pt: Point::default(),
        });
    });

    if was_visible { BOOL::TRUE } else { BOOL::FALSE }
}

/// Determine whether a window is minimized (iconic).
///
/// Returns `BOOL::TRUE` if the window has the `WS_MINIMIZE` style, `BOOL::FALSE` otherwise.
pub fn is_iconic(hwnd: HWND) -> BOOL {
    match WINDOW_MANAGER.get_window(hwnd) {
        Some(state) if state.style & window_style::WS_MINIMIZE != 0 => BOOL::TRUE,
        _ => BOOL::FALSE,
    }
}

/// Request a WM_PAINT for the given window.
///
/// # Arguments
/// * `hwnd`: Handle of the window to update.
///
/// # Safety
/// The caller must pass a valid window handle that belongs to this runtime.
/// The caller is responsible for ensuring that the window is not used after it has been destroyed,
/// as this would lead to undefined behavior.
/// The caller must also ensure that any necessary synchronization is performed if the window is
/// accessed from multiple threads.
///
/// # Returns
/// `BOOL::TRUE` always (UpdateWindow is a notification, not a query).
pub fn update_window(hwnd: HWND) -> BOOL {
    request_native_redraw(hwnd);

    THREAD_MESSAGE_QUEUE.with(|queue| {
        queue.post_message(Msg {
            hwnd,
            message: window_message::WM_PAINT,
            w_param: 0,
            l_param: 0,
            time: 0,
            pt: Point::default(),
        });
    });

    BOOL::TRUE
}

/// Mark the window's client area as needing a redraw.
///
/// `rect` and `erase` are accepted for API compatibility; the current
/// implementation always invalidates the entire window and posts `WM_PAINT`.
/// Returns `BOOL::TRUE` on success, `BOOL::FALSE` if the HWND is not found.
pub fn invalidate_rect(hwnd: HWND, _rect: Option<&Rect>, erase: BOOL) -> BOOL {
    if WINDOW_MANAGER.get_window(hwnd).is_none() {
        return BOOL::FALSE;
    }

    request_native_redraw(hwnd);

    THREAD_MESSAGE_QUEUE.with(|queue| {
        if erase == BOOL::TRUE {
            queue.post_message(Msg {
                hwnd,
                message: window_message::WM_ERASEBKGND,
                w_param: 0,
                l_param: 0,
                time: 0,
                pt: Point::default(),
            });
        }

        queue.post_message(Msg {
            hwnd,
            message: window_message::WM_PAINT,
            w_param: 0,
            l_param: 0,
            time: 0,
            pt: Point::default(),
        });
    });

    BOOL::TRUE
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{LazyLock, Mutex};

    static TEST_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    fn test_window_state() -> WindowState {
        WindowState {
            hwnd: HWND::NULL,
            class_name: "test-window-lifecycle".to_owned(),
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

    #[test]
    fn is_iconic_returns_false_for_normal_window() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state());
        assert_eq!(is_iconic(hwnd), BOOL::FALSE);
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn is_iconic_returns_true_for_minimized_window() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let state = WindowState {
            style: window_style::WS_MINIMIZE,
            ..test_window_state()
        };
        let hwnd = WINDOW_MANAGER.create_window(state);
        assert_eq!(is_iconic(hwnd), BOOL::TRUE);
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn is_iconic_returns_false_for_maximized_window() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let state = WindowState {
            style: window_style::WS_MAXIMIZE,
            ..test_window_state()
        };
        let hwnd = WINDOW_MANAGER.create_window(state);
        assert_eq!(is_iconic(hwnd), BOOL::FALSE);
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn is_iconic_returns_false_for_invalid_hwnd() {
        assert_eq!(is_iconic(HWND::NULL), BOOL::FALSE);
    }

    #[test]
    fn is_iconic_returns_true_after_show_window_minimized() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state());
        show_window(hwnd, show_window::SW_SHOWMINIMIZED);
        assert_eq!(is_iconic(hwnd), BOOL::TRUE);
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn is_iconic_returns_false_after_restore_from_minimized() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state());
        show_window(hwnd, show_window::SW_SHOWMINIMIZED);
        assert_eq!(is_iconic(hwnd), BOOL::TRUE);
        show_window(hwnd, show_window::SW_RESTORE);
        assert_eq!(is_iconic(hwnd), BOOL::FALSE);
        WINDOW_MANAGER.destroy_window(hwnd);
    }
}
