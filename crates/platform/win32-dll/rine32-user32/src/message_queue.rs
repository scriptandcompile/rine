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
/// The caller must ensure that `msg` is a valid pointer to a `Msg` structure.
/// The caller must ensure that the thread has a message queue (for example, by calling `GetMessage` or
/// `PeekMessage` at least once before).
/// The caller must ensure that the message loop is properly implemented to handle messages and avoid
/// deadlocks or unresponsive behavior.
///
/// # Returns
/// `1` if a message other than `WM_QUIT` is retrieved and placed in the `Msg` structure pointed to by `msg`.
/// `0` if the message is `WM_QUIT` and is placed in the `Msg` structure pointed to by `msg`.
/// `-1` if there is an error (for example, if `msg` is an invalid pointer).
///
/// # Notes
/// This function is a blocking call and will not return until a message is available in the thread's message queue.
/// The `hwnd`, `msg_filter_min`, and `msg_filter_max` parameters are currently ignored in this implementation,
/// but they are included to match the signature of the Windows API function and may be used in future enhancements
/// to filter messages based on the specified window handle and message range.
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub(crate) unsafe extern "stdcall" fn GetMessageA(
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
/// The caller must ensure that `msg` is a valid pointer to a `Msg` structure.
/// The caller must ensure that the thread has a message queue (for example, by calling `GetMessage` or
/// `PeekMessage` at least once before).
/// The caller must ensure that the message loop is properly implemented to handle messages and avoid
/// deadlocks or unresponsive behavior.
///
/// # Returns
/// `1` if a message other than `WM_QUIT` is retrieved and placed in the `Msg` structure pointed to by `msg`.
/// `0` if the message is `WM_QUIT` and is placed in the `Msg` structure pointed to by `msg`.
/// `-1` if there is an error (for example, if `msg` is an invalid pointer).
///
/// # Notes
/// This function is a blocking call and will not return until a message is available in the thread's message queue.
/// The `hwnd`, `msg_filter_min`, and `msg_filter_max` parameters are currently ignored in this implementation,
/// but they are included to match the signature of the Windows API function and may be used in future enhancements
/// to filter messages based on the specified window handle and message range.
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub(crate) unsafe extern "stdcall" fn GetMessageW(
    msg: *mut Msg,
    hwnd: usize,
    msg_filter_min: u32,
    msg_filter_max: u32,
) -> i32 {
    unsafe { common::get_message(msg, hwnd, msg_filter_min, msg_filter_max) }
}

/// Check the thread's message queue for a message and places it (if there is one)
/// in the `Msg` structure pointed to by `msg`.
///
/// # Arguments
/// * `msg` - Pointer to a `Msg` structure that receives message information from the thread's message queue.
/// * `hwnd` - Handle to the window whose messages are to be retrieved.
///   If this parameter is `0`, `PeekMessage` retrieves messages for any window that belongs to the calling thread,
///   and any messages on the current thread's message queue whose hwnd value is `0`.
/// * `msg_filter_min` - Specifies the integer value of the lowest message value to be examined.
/// * `msg_filter_max` - Specifies the integer value of the highest message value to be examined.
/// * `remove` - Specifies how messages are to be handled. This parameter can be a combination of the following values:
///   * `0` (PM_NOREMOVE): Messages are not removed from the queue after processing by `PeekMessage`.
///   * `1` (PM_REMOVE): Messages are removed from the queue after processing by `PeekMessage`.
///   * `2` (PM_NOYIELD): Prevents the system from releasing any thread that is waiting for the caller to go idle
///     (see `WaitMessage`) if a message is found. This flag is not supported in this implementation and is included only
///     to match the signature of the Windows API function.
///
/// # Safety
/// The caller must ensure that `msg` is a valid pointer to a `Msg` structure.
/// The caller must ensure that the thread has a message queue (for example, by calling `GetMessage` or `PeekMessage`
/// at least once before).
/// The caller must ensure that the message loop is properly implemented to handle messages and avoid deadlocks or
/// unresponsive behavior.
///
/// # Returns
/// `1` if a message is available in the thread's message queue and is placed in the `Msg` structure pointed to by `msg`.
/// `0` if there is no message available in the thread's message queue.
/// `-1` if there is an error (for example, if `msg` is an invalid pointer).
///
/// # Notes
/// This function is a non-blocking call and will return immediately whether or not a message is available in the
/// thread's message queue.
/// The `hwnd`, `msg_filter_min`, and `msg_filter_max` parameters are currently ignored in this implementation,
/// but they are included to match the signature of the Windows API function and may be used in future enhancements
/// to filter messages based on the specified window handle and message range.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn PeekMessageA(
    msg: *mut Msg,
    hwnd: usize,
    msg_filter_min: u32,
    msg_filter_max: u32,
    remove: u32,
) -> i32 {
    unsafe { common::peek_message(msg, hwnd, msg_filter_min, msg_filter_max, remove) }
}

