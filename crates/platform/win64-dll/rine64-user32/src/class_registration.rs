use rine_common_user32::{register_class, unregister_class};
use rine_types::strings::{read_cstr, read_wstr};
use rine_types::windows::*;

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub(crate) unsafe extern "win64" fn RegisterClassA(wc: *const WndClassExA) -> u16 {
    rine_common_user32::class_registration::register_class_a(wc)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn register_class_w(wc: *const WndClassExW) -> u16 {
    if wc.is_null() {
        return 0;
    }
    let wc = &*wc;
    let class_name = read_wstr(wc.lpsz_class_name).unwrap_or_default();
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
    register_class(class_name, class)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn register_class_ex_a(wc: *const WndClassExA) -> u16 {
    RegisterClassA(wc)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn register_class_ex_w(wc: *const WndClassExW) -> u16 {
    register_class_w(wc)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn unregister_class_a(
    class_name: *const u8,
    _h_instance: usize,
) -> i32 {
    let name = read_cstr(class_name).unwrap_or_default();
    unregister_class(&name)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn unregister_class_w(
    class_name: *const u16,
    _h_instance: usize,
) -> i32 {
    let name = read_wstr(class_name).unwrap_or_default();
    unregister_class(&name)
}
