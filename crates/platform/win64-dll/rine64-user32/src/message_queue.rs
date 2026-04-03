use rine_types::windows::*;
use std::sync::OnceLock;

use crate::backend::pump_backend_messages;

fn user32_debug_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var("RINE_USER32_DEBUG")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
            .unwrap_or(false)
    })
}

fn debug_log(msg: impl AsRef<str>) {
    if user32_debug_enabled() {
        eprintln!("[user32/msg] {}", msg.as_ref());
    }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn get_message_a(
    msg: *mut Msg,
    _hwnd: usize,
    _msg_filter_min: u32,
    _msg_filter_max: u32,
) -> i32 {
    if msg.is_null() {
        return -1;
    }

    loop {
        // Keep pumping host window events while waiting so close-box clicks
        // are converted into WM_CLOSE/WM_DESTROY messages.
        pump_backend_messages();

        let has_message = THREAD_MESSAGE_QUEUE.with(|queue| queue.peek_message(&mut *msg, true));
        if has_message {
            if (*msg).message == window_message::WM_QUIT {
                debug_log("GetMessage -> WM_QUIT");
                return 0;
            }

            debug_log(format!(
                "GetMessage -> hwnd={:#x} msg={:#06x}",
                (*msg).hwnd.as_raw(),
                (*msg).message
            ));
            return 1;
        }

        std::thread::yield_now();
    }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn get_message_w(
    msg: *mut Msg,
    hwnd: usize,
    msg_filter_min: u32,
    msg_filter_max: u32,
) -> i32 {
    get_message_a(msg, hwnd, msg_filter_min, msg_filter_max)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn peek_message_a(
    msg: *mut Msg,
    _hwnd: usize,
    _msg_filter_min: u32,
    _msg_filter_max: u32,
    remove: u32,
) -> i32 {
    if msg.is_null() {
        return 0;
    }

    pump_backend_messages();
    let remove = (remove & 0x0001) != 0;

    THREAD_MESSAGE_QUEUE.with(|queue| {
        if queue.peek_message(&mut *msg, remove) {
            debug_log(format!(
                "PeekMessage(remove={}) -> hwnd={:#x} msg={:#06x}",
                remove,
                (*msg).hwnd.as_raw(),
                (*msg).message
            ));
            1
        } else {
            0
        }
    })
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn peek_message_w(
    msg: *mut Msg,
    hwnd: usize,
    msg_filter_min: u32,
    msg_filter_max: u32,
    remove: u32,
) -> i32 {
    peek_message_a(msg, hwnd, msg_filter_min, msg_filter_max, remove)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn translate_message(_msg: *const Msg) -> i32 {
    1
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn dispatch_message_a(msg: *const Msg) -> isize {
    if msg.is_null() {
        return 0;
    }

    let msg = &*msg;
    debug_log(format!(
        "DispatchMessage hwnd={:#x} msg={:#06x}",
        msg.hwnd.as_raw(),
        msg.message
    ));

    if let Some(state) = WINDOW_MANAGER.get_window(msg.hwnd) {
        let wnd_proc: extern "win64" fn(usize, u32, usize, isize) -> isize =
            std::mem::transmute(state.wnd_proc);
        let result = wnd_proc(msg.hwnd.as_raw(), msg.message, msg.w_param, msg.l_param);
        debug_log(format!(
            "DispatchMessage result msg={:#06x} -> {}",
            msg.message, result
        ));
        result
    } else {
        debug_log("DispatchMessage skipped: hwnd not found");
        0
    }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn dispatch_message_w(msg: *const Msg) -> isize {
    dispatch_message_a(msg)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn post_quit_message(exit_code: i32) {
    THREAD_MESSAGE_QUEUE.with(|queue| {
        queue.post_quit(exit_code);
    });
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn post_message_a(
    hwnd: usize,
    msg: u32,
    w_param: usize,
    l_param: isize,
) -> i32 {
    let message = Msg {
        hwnd: Hwnd::from_raw(hwnd),
        message: msg,
        w_param,
        l_param,
        time: 0,
        pt: Point { x: 0, y: 0 },
    };

    THREAD_MESSAGE_QUEUE.with(|queue| {
        queue.post_message(message);
    });

    1
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn post_message_w(
    hwnd: usize,
    msg: u32,
    w_param: usize,
    l_param: isize,
) -> i32 {
    post_message_a(hwnd, msg, w_param, l_param)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn send_message_a(
    hwnd: usize,
    msg: u32,
    w_param: usize,
    l_param: isize,
) -> isize {
    let hwnd = Hwnd::from_raw(hwnd);

    if let Some(state) = WINDOW_MANAGER.get_window(hwnd) {
        let wnd_proc: extern "win64" fn(usize, u32, usize, isize) -> isize =
            std::mem::transmute(state.wnd_proc);
        wnd_proc(hwnd.as_raw(), msg, w_param, l_param)
    } else {
        0
    }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn send_message_w(
    hwnd: usize,
    msg: u32,
    w_param: usize,
    l_param: isize,
) -> isize {
    send_message_a(hwnd, msg, w_param, l_param)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn def_window_proc_a(
    _hwnd: usize,
    _msg: u32,
    _w_param: usize,
    _l_param: isize,
) -> isize {
    0
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn def_window_proc_w(
    hwnd: usize,
    msg: u32,
    w_param: usize,
    l_param: isize,
) -> isize {
    def_window_proc_a(hwnd, msg, w_param, l_param)
}
