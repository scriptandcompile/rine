//! Window class registration — shared logic for RegisterClass(Ex)A/W.

use std::sync::Once;

use rine_types::errors::BOOL;
use rine_types::windows::*;
use text::{read_cstr, read_wstr};

pub(crate) const INTERNAL_DEFAULT_WND_PROC: usize = usize::MAX;

const BUILTIN_CONTROL_CLASS_ALIASES: &[&str] = &[
    "STATIC", "static", "Static", "BUTTON", "button", "Button", "EDIT", "edit", "Edit",
];

pub(crate) fn is_internal_default_wnd_proc(proc_fn: usize) -> bool {
    proc_fn == INTERNAL_DEFAULT_WND_PROC
}

pub(crate) fn ensure_builtin_classes_registered() {
    static INIT: Once = Once::new();
    INIT.call_once(register_builtin_control_classes);
}

fn register_builtin_control_classes() {
    for &alias in BUILTIN_CONTROL_CLASS_ALIASES {
        WINDOW_CLASS_REGISTRY.register(
            alias.to_string(),
            WindowClass {
                name: alias.to_string(),
                style: 0,
                wnd_proc: INTERNAL_DEFAULT_WND_PROC,
                cls_extra: 0,
                wnd_extra: 0,
                instance: 0,
                icon: 0,
                cursor: 0,
                background: 0,
                menu_name: None,
                icon_sm: 0,
            },
        );
    }
}

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
    ensure_builtin_classes_registered();

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
    ensure_builtin_classes_registered();

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
    ensure_builtin_classes_registered();

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
    ensure_builtin_classes_registered();

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_control_classes_are_registered() {
        ensure_builtin_classes_registered();

        assert!(WINDOW_CLASS_REGISTRY.get("STATIC").is_some());
        assert!(WINDOW_CLASS_REGISTRY.get("BUTTON").is_some());
        assert!(WINDOW_CLASS_REGISTRY.get("EDIT").is_some());

        let static_class = WINDOW_CLASS_REGISTRY
            .get("STATIC")
            .expect("STATIC class should exist");
        assert!(is_internal_default_wnd_proc(static_class.wnd_proc));
    }
}
