#![allow(unsafe_op_in_unsafe_fn)]

use rine_dlls::{DllPlugin, Export, as_win_api};
use rine_types::strings::{read_cstr, read_wstr};
use rine_types::windows::*;

pub struct User32Plugin;

impl DllPlugin for User32Plugin {
    fn dll_names(&self) -> &[&str] {
        &["user32.dll"]
    }

    fn exports(&self) -> Vec<Export> {
        vec![
            Export::Func("RegisterClassA", as_win_api!(register_class_a)),
            Export::Func("RegisterClassW", as_win_api!(register_class_w)),
            Export::Func("RegisterClassExA", as_win_api!(register_class_ex_a)),
            Export::Func("RegisterClassExW", as_win_api!(register_class_ex_w)),
            Export::Func("UnregisterClassA", as_win_api!(unregister_class_a)),
            Export::Func("UnregisterClassW", as_win_api!(unregister_class_w)),
            Export::Func("CreateWindowExA", as_win_api!(create_window_ex_a)),
            Export::Func("CreateWindowExW", as_win_api!(create_window_ex_w)),
            Export::Func("DestroyWindow", as_win_api!(destroy_window)),
            Export::Func("ShowWindow", as_win_api!(show_window)),
            Export::Func("UpdateWindow", as_win_api!(update_window)),
            Export::Func("GetMessageA", as_win_api!(get_message_a)),
            Export::Func("GetMessageW", as_win_api!(get_message_w)),
            Export::Func("PeekMessageA", as_win_api!(peek_message_a)),
            Export::Func("PeekMessageW", as_win_api!(peek_message_w)),
            Export::Func("TranslateMessage", as_win_api!(translate_message)),
            Export::Func("DispatchMessageA", as_win_api!(dispatch_message_a)),
            Export::Func("DispatchMessageW", as_win_api!(dispatch_message_w)),
            Export::Func("PostQuitMessage", as_win_api!(post_quit_message)),
            Export::Func("PostMessageA", as_win_api!(post_message_a)),
            Export::Func("PostMessageW", as_win_api!(post_message_w)),
            Export::Func("SendMessageA", as_win_api!(send_message_a)),
            Export::Func("SendMessageW", as_win_api!(send_message_w)),
            Export::Func("DefWindowProcA", as_win_api!(def_window_proc_a)),
            Export::Func("DefWindowProcW", as_win_api!(def_window_proc_w)),
            Export::Func("SetWindowTextA", as_win_api!(set_window_text_a)),
            Export::Func("SetWindowTextW", as_win_api!(set_window_text_w)),
            Export::Func("GetWindowTextA", as_win_api!(get_window_text_a)),
            Export::Func("GetWindowTextW", as_win_api!(get_window_text_w)),
            Export::Func(
                "GetWindowTextLengthA",
                as_win_api!(get_window_text_length_a),
            ),
            Export::Func(
                "GetWindowTextLengthW",
                as_win_api!(get_window_text_length_w),
            ),
        ]
    }
}

// ---------------------------------------------------------------------------
// Window Class Registration
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
unsafe extern "C" fn register_class_a(wc: *const WndClassExA) -> u16 {
    if wc.is_null() {
        return 0;
    }
    let wc = &*wc;

    let class_name = read_cstr(wc.lpsz_class_name).unwrap_or_default();
    if class_name.is_empty() {
        return 0;
    }

    let menu_name = read_cstr(wc.lpsz_menu_name);

    let class = WindowClass {
        name: class_name.clone(),
        style: wc.style,
        wnd_proc: wc.lpfn_wnd_proc,
        cls_extra: wc.cb_cls_extra,
        wnd_extra: wc.cb_wnd_extra,
        instance: wc.h_instance,
        icon: wc.h_icon,
        cursor: wc.h_cursor,
        background: wc.hbr_background,
        menu_name,
        icon_sm: wc.h_icon_sm,
    };

    WINDOW_CLASS_REGISTRY.register(class_name, class);
    1 // Success (non-zero atom)
}

