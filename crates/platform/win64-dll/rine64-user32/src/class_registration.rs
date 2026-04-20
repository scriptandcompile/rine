use rine_common_user32::class_registration as common;
use rine_types::strings::{read_cstr, read_wstr};
use rine_types::windows::*;

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub(crate) unsafe extern "win64" fn RegisterClassA(wc: *const WndClassExA) -> u16 {
    common::register_class_a(wc)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn RegisterClassW(wc: *const WndClassExW) -> u16 {
    common::register_class_w(wc)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn register_class_ex_a(wc: *const WndClassExA) -> u16 {
    RegisterClassA(wc)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn register_class_ex_w(wc: *const WndClassExW) -> u16 {
    RegisterClassW(wc)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn unregister_class_a(
    class_name: *const u8,
    _h_instance: usize,
) -> i32 {
    let name = read_cstr(class_name).unwrap_or_default();
    common::unregister_class(&name)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn unregister_class_w(
    class_name: *const u16,
    _h_instance: usize,
) -> i32 {
    let name = read_wstr(class_name).unwrap_or_default();
    common::unregister_class(&name)
}
