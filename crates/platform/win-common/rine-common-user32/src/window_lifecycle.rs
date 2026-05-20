//! Window lifecycle — shared logic for CreateWindowEx, DestroyWindow, ShowWindow, UpdateWindow.

use rine_types::windows::*;
use rine_types::{errors::BOOL, strings::json_escape};

use crate::backend::{
    create_native_window, destroy_native_window, request_native_redraw, set_native_visibility,
    set_native_window_rect,
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

/// Enable or disable mouse and keyboard input to the specified window.
///
/// Returns `BOOL::TRUE` if the window was previously enabled, `BOOL::FALSE` otherwise.
/// Posts a `WM_ENABLE` message to the window.
pub fn enable_window(hwnd: HWND, enable: BOOL) -> BOOL {
    let was_enabled = WINDOW_MANAGER
        .get_window(hwnd)
        .map(|state| state.enabled)
        .unwrap_or(false);

    WINDOW_MANAGER.update_window(hwnd, |state| {
        state.enabled = enable.is_true();
    });

    THREAD_MESSAGE_QUEUE.with(|queue| {
        queue.post_message(Msg {
            hwnd,
            message: window_message::WM_ENABLE,
            w_param: usize::from(enable.is_true()),
            l_param: 0,
            time: 0,
            pt: Point::default(),
        });
    });

    if was_enabled { BOOL::TRUE } else { BOOL::FALSE }
}

/// Retrieve the client-area coordinates of a window.
///
/// Fills `rect` with the client rectangle. The `left` and `top` members are
/// always 0; `right` and `bottom` contain the width and height of the client
/// area.
///
/// Returns `BOOL::TRUE` on success, `BOOL::FALSE` if the HWND is not found.
///
/// # Notes
/// This function returns the `client_rect` stored in the window manager. The
/// client rect is updated by `move_window` and `create_window`, but it does
/// **not** account for:
/// - Non-client areas (menus, scroll bars, title bar) — that would require a
///   `AdjustWindowRect[Ex]` helper which is not implemented yet.
/// - DPI scaling / per-monitor DPI awareness.
/// - Actual native compositor / window-manager decorations on the underlying
///   winit window, because the backend does not expose decoration sizes.
/// - `WM_NCCALCSIZE` handling — applications that customise their non-client
///   area will see stale values.
pub fn get_client_rect(hwnd: HWND, rect: &mut Rect) -> BOOL {
    match WINDOW_MANAGER.get_window(hwnd) {
        Some(state) => {
            *rect = state.client_rect;
            BOOL::TRUE
        }
        None => BOOL::FALSE,
    }
}

/// Convert screen coordinates to client-area coordinates.
///
/// Converts the screen coordinates of `point` to client coordinates relative
/// to the specified window. On success the point is modified in place.
///
/// # Arguments
/// * `hwnd`: Handle of the window whose client area defines the coordinate
///   space to convert into.
/// * `point`: A `Point` containing screen coordinates to convert. On success
///   the fields are overwritten with the corresponding client-area coordinates.
///
/// # Returns
/// `BOOL::TRUE` on success, `BOOL::FALSE` if the HWND is not found (the point
/// is not modified on failure).
///
/// # Notes
/// The current implementation uses the window's `rect` origin as the client-area
/// origin on screen. It does **not** account for:
/// - Non-client areas (title bar, borders, scroll bars) — the offset between
///   the window origin and the client-area origin is not tracked. A full
///   implementation would need `AdjustWindowRect[Ex]` or native decoration
///   queries.
/// - DPI scaling / per-monitor DPI awareness.
/// - Child-window coordinate chains: if `hwnd` is a child window, the
///   coordinates should logically be relative to its parent's client area, but
///   this conversion is not yet implemented.
/// - `GetLastError` is not set on failure.
pub fn screen_to_client(hwnd: HWND, point: &mut Point) -> BOOL {
    match WINDOW_MANAGER.get_window(hwnd) {
        Some(state) => {
            point.x -= state.rect.left;
            point.y -= state.rect.top;
            BOOL::TRUE
        }
        None => BOOL::FALSE,
    }
}

/// Move and optionally resize a window.
///
/// Changes the position and dimensions of the window. Posts `WM_MOVE` and `WM_SIZE`
/// messages. If `repaint` is `BOOL::TRUE`, also posts `WM_PAINT`.
///
/// Returns `BOOL::TRUE` on success, `BOOL::FALSE` if the HWND is not found.
pub fn move_window(hwnd: HWND, x: i32, y: i32, width: i32, height: i32, repaint: BOOL) -> BOOL {
    let found = WINDOW_MANAGER.update_window(hwnd, |state| {
        state.rect = Rect {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        };
        state.client_rect = Rect {
            left: 0,
            top: 0,
            right: width,
            bottom: height,
        };
    });

    if !found {
        return BOOL::FALSE;
    }

    set_native_window_rect(hwnd, x, y, width, height);

    THREAD_MESSAGE_QUEUE.with(|queue| {
        let lparam_move = ((y as i16 as u16 as usize) << 16) | (x as i16 as u16 as usize);
        queue.post_message(Msg {
            hwnd,
            message: window_message::WM_MOVE,
            w_param: 0,
            l_param: lparam_move as isize,
            time: 0,
            pt: Point { x, y },
        });

        let lparam_size = ((height as u16 as usize) << 16) | (width as u16 as usize);
        queue.post_message(Msg {
            hwnd,
            message: window_message::WM_SIZE,
            w_param: 0,
            l_param: lparam_size as isize,
            time: 0,
            pt: Point::default(),
        });

        if repaint == BOOL::TRUE {
            queue.post_message(Msg {
                hwnd,
                message: window_message::WM_PAINT,
                w_param: 0,
                l_param: 0,
                time: 0,
                pt: Point::default(),
            });
        }
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

    #[test]
    fn get_client_rect_returns_correct_size() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let state = WindowState {
            rect: Rect {
                left: 10,
                top: 20,
                right: 110,
                bottom: 70,
            },
            client_rect: Rect {
                left: 0,
                top: 0,
                right: 100,
                bottom: 50,
            },
            ..test_window_state()
        };
        let hwnd = WINDOW_MANAGER.create_window(state);
        let mut rect = Rect::default();
        assert_eq!(get_client_rect(hwnd, &mut rect), BOOL::TRUE);
        assert_eq!(rect.left, 0);
        assert_eq!(rect.top, 0);
        assert_eq!(rect.right, 100);
        assert_eq!(rect.bottom, 50);
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn get_client_rect_returns_false_for_invalid_hwnd() {
        let mut rect = Rect::default();
        assert_eq!(get_client_rect(HWND::NULL, &mut rect), BOOL::FALSE);
    }

    #[test]
    fn get_client_rect_reflects_move_window() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state());
        move_window(hwnd, 5, 10, 200, 150, BOOL::FALSE);
        let mut rect = Rect::default();
        assert_eq!(get_client_rect(hwnd, &mut rect), BOOL::TRUE);
        assert_eq!(rect.left, 0);
        assert_eq!(rect.top, 0);
        assert_eq!(rect.right, 200);
        assert_eq!(rect.bottom, 150);
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn get_client_rect_does_not_modify_rect_on_failure() {
        let mut rect = Rect {
            left: -1,
            top: -2,
            right: -3,
            bottom: -4,
        };
        let original = rect;
        assert_eq!(get_client_rect(HWND::NULL, &mut rect), BOOL::FALSE);
        assert_eq!(rect, original);
    }

    #[test]
    fn screen_to_client_converts_screen_point_to_client() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let state = WindowState {
            rect: Rect {
                left: 100,
                top: 200,
                right: 400,
                bottom: 500,
            },
            client_rect: Rect {
                left: 0,
                top: 0,
                right: 300,
                bottom: 300,
            },
            ..test_window_state()
        };
        let hwnd = WINDOW_MANAGER.create_window(state);
        let mut pt = Point { x: 150, y: 250 };
        assert_eq!(screen_to_client(hwnd, &mut pt), BOOL::TRUE);
        assert_eq!(pt.x, 50);
        assert_eq!(pt.y, 50);
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn screen_to_client_point_above_window_produces_negative_coords() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let state = WindowState {
            rect: Rect {
                left: 100,
                top: 200,
                right: 400,
                bottom: 500,
            },
            ..test_window_state()
        };
        let hwnd = WINDOW_MANAGER.create_window(state);
        let mut pt = Point { x: 50, y: 50 };
        assert_eq!(screen_to_client(hwnd, &mut pt), BOOL::TRUE);
        assert_eq!(pt.x, -50);
        assert_eq!(pt.y, -150);
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn screen_to_client_returns_false_for_invalid_hwnd() {
        let mut pt = Point { x: 100, y: 100 };
        assert_eq!(screen_to_client(HWND::NULL, &mut pt), BOOL::FALSE);
    }

    #[test]
    fn screen_to_client_does_not_modify_point_on_failure() {
        let mut pt = Point { x: 42, y: 99 };
        let original = pt;
        assert_eq!(screen_to_client(HWND::NULL, &mut pt), BOOL::FALSE);
        assert_eq!(pt.x, original.x);
        assert_eq!(pt.y, original.y);
    }

    #[test]
    fn screen_to_client_reflects_move_window() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state());
        move_window(hwnd, 50, 60, 200, 150, BOOL::FALSE);
        let mut pt = Point { x: 100, y: 110 };
        assert_eq!(screen_to_client(hwnd, &mut pt), BOOL::TRUE);
        assert_eq!(pt.x, 50);
        assert_eq!(pt.y, 50);
        WINDOW_MANAGER.destroy_window(hwnd);
    }
}
