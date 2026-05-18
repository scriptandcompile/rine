use rine_common_user32 as common;
use rine_types::handles::{HACCEL, HINSTANCE};
use rine_types::strings::{LPCSTR, LPCWSTR};
use rine_types::windows::{HWND, Msg};

/// Loads an accelerator table from the specified module.
///
/// # Arguments
/// * `instance` - Handle to the module containing the accelerator resource.
/// * `table_name` - Integer resource identifier (`MAKEINTRESOURCE`) of the accelerator table.
///
/// # Safety
/// `table_name` must be null or a valid resource identifier pointer.
///
/// # Returns
/// A handle to the loaded accelerator table, or `HACCEL::NULL` on failure.
///
/// # Notes
/// Missing implementation features:
/// - Only integer resource identifiers are supported.
/// - Resource-name string lookup is not implemented.
/// - Resource lookup is currently limited to the main guest executable module.
/// - `instance`-based module-name lookup is not implemented.
/// - Entries without `FVIRTKEY` are skipped.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn LoadAcceleratorsA(instance: HINSTANCE, table_name: LPCSTR) -> HACCEL {
    let Some(resource_id) = integer_resource_id_from_ansi(table_name) else {
        return HACCEL::NULL;
    };

    common::accelerator::load_accelerators(instance, resource_id)
}

/// Loads an accelerator table from the specified module.
///
/// # Arguments
/// * `instance` - Handle to the module containing the accelerator resource.
/// * `table_name` - Integer resource identifier (`MAKEINTRESOURCE`) of the accelerator table.
///
/// # Safety
/// `table_name` must be null or a valid resource identifier pointer.
///
/// # Returns
/// A handle to the loaded accelerator table, or `HACCEL::NULL` on failure.
///
/// # Notes
/// Missing implementation features:
/// - Only integer resource identifiers are supported.
/// - Resource-name string lookup is not implemented.
/// - Resource lookup is currently limited to the main guest executable module.
/// - `instance`-based module-name lookup is not implemented.
/// - Entries without `FVIRTKEY` are skipped.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn LoadAcceleratorsW(instance: HINSTANCE, table_name: LPCWSTR) -> HACCEL {
    let Some(resource_id) = integer_resource_id_from_wide(table_name) else {
        return HACCEL::NULL;
    };

    common::accelerator::load_accelerators(instance, resource_id)
}

/// Translates keyboard messages using an accelerator table.
///
/// # Arguments
/// * `hwnd` - Frame window receiving translated `WM_COMMAND` messages.
/// * `haccel` - Accelerator table handle from `LoadAccelerators`.
/// * `msg` - Pointer to the message to translate.
///
/// # Safety
/// `msg` must point to a valid `Msg` when non-null.
///
/// # Returns
/// `1` when a message is translated, otherwise `0`.
///
/// # Notes
/// Missing implementation features:
/// - Matching currently supports only `FVIRTKEY` accelerator entries.
/// - Character/dead-key/keyboard-layout-aware translation is not implemented.
/// - Hosted-window keyboard bridge support is not implemented.
/// - `FNOINVERT` visual behavior is not implemented.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn TranslateAcceleratorA(
    hwnd: HWND,
    haccel: HACCEL,
    msg: *const Msg,
) -> i32 {
    unsafe {
        common::accelerator::translate_accelerator(hwnd, haccel, msg, |proc_fn, hwnd, m, wp, lp| {
            let f: extern "win64" fn(HWND, u32, usize, isize) -> isize =
                std::mem::transmute(proc_fn);
            f(hwnd, m, wp, lp)
        })
    }
}

/// Translates keyboard messages using an accelerator table.
///
/// # Arguments
/// * `hwnd` - Frame window receiving translated `WM_COMMAND` messages.
/// * `haccel` - Accelerator table handle from `LoadAccelerators`.
/// * `msg` - Pointer to the message to translate.
///
/// # Safety
/// `msg` must point to a valid `Msg` when non-null.
///
/// # Returns
/// `1` when a message is translated, otherwise `0`.
///
/// # Notes
/// Missing implementation features:
/// - Matching currently supports only `FVIRTKEY` accelerator entries.
/// - Character/dead-key/keyboard-layout-aware translation is not implemented.
/// - Hosted-window keyboard bridge support is not implemented.
/// - `FNOINVERT` visual behavior is not implemented.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn TranslateAcceleratorW(
    hwnd: HWND,
    haccel: HACCEL,
    msg: *const Msg,
) -> i32 {
    unsafe {
        common::accelerator::translate_accelerator(hwnd, haccel, msg, |proc_fn, hwnd, m, wp, lp| {
            let f: extern "win64" fn(HWND, u32, usize, isize) -> isize =
                std::mem::transmute(proc_fn);
            f(hwnd, m, wp, lp)
        })
    }
}

fn integer_resource_id_from_ansi(table_name: LPCSTR) -> Option<u16> {
    if table_name.is_null() {
        return None;
    }

    let raw = table_name.as_ptr() as usize;
    if raw <= 0xFFFF {
        Some(raw as u16)
    } else {
        None
    }
}

fn integer_resource_id_from_wide(table_name: LPCWSTR) -> Option<u16> {
    if table_name.is_null() {
        return None;
    }

    let raw = table_name.as_ptr() as usize;
    if raw <= 0xFFFF {
        Some(raw as u16)
    } else {
        None
    }
}
