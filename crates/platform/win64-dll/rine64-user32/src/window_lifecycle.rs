use rine_types::strings::{json_escape, read_cstr, read_wstr};
use rine_types::windows::*;

use crate::backend::{
    create_native_window, destroy_native_window, request_native_redraw, set_native_visibility,
};

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn create_window_ex_a(
    ex_style: u32,
    class_name: *const u8,
    window_name: *const u8,
    style: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    parent: usize,
    _menu: usize,
    _instance: usize,
    _param: *mut u8,
) -> usize {
    let class_name_str = read_cstr(class_name).unwrap_or_default();
    let window_title = read_cstr(window_name).unwrap_or_default();

    create_window_common(
        ex_style,
        class_name_str,
        window_title,
        style,
        Rect {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        },
        parent,
    )
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn create_window_ex_w(
    ex_style: u32,
    class_name: *const u16,
    window_name: *const u16,
    style: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    parent: usize,
    _menu: usize,
    _instance: usize,
    _param: *mut u8,
) -> usize {
    let class_name_str = read_wstr(class_name).unwrap_or_default();
    let window_title = read_wstr(window_name).unwrap_or_default();

    create_window_common(
        ex_style,
        class_name_str,
        window_title,
        style,
        Rect {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        },
        parent,
    )
}

fn create_window_common(
    ex_style: u32,
    class_name_str: String,
    window_title: String,
    style: u32,
    rect: Rect,
    parent: usize,
) -> usize {
    let escaped_class_name = json_escape(&class_name_str);
    let escaped_window_title = json_escape(&window_title);

    let class = match WINDOW_CLASS_REGISTRY.get(&class_name_str) {
        Some(c) => c,
        None => return 0,
    };

    let state = WindowState {
        hwnd: Hwnd::NULL,
        class_name: class_name_str,
        title: window_title,
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
        escaped_window_title,
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

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn destroy_window(hwnd: usize) -> i32 {
    let hwnd = Hwnd::from_raw(hwnd);

    let Some(state) = WINDOW_MANAGER.get_window(hwnd) else {
        return 0;
    };

    // Deliver WM_DESTROY synchronously while the window state is still valid.
    // Apps commonly call PostQuitMessage from this handler.
    let wnd_proc: extern "win64" fn(usize, u32, usize, isize) -> isize =
        std::mem::transmute(state.wnd_proc);
    let _ = wnd_proc(hwnd.as_raw(), window_message::WM_DESTROY, 0, 0);

    destroy_native_window(hwnd);

    if WINDOW_MANAGER.destroy_window(hwnd) {
        rine_types::dev_notify!(on_handle_closed(hwnd.as_raw() as i64));
        1
    } else {
        0
    }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn show_window(hwnd: usize, cmd_show: i32) -> i32 {
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

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn update_window(hwnd: usize) -> i32 {
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
