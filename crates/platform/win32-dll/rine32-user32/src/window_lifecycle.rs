use rine_common_user32 as common;
use rine_types::strings::{read_cstr, read_wstr};
use rine_types::windows::*;

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn create_window_ex_a(
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
    common::create_window(
        ex_style,
        read_cstr(class_name).unwrap_or_default(),
        read_cstr(window_name).unwrap_or_default(),
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
pub(crate) unsafe extern "stdcall" fn create_window_ex_w(
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
    common::create_window(
        ex_style,
        read_wstr(class_name).unwrap_or_default(),
        read_wstr(window_name).unwrap_or_default(),
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
pub(crate) unsafe extern "stdcall" fn destroy_window(hwnd: usize) -> i32 {
    unsafe {
        common::destroy_window(hwnd, |proc_fn, h, msg, wp, lp| {
            let f: extern "stdcall" fn(usize, u32, usize, isize) -> isize =
                std::mem::transmute(proc_fn);
            f(h, msg, wp, lp)
        })
    }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn show_window(hwnd: usize, cmd_show: i32) -> i32 {
    common::show_window(hwnd, cmd_show)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn update_window(hwnd: usize) -> i32 {
    common::update_window(hwnd)
}
