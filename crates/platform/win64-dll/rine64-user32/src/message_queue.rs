use rine_common_user32 as common;
use rine_types::windows::*;

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
/// * This function is a blocking call and will not return until a message is available in the thread's message queue.
/// * The `hwnd`, `msg_filter_min`, and `msg_filter_max` parameters are currently ignored in this implementation,
///   but they are included to match the signature of the Windows API function and may be used in future enhancements
///   to filter messages based on the specified window handle and message range.
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub(crate) unsafe extern "win64" fn GetMessageA(
    msg: *mut Msg,
    hwnd: usize,
    msg_filter_min: u32,
    msg_filter_max: u32,
) -> i32 {
    unsafe { common::get_message(msg, hwnd, msg_filter_min, msg_filter_max) }
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
/// * This function is a blocking call and will not return until a message is available in the thread's message queue.
/// * The `hwnd`, `msg_filter_min`, and `msg_filter_max` parameters are currently ignored in this implementation,
///   but they are included to match the signature of the Windows API function and may be used in future enhancements
///   to filter messages based on the specified window handle and message range.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn GetMessageW(
    msg: *mut Msg,
    hwnd: usize,
    msg_filter_min: u32,
    msg_filter_max: u32,
) -> i32 {
    unsafe { common::get_message(msg, hwnd, msg_filter_min, msg_filter_max) }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn peek_message_a(
    msg: *mut Msg,
    hwnd: usize,
    msg_filter_min: u32,
    msg_filter_max: u32,
    remove: u32,
) -> i32 {
    unsafe { common::peek_message(msg, hwnd, msg_filter_min, msg_filter_max, remove) }
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
pub(crate) unsafe extern "win64" fn translate_message(msg: *const Msg) -> i32 {
    unsafe { common::translate_message(msg) }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn dispatch_message_a(msg: *const Msg) -> isize {
    unsafe {
        common::dispatch_message(msg, |proc_fn, hwnd, m, wp, lp| {
            let f: extern "win64" fn(usize, u32, usize, isize) -> isize =
                std::mem::transmute(proc_fn);
            f(hwnd, m, wp, lp)
        })
    }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn dispatch_message_w(msg: *const Msg) -> isize {
    dispatch_message_a(msg)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn post_quit_message(exit_code: i32) {
    common::post_quit_message(exit_code);
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn post_message_a(
    hwnd: usize,
    msg: u32,
    w_param: usize,
    l_param: isize,
) -> i32 {
    common::post_message(hwnd, msg, w_param, l_param)
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
    unsafe {
        common::send_message(hwnd, msg, w_param, l_param, |proc_fn, h, m, wp, lp| {
            let f: extern "win64" fn(usize, u32, usize, isize) -> isize =
                std::mem::transmute(proc_fn);
            f(h, m, wp, lp)
        })
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
    hwnd: usize,
    msg: u32,
    w_param: usize,
    l_param: isize,
) -> isize {
    common::def_window_proc(hwnd, msg, w_param, l_param)
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