/// Check the thread's message queue for a message and places it (if there is one)
/// in the `Msg` structure pointed to by `msg`.
///
/// # Arguments
/// * `msg` - Pointer to a `Msg` structure that receives message information from the thread's message queue.
/// * `hwnd` - Handle to the window whose messages are to be retrieved.
///   If this parameter is `0`, `PeekMessage` retrieves messages for any window that belongs to the calling thread,
///   and any messages on the current thread's message queue whose hwnd value is `0`.
/// * `msg_filter_min` - Specifies the integer value of the lowest message value to be examined.
/// * `msg_filter_max` - Specifies the integer value of the highest message value to be examined.
/// * `remove` - Specifies how messages are to be handled. This parameter can be a combination of the following values:
///   * `0` (PM_NOREMOVE): Messages are not removed from the queue after processing by `PeekMessage`.
///   * `1` (PM_REMOVE): Messages are removed from the queue after processing by `PeekMessage`.
///   * `2` (PM_NOYIELD): Prevents the system from releasing any thread that is waiting for the caller to go idle
///     (see `WaitMessage`) if a message is found. This flag is not supported in this implementation and is included only
///     to match the signature of the Windows API function.
///
/// # Safety
/// The caller must ensure that `msg` is a valid pointer to a `Msg` structure.
/// The caller must ensure that the thread has a message queue (for example, by calling `GetMessage` or `PeekMessage`
/// at least once before).
/// The caller must ensure that the message loop is properly implemented to handle messages and avoid deadlocks or
/// unresponsive behavior.
///
/// # Returns
/// `1` if a message is available in the thread's message queue and is placed in the `Msg` structure pointed to by `msg`.
/// `0` if there is no message available in the thread's message queue.
/// `-1` if there is an error (for example, if `msg` is an invalid pointer).
///
/// # Notes
/// This function is a non-blocking call and will return immediately whether or not a message is available in the
/// thread's message queue.
/// The `hwnd`, `msg_filter_min`, and `msg_filter_max` parameters are currently ignored in this implementation,
/// but they are included to match the signature of the Windows API function and may be used in future enhancements
/// to filter messages based on the specified window handle and message range.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn PeekMessageW(
    msg: *mut Msg,
    hwnd: usize,
    msg_filter_min: u32,
    msg_filter_max: u32,
    remove: u32,
) -> i32 {
    unsafe { common::peek_message(msg, hwnd, msg_filter_min, msg_filter_max, remove) }
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
/// The `TranslateMessage` function does not modify the message pointed to by `msg`, but it may generate
/// additional messages (for example, character messages) and place them in the thread's message queue based
/// on the virtual-key messages it processes.
/// The caller should ensure that the message loop is designed to handle any additional messages generated by
/// `TranslateMessage` to avoid unresponsive behavior or deadlocks.
/// Currently, this implementation of `TranslateMessage` does not perform any actual translation and simply stubs to  returns `1`.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn TranslateMessage(msg: *const Msg) -> i32 {
    unsafe { common::translate_message(msg) }
}

