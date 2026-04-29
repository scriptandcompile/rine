use rine_common_user32::class_registration as common;
use rine_types::errors::WinBool;
use rine_types::strings::{LPCSTR, LPCWSTR};
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
/// Missing implementation features:
/// - No Win32-style atom allocation table is maintained (success always returns 1).
/// - No detailed validation of class fields/styles is performed.
/// - No Win32-accurate `GetLastError` mapping is provided on failure.
/// - Instance/namespace semantics are simplified compared with Windows.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn RegisterClassA(wc: *const WndClassA) -> ATOM {
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
/// * Atom of the registered class on success, 0 on failure.
///  
/// # Notes
/// Missing implementation features:
/// - No Win32-style atom allocation table is maintained (success always returns 1).
/// - No detailed validation of class fields/styles is performed.
/// - No Win32-accurate `GetLastError` mapping is provided on failure.
/// - Instance/namespace semantics are simplified compared with Windows.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn RegisterClassW(wc: *const WndClassW) -> ATOM {
    common::register_class_w(wc)
}

/// Window class registration (ex versions)
///
/// # Arguments
/// * `wc` - Pointer to a `WndClassExA` structure containing the window class information.
///
/// # Safety
/// * The caller must ensure that `wc` is a valid pointer to a properly initialized `WndClassExA` structure.
/// * The caller must ensure that the window class name is unique and not already registered,
///
/// # Returns
/// * Atom of the registered class on success, 0 on failure.
///
/// # Notes
/// Missing implementation features:
/// - No Win32-style atom allocation table is maintained (success always returns 1).
/// - No detailed validation of class fields/styles is performed.
/// - No Win32-accurate `GetLastError` mapping is provided on failure.
/// - Instance/namespace semantics are simplified compared with Windows.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn RegisterClassExA(wc: *const WndClassExA) -> ATOM {
    common::register_class_ex_a(wc)
}

/// Window class registration (ex versions)
///
/// # Arguments
/// * `wc` - Pointer to a `WndClassExW` structure containing the window class information.
///
/// # Safety
/// * The caller must ensure that `wc` is a valid pointer to a properly initialized `WndClassExW` structure.
/// * The caller must ensure that the window class name is unique and not already registered,
///
/// # Returns
/// * Atom of the registered class on success, 0 on failure.
///
/// # Notes
/// Missing implementation features:
/// - No Win32-style atom allocation table is maintained (success always returns 1).
/// - No detailed validation of class fields/styles is performed.
/// - No Win32-accurate `GetLastError` mapping is provided on failure.
/// - Instance/namespace semantics are simplified compared with Windows.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn RegisterClassExW(wc: *const WndClassExW) -> ATOM {
    common::register_class_ex_w(wc)
}

/// Unregister window class
///
/// # Arguments
/// * `class_name` - Pointer to a null-terminated string containing the name of the class to unregister.
/// * `_h_instance` - Handle to the instance of the module that registered the class.
///   This parameter is ignored in this implementation since we don't manage instances.
///
/// # Safety
/// * The caller must ensure that `class_name` is a valid pointer to a null-terminated string.
/// * The caller must ensure that the class name corresponds to a registered class, or that it is
///   safe to attempt to unregister a non-existent class (which will simply fail and return `WinBool::FALSE`).
///
/// # Returns
/// `WinBool::TRUE` if the class was found and unregistered, `WinBool::FALSE` if the class was not found.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn UnregisterClassA(class_name: LPCSTR, _h_instance: usize) -> WinBool {
    let name = class_name.read_string().unwrap_or_default();
    common::unregister_class(&name)
}

/// Unregister window class
///
/// # Arguments
/// * `class_name` - Pointer to a null-terminated string containing the name of the class to unregister.
/// * `_h_instance` - Handle to the instance of the module that registered the class.
///   This parameter is ignored in this implementation since we don't manage instances.
///
/// # Safety
/// * The caller must ensure that `class_name` is a valid pointer to a null-terminated string.
/// * The caller must ensure that the class name corresponds to a registered class, or that it is
///   safe to attempt to unregister a non-existent class (which will simply fail and return `WinBool::FALSE`).
///
/// # Returns
/// `WinBool::TRUE` if the class was found and unregistered, `WinBool::FALSE` if the class was not found.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn UnregisterClassW(
    class_name: LPCWSTR,
    _h_instance: usize,
) -> WinBool {
    let name = class_name.read_string().unwrap_or_default();
    common::unregister_class(&name)
}