#[unsafe(no_mangle)]
unsafe extern "C" fn register_class_w(wc: *const WndClassExW) -> u16 {
    if wc.is_null() {
        return 0;
    }
    let wc = &*wc;

    let class_name = read_wstr(wc.lpsz_class_name).unwrap_or_default();
    if class_name.is_empty() {
        return 0;
    }

    let menu_name = read_wstr(wc.lpsz_menu_name);

    let class = WindowClass {
        name: class_name.clone(),
        style: wc.style,
        wnd_proc: wc.lpfn_wnd_proc,
        cls_extra: wc.cb_cls_extra,
        wnd_extra: wc.cb_wnd_extra,
        instance: wc.h_instance,
        icon: wc.h_icon,
        cursor: wc.h_cursor,
        background: wc.hbr_background,
        menu_name,
        icon_sm: wc.h_icon_sm,
    };

    WINDOW_CLASS_REGISTRY.register(class_name, class);
    1 // Success (non-zero atom)
}

#[unsafe(no_mangle)]
unsafe extern "C" fn register_class_ex_a(wc: *const WndClassExA) -> u16 {
    register_class_a(wc)
}

#[unsafe(no_mangle)]
unsafe extern "C" fn register_class_ex_w(wc: *const WndClassExW) -> u16 {
    register_class_w(wc)
}

