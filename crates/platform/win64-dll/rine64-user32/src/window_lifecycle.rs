use rine_common_user32 as common;
use rine_types::errors::WinBool;
use rine_types::strings::{read_cstr, read_wstr};
use rine_types::windows::*;

/// Create a new window.
///
/// # Arguments
/// * `ex_style`: Extended window style.
/// * `class_name`: Name of the window class to use.
/// * `window_name`: Window title.
/// * `style`: Window style.
/// * `x`: X position of the window.
/// * `y`: Y position of the window.
/// * `width`: Width of the window.
/// * `height`: Height of the window.
/// * `parent`: Handle of the parent window, or 0 for no parent.
/// * `_menu`: Handle of the menu, or 0 for no menu. Ignored.
/// * `_instance`: Handle of the instance, or 0 for the current instance. Ignored.
/// * `_param`: Pointer to window creation data. Ignored.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers and interacts with the window manager.
/// The caller must ensure that the pointers are valid and that the window manager is in a consistent state.
/// The caller must also ensure that the provided parent handle (if any) is valid and belongs to this runtime.
/// The caller must also ensure that the window class specified by `class_name` has been registered before calling this function.
/// The caller is responsible for eventually destroying the created window to avoid resource leaks.
///
/// # Returns
/// The handle of the created window, or 0 on failure.
///
/// # Notes
/// Currently, the `_menu`, `_instance`, and `_param` parameters are ignored, as they are not commonly used in typical
/// window creation scenarios and would require additional infrastructure to support properly.
/// On error the `GetLastError` code should be set to indicate the reason for failure, such as `ERROR_CLASS_NOT_FOUND`
/// if the specified class name does not exist. Currently, we do not set `GetLastError`.
#[allow(non_snake_case, clippy::too_many_arguments)]
#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn CreateWindowExA(
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

/// Create a new window.
///
/// # Arguments
/// * `ex_style`: Extended window style.
/// * `class_name`: Name of the window class to use.
/// * `window_name`: Window title.
/// * `style`: Window style.
/// * `x`: X position of the window.
/// * `y`: Y position of the window.
/// * `width`: Width of the window.
/// * `height`: Height of the window.
/// * `parent`: Handle of the parent window, or 0 for no parent.
/// * `_menu`: Handle of the menu, or 0 for no menu. Ignored.
/// * `_instance`: Handle of the instance, or 0 for the current instance. Ignored.
/// * `_param`: Pointer to window creation data. Ignored.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers and interacts with the window manager.
/// The caller must ensure that the pointers are valid and that the window manager is in a consistent state.
/// The caller must also ensure that the provided parent handle (if any) is valid and belongs to this runtime.
/// The caller must also ensure that the window class specified by `class_name` has been registered before calling this function.
/// The caller is responsible for eventually destroying the created window to avoid resource leaks.
///
/// # Returns
/// The handle of the created window, or 0 on failure.
///
/// # Notes
/// Currently, the `_menu`, `_instance`, and `_param` parameters are ignored, as they are not commonly used in typical
/// window creation scenarios and would require additional infrastructure to support properly.
/// On error the `GetLastError` code should be set to indicate the reason for failure, such as `ERROR_CLASS_NOT_FOUND`
/// if the specified class name does not exist. Currently, we do not set `GetLastError`.
#[allow(non_snake_case, clippy::too_many_arguments)]
#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn CreateWindowExW(
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

/// Destroy a window.
///
/// # Arguments
/// * `hwnd`: Handle of the window to destroy.
///
/// # Safety
/// The caller must pass a valid window handle that belongs to this runtime and provide a callback
/// that can safely invoke the target window procedure.
/// The caller is responsible for ensuring that the window is not used after it has been destroyed,
/// as this would lead to undefined behavior.
/// The caller must also ensure that any necessary synchronization is performed if the window is
/// accessed from multiple threads.
/// The caller must also ensure that the window procedure callback provided to this function can
/// safely call back into the caller's ABI, as this function will invoke the window procedure with
/// WM_DESTROY to allow for proper cleanup.
/// The caller is responsible for eventually destroying the created window to avoid resource leaks.
///
/// # Returns
/// 1 on success, 0 if the HWND was not found.
///
/// # Notes
/// On error the `GetLastError` code should be set to indicate the reason for failure, such as
/// `ERROR_INVALID_WINDOW_HANDLE` if the specified handle does not correspond to a valid window.
/// Currently, we do not set `GetLastError`.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn DestroyWindow(hwnd: usize) -> i32 {
    unsafe {
        common::destroy_window(hwnd, |proc_fn, h, msg, wp, lp| {
            let f: extern "win64" fn(usize, u32, usize, isize) -> isize =
                std::mem::transmute(proc_fn);
            f(h, msg, wp, lp)
        })
    }
}

/// Show a window.
///
/// # Arguments
/// * `hwnd`: Handle of the window to show.
/// * `cmd_show`: Show command (e.g. SW_SHOW).
///
/// # Safety
/// The caller must pass a valid window handle that belongs to this runtime.
/// The caller is responsible for ensuring that the window is not used after it has been destroyed,
/// as this would lead to undefined behavior.
///
/// # Returns
/// The return value is the result of the `ShowWindow` operation, which is nonzero if the window was
/// previously visible and zero if it was hidden.
/// On error (e.g. if the window handle is invalid), the function returns 0, which is the same as the
/// return value for a window that was hidden.
///
/// # Notes
/// Currently, we do not set `GetLastError`, so there is no way to distinguish between these cases.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn ShowWindow(hwnd: usize, cmd_show: i32) -> WinBool {
    common::show_window(hwnd, cmd_show)
}

/// Request a WM_PAINT for the given window.
///
/// # Arguments
/// * `hwnd`: Handle of the window to update.
///
/// # Safety
/// The caller must pass a valid window handle that belongs to this runtime.
/// The caller is responsible for ensuring that the window is not used after it has been destroyed,
/// as this would lead to undefined behavior.
/// The caller must also ensure that any necessary synchronization is performed if the window is
/// accessed from multiple threads.
///
/// # Returns
/// `WinBool::TRUE` always (UpdateWindow is a notification, not a query).
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn UpdateWindow(hwnd: usize) -> WinBool {
    common::update_window(hwnd)
}