/// Dispatches a message to a window procedure.
///
/// # Arguments
/// * `msg` - Pointer to a `Msg` structure that contains message information retrieved from the
///   thread's message queue by `GetMessage` or `PeekMessage`.
///
/// # Safety
/// The caller must ensure that `msg` is a valid pointer to a `Msg` structure that contains message
/// information retrieved from the thread's message queue by `GetMessage` or `PeekMessage`.
/// The caller must ensure that the message loop is properly implemented to handle messages and avoid
/// deadlocks or unresponsive behavior.
/// The caller must ensure that the window procedure associated with the message's target window is
/// designed to handle the messages dispatched by this function to avoid unresponsive behavior or deadlocks.
/// The caller should ensure that the message loop is designed to handle any additional messages
/// generated by the window procedure in response to the dispatched message to avoid unresponsive behavior or deadlocks.
/// The caller should ensure that the message loop is designed to handle any messages that may be
/// generated by the window procedure in response to the dispatched message (for example, by calling
/// `GetMessage` or `PeekMessage` in a loop) to avoid unresponsive behavior or deadlocks.
///
/// # Returns
/// The return value is the result of the message processing and depends on the message sent.
/// The return value is typically determined by the window procedure that processes the message and may
/// vary based on the specific message and the implementation of the window procedure.
/// The caller should refer to the documentation for the specific messages being dispatched and the
/// expected return values from the window procedure to understand the meaning of the return value from this function.
///
/// # Notes
/// This function is typically called in the message loop after retrieving a message with `GetMessage` or
/// `PeekMessage` and optionally translating the message with `TranslateMessage`, to dispatch the message
/// to the appropriate window procedure for processing.
/// The `DispatchMessage` function does not perform any message filtering or modification and simply forwards
/// the message to the window procedure associated with the message's target window for processing.
/// The caller should ensure that the message loop is designed to handle any messages generated by the window
/// procedure in response to the dispatched message to avoid unresponsive behavior or deadlocks.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn DispatchMessageA(msg: *const Msg) -> isize {
    unsafe {
        common::dispatch_message(msg, |proc_fn, hwnd, m, wp, lp| {
            let f: extern "stdcall" fn(usize, u32, usize, isize) -> isize =
                std::mem::transmute(proc_fn);
            f(hwnd, m, wp, lp)
        })
    }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn dispatch_message_w(msg: *const Msg) -> isize {
    unsafe {
        common::dispatch_message(msg, |proc_fn, hwnd, m, wp, lp| {
            let f: extern "stdcall" fn(usize, u32, usize, isize) -> isize =
                std::mem::transmute(proc_fn);
            f(hwnd, m, wp, lp)
        })
    }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn post_quit_message(exit_code: i32) {
    common::post_quit_message(exit_code);
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn post_message_a(
    hwnd: usize,
    msg: u32,
    w_param: usize,
    l_param: isize,
) -> i32 {
    common::post_message(hwnd, msg, w_param, l_param)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn post_message_w(
    hwnd: usize,
    msg: u32,
    w_param: usize,
    l_param: isize,
) -> i32 {
    post_message_a(hwnd, msg, w_param, l_param)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn send_message_a(
    hwnd: usize,
    msg: u32,
    w_param: usize,
    l_param: isize,
) -> isize {
    unsafe {
        common::send_message(hwnd, msg, w_param, l_param, |proc_fn, h, m, wp, lp| {
            let f: extern "stdcall" fn(usize, u32, usize, isize) -> isize =
                std::mem::transmute(proc_fn);
            f(h, m, wp, lp)
        })
    }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn send_message_w(
    hwnd: usize,
    msg: u32,
    w_param: usize,
    l_param: isize,
) -> isize {
    send_message_a(hwnd, msg, w_param, l_param)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn def_window_proc_a(
    hwnd: usize,
    msg: u32,
    w_param: usize,
    l_param: isize,
) -> isize {
    common::def_window_proc(hwnd, msg, w_param, l_param)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn def_window_proc_w(
    hwnd: usize,
    msg: u32,
    w_param: usize,
    l_param: isize,
) -> isize {
    def_window_proc_a(hwnd, msg, w_param, l_param)
}
