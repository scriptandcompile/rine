//! Message queue — shared logic for Get/PeekMessage, Dispatch/Post/SendMessage, etc.

use std::sync::OnceLock;

use rine_types::windows::*;

use crate::backend::pump_backend_messages;

pub fn user32_debug_enabled() -> bool {
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

/// GetMessage — block until a non-WM_QUIT message is available.
///
/// Returns 1 when a message was retrieved, 0 for WM_QUIT, -1 on error.
///
/// # Safety
/// `msg` must be a valid pointer to a `Msg`.
pub unsafe fn get_message(
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

/// PeekMessage — non-blocking check for pending messages.
///
/// `remove` controls whether the message is consumed (PM_REMOVE = 0x0001).
///
/// # Safety
/// `msg` must be a valid pointer to a `Msg`.
pub unsafe fn peek_message(
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

/// TranslateMessage — translate virtual-key messages. Always returns 1 (stub).
///
/// # Safety
/// `msg` must be valid or null.
pub unsafe fn translate_message(_msg: *const Msg) -> i32 {
    1
}

/// DispatchMessage — call the window procedure for the message's target HWND.
///
/// `call_wnd_proc` receives `(proc_fn, hwnd, msg, w_param, l_param)` and
/// must invoke the function pointer using the correct calling convention.
///
/// # Safety
/// `msg` must be a valid pointer to a `Msg`.
pub unsafe fn dispatch_message(
    msg: *const Msg,
    call_wnd_proc: impl Fn(usize, usize, u32, usize, isize) -> isize,
) -> isize {
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
        let result = call_wnd_proc(
            state.wnd_proc,
            msg.hwnd.as_raw(),
            msg.message,
            msg.w_param,
            msg.l_param,
        );
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

/// PostQuitMessage — post WM_QUIT to the thread message queue.
pub fn post_quit_message(exit_code: i32) {
    THREAD_MESSAGE_QUEUE.with(|queue| {
        queue.post_quit(exit_code);
    });
}

/// PostMessage — post a message to the thread message queue without waiting.
///
/// Returns 1 always (errors are not surfaced in Phase 1).
pub fn post_message(hwnd: usize, msg: u32, w_param: usize, l_param: isize) -> i32 {
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

/// SendMessage — call the window procedure synchronously and return its result.
///
/// `call_wnd_proc` receives `(proc_fn, hwnd, msg, w_param, l_param)` and
/// must invoke the function pointer using the correct calling convention.
///
/// # Safety
/// None — all parameters are by value.
pub unsafe fn send_message(
    hwnd: usize,
    msg: u32,
    w_param: usize,
    l_param: isize,
    call_wnd_proc: impl Fn(usize, usize, u32, usize, isize) -> isize,
) -> isize {
    let hwnd_typed = Hwnd::from_raw(hwnd);

    if let Some(state) = WINDOW_MANAGER.get_window(hwnd_typed) {
        call_wnd_proc(state.wnd_proc, hwnd, msg, w_param, l_param)
    } else {
        0
    }
}

/// DefWindowProc — default window procedure; returns 0 for all messages.
pub fn def_window_proc(_hwnd: usize, _msg: u32, _w_param: usize, _l_param: isize) -> isize {
    0
}
