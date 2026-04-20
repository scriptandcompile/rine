use rine_common_user32 as common;
use rine_types::strings::{read_cstr, read_wstr};
use rine_types::windows::*;

/// Window class registration
///
/// # Arguments
/// * `wc` - Pointer to a WNDCLASSEX structure containing the window class information.
///
/// # Safety
/// * The caller must ensure that `wc` is a valid pointer to a properly initialized WNDCLASSEX structure.
/// * The caller must ensure that the window class name is unique and not already registered,
///   or that it matches an existing class if intended to be reused.
///
/// # Returns
/// * `ATOM` - Atom of the registered class on success, 0 on failure.
///  
/// # Notes
/// This function is a simplified implementation and does not perform all the checks and operations
/// that the real RegisterClassExA/W functions do. It also always returns 1 on success for simplicity,
/// as we do not manage actual atoms.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn RegisterClassA(wc: *const WndClassExA) -> ATOM {
    common::class_registration::register_class_a(wc)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn register_class_w(wc: *const WndClassExW) -> u16 {
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
    common::register_class(class_name, class)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn register_class_ex_a(wc: *const WndClassExA) -> u16 {
    RegisterClassA(wc)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn register_class_ex_w(wc: *const WndClassExW) -> u16 {
    register_class_w(wc)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn unregister_class_a(
    class_name: *const u8,
    _h_instance: usize,
) -> i32 {
    let name = read_cstr(class_name).unwrap_or_default();
    common::unregister_class(&name)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn unregister_class_w(
    class_name: *const u16,
    _h_instance: usize,
) -> i32 {
    let name = read_wstr(class_name).unwrap_or_default();
    common::unregister_class(&name)
}