#[unsafe(no_mangle)]
unsafe extern "C" fn unregister_class_a(class_name: *const u8, _h_instance: usize) -> i32 {
    let name = read_cstr(class_name).unwrap_or_default();
    if WINDOW_CLASS_REGISTRY.unregister(&name) {
        1
    } else {
        0
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn unregister_class_w(class_name: *const u16, _h_instance: usize) -> i32 {
    let name = read_wstr(class_name).unwrap_or_default();
    if WINDOW_CLASS_REGISTRY.unregister(&name) {
        1
    } else {
        0
    }
}

// ---------------------------------------------------------------------------
// Window Creation and Destruction
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
unsafe extern "C" fn create_window_ex_a(
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

    // Look up the window class
    let class = match WINDOW_CLASS_REGISTRY.get(&class_name_str) {
        Some(c) => c,
        None => return 0,
    };

    let state = WindowState {
        hwnd: Hwnd::NULL, // Will be set by window manager
        class_name: class_name_str,
        title: window_title,
        style,
        ex_style,
        rect: Rect {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        },
        client_rect: Rect {
            left: 0,
            top: 0,
            right: width,
            bottom: height,
        },
        parent: Hwnd::from_raw(parent),
        visible: (style & window_style::WS_VISIBLE) != 0,
        enabled: (style & window_style::WS_DISABLED) == 0,
        wnd_proc: class.wnd_proc,
        user_data: 0,
    };

    let hwnd = WINDOW_MANAGER.create_window(state);

    // TODO: Call WndProc with WM_CREATE
    // TODO: Actually create a window with X11/Wayland backend

    hwnd.as_raw()
}

#[unsafe(no_mangle)]
unsafe extern "C" fn create_window_ex_w(
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

    // For simplicity, we'll duplicate the logic here
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
        rect: Rect {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        },
        client_rect: Rect {
            left: 0,
            top: 0,
            right: width,
            bottom: height,
        },
        parent: Hwnd::from_raw(parent),
        visible: (style & window_style::WS_VISIBLE) != 0,
        enabled: (style & window_style::WS_DISABLED) == 0,
        wnd_proc: class.wnd_proc,
        user_data: 0,
    };

    let hwnd = WINDOW_MANAGER.create_window(state);
    hwnd.as_raw()
}

#[unsafe(no_mangle)]
unsafe extern "C" fn destroy_window(hwnd: usize) -> i32 {
    let hwnd = Hwnd::from_raw(hwnd);

    // TODO: Call WndProc with WM_DESTROY
    // TODO: Actually destroy the X11/Wayland window

    if WINDOW_MANAGER.destroy_window(hwnd) {
        1
    } else {
        0
    }
}

// ---------------------------------------------------------------------------
// Window Display
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
unsafe extern "C" fn show_window(hwnd: usize, cmd_show: i32) -> i32 {
    let hwnd = Hwnd::from_raw(hwnd);

    // Get the old visibility state and update based on command
    let was_visible = WINDOW_MANAGER
        .get_window(hwnd)
        .map(|state| state.visible)
        .unwrap_or(false);

    WINDOW_MANAGER.update_window(hwnd, |state| {
        match cmd_show {
            show_window::SW_HIDE => state.visible = false,
            show_window::SW_SHOWNORMAL | show_window::SW_SHOW | show_window::SW_SHOWDEFAULT => {
                state.visible = true
            }
            show_window::SW_SHOWMINIMIZED => {
                state.visible = true;
                // TODO: Minimize
            }
            show_window::SW_SHOWMAXIMIZED => {
                state.visible = true;
                // TODO: Maximize
            }
            show_window::SW_RESTORE => {
                state.visible = true;
                // TODO: Restore
            }
            _ => {}
        }
    });

    // TODO: Actually show/hide the X11/Wayland window

    if was_visible { 1 } else { 0 }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn update_window(_hwnd: usize) -> i32 {
    // TODO: Force a WM_PAINT message and process it immediately
    // For now, just succeed
    1
}

// ---------------------------------------------------------------------------
// Message Queue Functions
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
unsafe extern "C" fn get_message_a(
    msg: *mut Msg,
    _hwnd: usize,
    _msg_filter_min: u32,
    _msg_filter_max: u32,
) -> i32 {
    if msg.is_null() {
        return -1;
    }

    THREAD_MESSAGE_QUEUE.with(|queue| {
        if queue.get_message(&mut *msg) {
            1 // Got a message
        } else {
            0 // WM_QUIT received
        }
    })
}

#[unsafe(no_mangle)]
unsafe extern "C" fn get_message_w(
    msg: *mut Msg,
    hwnd: usize,
    msg_filter_min: u32,
    msg_filter_max: u32,
) -> i32 {
    get_message_a(msg, hwnd, msg_filter_min, msg_filter_max)
}

#[unsafe(no_mangle)]
unsafe extern "C" fn peek_message_a(
    msg: *mut Msg,
    _hwnd: usize,
    _msg_filter_min: u32,
    _msg_filter_max: u32,
    remove: u32,
) -> i32 {
    if msg.is_null() {
        return 0;
    }

    let remove = (remove & 0x0001) != 0; // PM_REMOVE

    THREAD_MESSAGE_QUEUE.with(|queue| {
        if queue.peek_message(&mut *msg, remove) {
            1
        } else {
            0
        }
    })
}

#[unsafe(no_mangle)]
unsafe extern "C" fn peek_message_w(
    msg: *mut Msg,
    hwnd: usize,
    msg_filter_min: u32,
    msg_filter_max: u32,
    remove: u32,
) -> i32 {
    peek_message_a(msg, hwnd, msg_filter_min, msg_filter_max, remove)
}

#[unsafe(no_mangle)]
unsafe extern "C" fn translate_message(_msg: *const Msg) -> i32 {
    // In a full implementation, this would translate virtual-key messages
    // (WM_KEYDOWN, WM_KEYUP) into character messages (WM_CHAR).
    // For now, just return success.
    1
}

#[unsafe(no_mangle)]
unsafe extern "C" fn dispatch_message_a(msg: *const Msg) -> isize {
    if msg.is_null() {
        return 0;
    }

    let msg = &*msg;

    // Look up the window and call its WndProc
    if let Some(state) = WINDOW_MANAGER.get_window(msg.hwnd) {
        let wnd_proc: extern "C" fn(usize, u32, usize, isize) -> isize =
            std::mem::transmute(state.wnd_proc);
        wnd_proc(msg.hwnd.as_raw(), msg.message, msg.w_param, msg.l_param)
    } else {
        0
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn dispatch_message_w(msg: *const Msg) -> isize {
    dispatch_message_a(msg)
}

#[unsafe(no_mangle)]
unsafe extern "C" fn post_quit_message(exit_code: i32) {
    THREAD_MESSAGE_QUEUE.with(|queue| {
        queue.post_quit(exit_code);
    });
}

#[unsafe(no_mangle)]
unsafe extern "C" fn post_message_a(hwnd: usize, msg: u32, w_param: usize, l_param: isize) -> i32 {
    let message = Msg {
        hwnd: Hwnd::from_raw(hwnd),
        message: msg,
        w_param,
        l_param,
        time: 0, // TODO: GetTickCount
        pt_x: 0,
        pt_y: 0,
    };

    THREAD_MESSAGE_QUEUE.with(|queue| {
        queue.post_message(message);
    });

    1
}

#[unsafe(no_mangle)]
unsafe extern "C" fn post_message_w(hwnd: usize, msg: u32, w_param: usize, l_param: isize) -> i32 {
    post_message_a(hwnd, msg, w_param, l_param)
}

#[unsafe(no_mangle)]
unsafe extern "C" fn send_message_a(
    hwnd: usize,
    msg: u32,
    w_param: usize,
    l_param: isize,
) -> isize {
    let hwnd = Hwnd::from_raw(hwnd);

    // Look up the window and call its WndProc directly (synchronous)
    if let Some(state) = WINDOW_MANAGER.get_window(hwnd) {
        let wnd_proc: extern "C" fn(usize, u32, usize, isize) -> isize =
            std::mem::transmute(state.wnd_proc);
        wnd_proc(hwnd.as_raw(), msg, w_param, l_param)
    } else {
        0
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn send_message_w(
    hwnd: usize,
    msg: u32,
    w_param: usize,
    l_param: isize,
) -> isize {
    send_message_a(hwnd, msg, w_param, l_param)
}

#[unsafe(no_mangle)]
unsafe extern "C" fn def_window_proc_a(
    _hwnd: usize,
    _msg: u32,
    _w_param: usize,
    _l_param: isize,
) -> isize {
    // Default window procedure - minimal implementation
    // In a real implementation, this would handle default processing
    // for various messages
    0
}

#[unsafe(no_mangle)]
unsafe extern "C" fn def_window_proc_w(
    hwnd: usize,
    msg: u32,
    w_param: usize,
    l_param: isize,
) -> isize {
    def_window_proc_a(hwnd, msg, w_param, l_param)
}

// ---------------------------------------------------------------------------
// Window Text Functions
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
unsafe extern "C" fn set_window_text_a(hwnd: usize, text: *const u8) -> i32 {
    let hwnd = Hwnd::from_raw(hwnd);
    let text_str = read_cstr(text).unwrap_or_default();

    WINDOW_MANAGER.update_window(hwnd, |state| {
        state.title = text_str;
    });

    // TODO: Update actual window title in X11/Wayland

    1
}

#[unsafe(no_mangle)]
unsafe extern "C" fn set_window_text_w(hwnd: usize, text: *const u16) -> i32 {
    let hwnd = Hwnd::from_raw(hwnd);
    let text_str = read_wstr(text).unwrap_or_default();

    WINDOW_MANAGER.update_window(hwnd, |state| {
        state.title = text_str;
    });

    1
}

#[unsafe(no_mangle)]
unsafe extern "C" fn get_window_text_a(hwnd: usize, buffer: *mut u8, max_count: i32) -> i32 {
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
    *buffer.add(copy_len) = 0; // Null terminator

    copy_len as i32
}

#[unsafe(no_mangle)]
unsafe extern "C" fn get_window_text_w(hwnd: usize, buffer: *mut u16, max_count: i32) -> i32 {
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
    *buffer.add(copy_len) = 0; // Null terminator

    copy_len as i32
}

#[unsafe(no_mangle)]
unsafe extern "C" fn get_window_text_length_a(hwnd: usize) -> i32 {
    let hwnd = Hwnd::from_raw(hwnd);

    match WINDOW_MANAGER.get_window(hwnd) {
        Some(state) => state.title.len() as i32,
        None => 0,
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn get_window_text_length_w(hwnd: usize) -> i32 {
    let hwnd = Hwnd::from_raw(hwnd);

    match WINDOW_MANAGER.get_window(hwnd) {
        Some(state) => state.title.encode_utf16().count() as i32,
        None => 0,
    }
}
