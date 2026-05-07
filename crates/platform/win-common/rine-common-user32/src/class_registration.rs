//! Window class registration — shared logic for RegisterClass(Ex)A/W.

use rine_types::errors::BOOL;
use rine_types::strings::{read_cstr, read_wstr};
use rine_types::windows::*;

/// Window class registration
///
/// # Arguments
/// * `wc` - Pointer to a `WndClassA` structure containing the window class information.
///
/// # Safety
/// * The caller must ensure that `wc` is a valid pointer to a properly initialized `WndClassA` structure.
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
pub unsafe fn register_class_a(wc: *const WndClassA) -> ATOM {
    if wc.is_null() {
        return 0;
    }

    let wc = &*wc;
    let class_name = read_cstr(wc.lpsz_class_name).unwrap_or_default();
    let menu_name = read_cstr(wc.lpsz_menu_name);
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
        icon_sm: 0,
    };

    register_class(class_name, class)
}

/// Window class registration
///
/// # Arguments
/// * `wc` - Pointer to a `WndClassW` structure containing the window class information.
///
/// # Safety
/// * The caller must ensure that `wc` is a valid pointer to a properly initialized `WndClassW` structure.
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
pub unsafe fn register_class_w(wc: *const WndClassW) -> ATOM {
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
        icon_sm: 0,
    };

    register_class(class_name, class)
}

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
pub unsafe fn register_class_ex_a(wc: *const WndClassExA) -> ATOM {
    if wc.is_null() {
        return 0;
    }

    let wc = &*wc;
    let class_name = read_cstr(wc.lpsz_class_name).unwrap_or_default();
    let menu_name = read_cstr(wc.lpsz_menu_name);
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
/// Missing implementation features:
/// - No Win32-style atom allocation table is maintained (success always returns 1).
/// - No detailed validation of class fields/styles is performed.
/// - No Win32-accurate `GetLastError` mapping is provided on failure.
/// - Instance/namespace semantics are simplified compared with Windows.
pub unsafe fn register_class_ex_w(wc: *const WndClassExW) -> ATOM {
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

/// Register a window class by name.
///
/// Returns 1 on success, 0 if the name is empty.
fn register_class(name: String, class: WindowClass) -> ATOM {
    if name.is_empty() {
        return 0;
    }
    WINDOW_CLASS_REGISTRY.register(name, class);
    1
}

/// Unregister a previously registered window class.
///
/// # Arguments
/// * `name` - The name of the window class to unregister.
///
/// # Returns
/// * `BOOL::TRUE` if the class was successfully unregistered, `BOOL::FALSE` if the class was not found.
pub fn unregister_class(name: &str) -> BOOL {
    if WINDOW_CLASS_REGISTRY.unregister(name) {
        BOOL::TRUE
    } else {
        BOOL::FALSE
    }
}
