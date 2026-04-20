use rine_common_user32::class_registration as common;
use rine_types::strings::{read_cstr, read_wstr};
use rine_types::windows::*;

/// Window class registration
///
/// # Arguments
/// * `wc` - Pointer to a `WndClassExA` structure containing the window class information.
///
/// # Safety
/// * The caller must ensure that `wc` is a valid pointer to a properly initialized `WndClassExA` structure.
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
    common::register_class_a(wc)
}

/// Window class registration
///
/// # Arguments
/// * `wc` - Pointer to a `WndClassExW` structure containing the window class information.
///
/// # Safety
/// * The caller must ensure that `wc` is a valid pointer to a properly initialized `WndClassExW` structure.
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
pub(crate) unsafe extern "stdcall" fn RegisterClassW(wc: *const WndClassExW) -> u16 {
    common::register_class_w(wc)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn register_class_ex_a(wc: *const WndClassExA) -> u16 {
    RegisterClassA(wc)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn register_class_ex_w(wc: *const WndClassExW) -> u16 {
    RegisterClassW(wc)
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
