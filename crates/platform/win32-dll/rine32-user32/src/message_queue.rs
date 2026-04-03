use rine_common_user32 as common;
use rine_types::windows::*;

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn get_message_a(
    msg: *mut Msg,
    hwnd: usize,
    msg_filter_min: u32,
    msg_filter_max: u32,
) -> i32 {
    unsafe { common::get_message(msg, hwnd, msg_filter_min, msg_filter_max) }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn get_message_w(
    msg: *mut Msg,
    hwnd: usize,
    msg_filter_min: u32,
    msg_filter_max: u32,
) -> i32 {
    get_message_a(msg, hwnd, msg_filter_min, msg_filter_max)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn peek_message_a(
    msg: *mut Msg,
    hwnd: usize,
    msg_filter_min: u32,
    msg_filter_max: u32,
    remove: u32,
) -> i32 {
    unsafe { common::peek_message(msg, hwnd, msg_filter_min, msg_filter_max, remove) }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn peek_message_w(
    msg: *mut Msg,
    hwnd: usize,
    msg_filter_min: u32,
    msg_filter_max: u32,
    remove: u32,
) -> i32 {
    peek_message_a(msg, hwnd, msg_filter_min, msg_filter_max, remove)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn translate_message(msg: *const Msg) -> i32 {
    unsafe { common::translate_message(msg) }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn dispatch_message_a(msg: *const Msg) -> isize {
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
    dispatch_message_a(msg)
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
