//! Message queue — shared logic for Get/PeekMessage, Dispatch/Post/SendMessage, etc.

use std::sync::OnceLock;

use rine_types::errors::BOOL;
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

/// Block until a non-WM_QUIT message is available then get it from the calling thread's message queue.
///
/// # Arguments
/// * `msg` - Pointer to a `Msg` structure that receives message information from the thread's message queue.
/// * `hwnd` - Handle to the window whose messages are to be retrieved.
///   If this parameter is `0`, `GetMessage` retrieves messages for any window that belongs to the calling thread,
///   and any messages on the current thread's message queue whose hwnd value is `0`.
///
/// # Safety
/// * The caller must ensure that `msg` is a valid pointer to a `Msg` structure.
/// * The caller must ensure that the thread has a message queue (for example, by calling `GetMessage` or
///   `PeekMessage` at least once before).
/// * The caller must ensure that the message loop is properly implemented to handle messages and avoid
///   deadlocks or unresponsive behavior.
///
/// # Returns
/// * `1` if a message other than `WM_QUIT` is retrieved and placed in the `Msg` structure pointed to by `msg`.
/// * `0` if the message is `WM_QUIT` and is placed in the `Msg` structure pointed to by `msg`.
/// * `-1` if there is an error (for example, if `msg` is an invalid pointer).
///
/// # Notes
/// Missing implementation features:
/// - This is a blocking call, but it does not fully mirror Win32 wake/scheduling behavior.
/// - `hwnd`, `msg_filter_min`, and `msg_filter_max` filtering is not implemented.
/// - No Win32-accurate error reporting (`GetLastError`) is provided for invalid
///   pointer and queue-state failures.
pub unsafe fn get_message(
    msg: *mut Msg,
    _hwnd: HWND,
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
    _hwnd: HWND,
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

/// Translates virtual-key messages into character messages.
///
/// # Arguments
/// * `msg` - Pointer to a `Msg` structure that contains message information retrieved from the thread's
///   message queue by `GetMessage` or `PeekMessage`.
///
/// # Safety
/// The caller must ensure that `msg` is a valid pointer to a `Msg` structure that contains message
/// information retrieved from the thread's message queue by `GetMessage` or `PeekMessage`.
/// The caller must ensure that the message loop is properly implemented to handle messages and avoid
/// deadlocks or unresponsive behavior.
///
/// # Returns
/// `1` if the message is translated and placed in the thread's message queue.
/// `0` if the message is not translated (for example, if it is not a virtual- key message or if the translation fails).
///
/// # Notes
/// This function is typically called in the message loop after retrieving a message with `GetMessage` or
/// `PeekMessage` and before dispatching the message with `DispatchMessage`.
/// Missing implementation features:
/// - No virtual-key to character translation is performed.
/// - No dead-key/keyboard-layout aware translation is implemented.
/// - No `WM_CHAR`/`WM_SYSCHAR` synthesis is performed.
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
    call_wnd_proc: impl Fn(usize, HWND, u32, usize, isize) -> isize,
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
            msg.hwnd,
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
/// Returns 1 always.
pub fn post_message(hwnd: HWND, msg: u32, w_param: usize, l_param: isize) -> i32 {
    let message = Msg {
        hwnd,
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
    hwnd: HWND,
    msg: u32,
    w_param: usize,
    l_param: isize,
    call_wnd_proc: impl Fn(usize, HWND, u32, usize, isize) -> isize,
) -> isize {
    if let Some(state) = WINDOW_MANAGER.get_window(hwnd) {
        call_wnd_proc(state.wnd_proc, hwnd, msg, w_param, l_param)
    } else {
        0
    }
}

/// DefWindowProc — default window procedure; returns 0 for all messages.
///
/// # Notes
/// Missing implementation features:
/// - No default message handling is implemented (non-client, keyboard, mouse, sizing, system commands).
/// - No message-specific return semantics are implemented; this stub always returns 0.
pub fn def_window_proc(_hwnd: HWND, _msg: u32, _w_param: usize, _l_param: isize) -> isize {
    0
}

/// Determines whether the specified message is intended for the dialog box and, if it is, processes the message.
///
/// # Arguments
/// * `hdlg` - Handle to the dialog box that is the target of the message.
/// * `msg` - Pointer to a `Msg` structure that contains message information retrieved from the thread's message
///   queue by `GetMessage` or `PeekMessage`.
///
/// # Safety
/// The caller must ensure that `hdlg` is a valid handle to a dialog box and that `msg` is a valid pointer to a
/// `Msg` structure that contains message information retrieved from the thread's message queue by `GetMessage` or `PeekMessage`.
///
/// # Returns
/// `BOOL::TRUE` if the message is intended for the dialog box and has been processed by this function.
/// `BOOL::FALSE` if the message is not intended for the dialog box and has not been processed by this function.
///
/// # Notes
/// Currently this function does not perform any actual dialog message processing and simply returns `BOOL::FALSE` for all messages.
pub unsafe fn is_dialog_message(hdlg: HWND, msg: *const Msg) -> BOOL {
    debug_log(format!(
        "IsDialogMessage hdlg={:#x} msg={:#06x}",
        hdlg.as_raw(),
        if msg.is_null() { 0 } else { (*msg).message }
    ));
    BOOL::FALSE
}
