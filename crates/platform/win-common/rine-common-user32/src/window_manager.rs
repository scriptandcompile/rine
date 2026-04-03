//! Window lifecycle — shared logic for CreateWindowEx, DestroyWindow, ShowWindow, UpdateWindow.

use rine_types::strings::json_escape;
use rine_types::windows::*;

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
    parent: usize,
) -> usize {
    let escaped_class_name = json_escape(&class_name);
    let escaped_title = json_escape(&title);

    let class = match WINDOW_CLASS_REGISTRY.get(&class_name) {
        Some(c) => c,
        None => return 0,
    };

    let state = WindowState {
        hwnd: Hwnd::NULL,
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
        parent: Hwnd::from_raw(parent),
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
        parent,
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

    hwnd.as_raw()
}

/// Destroy a window.
///
/// Calls the window procedure with WM_DESTROY (using the provided callback
/// to respect the caller's ABI), tears down the native window, and removes
/// the window from the manager.
///
/// `call_wnd_proc` receives `(proc_fn, hwnd, msg, w_param, l_param)`.
///
/// Returns 1 on success, 0 if the HWND was not found.
pub unsafe fn destroy_window(
    hwnd: usize,
    call_wnd_proc: impl Fn(usize, usize, u32, usize, isize) -> isize,
) -> i32 {
    let hwnd = Hwnd::from_raw(hwnd);

    let Some(state) = WINDOW_MANAGER.get_window(hwnd) else {
        return 0;
    };

    // Deliver WM_DESTROY synchronously while the window state is still valid.
    // Apps commonly call PostQuitMessage from this handler.
    let _ = call_wnd_proc(
        state.wnd_proc,
        hwnd.as_raw(),
        window_message::WM_DESTROY,
        0,
        0,
    );

    destroy_native_window(hwnd);

    if WINDOW_MANAGER.destroy_window(hwnd) {
        rine_types::dev_notify!(on_handle_closed(hwnd.as_raw() as i64));
        1
    } else {
        0
    }
}

/// Show or hide a window according to `cmd_show` (SW_* constants).
///
/// Returns 1 if the window was previously visible, 0 otherwise.
pub fn show_window(hwnd: usize, cmd_show: i32) -> i32 {
    let hwnd = Hwnd::from_raw(hwnd);

    let was_visible = WINDOW_MANAGER
        .get_window(hwnd)
        .map(|state| state.visible)
        .unwrap_or(false);

    WINDOW_MANAGER.update_window(hwnd, |state| match cmd_show {
        show_window::SW_HIDE => state.visible = false,
        show_window::SW_SHOWNORMAL | show_window::SW_SHOW | show_window::SW_SHOWDEFAULT => {
            state.visible = true
        }
        show_window::SW_SHOWMINIMIZED | show_window::SW_SHOWMAXIMIZED | show_window::SW_RESTORE => {
            state.visible = true;
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

    if was_visible { 1 } else { 0 }
}

/// Request a WM_PAINT for the given window.
///
/// Returns 1 always (UpdateWindow is a notification, not a query).
pub fn update_window(hwnd: usize) -> i32 {
    let hwnd = Hwnd::from_raw(hwnd);

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

    1
}
