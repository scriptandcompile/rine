use rine_types::windows::{HWND, WINDOW_MANAGER};

pub mod button_control;
pub mod edit_control;
pub mod static_control;

pub fn dispatch_default_control_proc(
    hwnd: HWND,
    msg: u32,
    w_param: usize,
    l_param: isize,
) -> Option<isize> {
    let state = WINDOW_MANAGER.get_window(hwnd)?;
    let class_name = state.class_name;

    if class_name.eq_ignore_ascii_case("STATIC") {
        return Some(static_control::default_proc(hwnd, msg, w_param, l_param));
    }

    if class_name.eq_ignore_ascii_case("BUTTON") {
        return Some(button_control::default_proc(hwnd, msg, w_param, l_param));
    }

    if class_name.eq_ignore_ascii_case("EDIT") {
        return Some(edit_control::default_proc(hwnd, msg, w_param, l_param));
    }

    None
}
