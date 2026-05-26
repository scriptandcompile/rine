use rine_common_user32 as common;
use rine_types::errors::BOOL;
use rine_types::handles::HMENU;
use rine_types::strings::{LPCSTR, LPCWSTR};
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
/// * `parent`: Handle of the parent window, or `HWND::NULL` for no parent.
/// * `_menu`: Handle of the menu for top-level windows, or child control identifier when `WS_CHILD` is set.
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
/// For child windows (`WS_CHILD`), `_menu` is recorded as the control ID used by dialog item APIs.
/// `_instance` and `_param` are currently ignored.
/// On error the `GetLastError` code should be set to indicate the reason for failure, such as `ERROR_CLASS_NOT_FOUND`
/// if the specified class name does not exist. Currently, we do not set `GetLastError`.
#[rine_dlls::partial]
#[allow(non_snake_case, clippy::too_many_arguments)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn CreateWindowExA(
    ex_style: u32,
    class_name: LPCSTR,
    window_name: LPCSTR,
    style: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    parent: HWND,
    _menu: HMENU,
    _instance: usize,
    _param: *mut u8,
) -> HWND {
    let hwnd = common::create_window(
        ex_style,
        class_name.read_string().unwrap_or_default(),
        window_name.read_string().unwrap_or_default(),
        style,
        Rect {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        },
        parent,
    );

    if !hwnd.is_null() && (style & window_style::WS_CHILD) != 0 {
        let _ = common::set_window_control_id(hwnd, _menu.as_raw() as i32);
    }

    hwnd
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
/// * `parent`: Handle of the parent window, or `HWND::NULL` for no parent.
/// * `_menu`: Handle of the menu for top-level windows, or child control identifier when `WS_CHILD` is set.
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
/// The handle of the created window, or `HWND::NULL` on failure.
///
/// # Notes
/// For child windows (`WS_CHILD`), `_menu` is recorded as the control ID used by dialog item APIs.
/// `_instance` and `_param` are currently ignored.
/// On error the `GetLastError` code should be set to indicate the reason for failure, such as `ERROR_CLASS_NOT_FOUND`
/// if the specified class name does not exist. Currently, we do not set `GetLastError`.
#[rine_dlls::partial]
#[allow(non_snake_case, clippy::too_many_arguments)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn CreateWindowExW(
    ex_style: u32,
    class_name: LPCWSTR,
    window_name: LPCWSTR,
    style: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    parent: HWND,
    _menu: HMENU,
    _instance: usize,
    _param: *mut u8,
) -> HWND {
    let hwnd = common::create_window(
        ex_style,
        class_name.read_string().unwrap_or_default(),
        window_name.read_string().unwrap_or_default(),
        style,
        Rect {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        },
        parent,
    );

    if !hwnd.is_null() && (style & window_style::WS_CHILD) != 0 {
        let _ = common::set_window_control_id(hwnd, _menu.as_raw() as i32);
    }

    hwnd
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
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn DestroyWindow(hwnd: HWND) -> BOOL {
    unsafe {
        common::destroy_window(hwnd, |proc_fn, h, msg, wp, lp| {
            let f: extern "stdcall" fn(HWND, u32, usize, isize) -> isize =
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
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn ShowWindow(hwnd: HWND, cmd_show: i32) -> BOOL {
    common::show_window(hwnd, cmd_show)
}

/// Determine whether a window is minimized (iconic).
///
/// # Arguments
/// * `hwnd`: Handle of the window to query.
///
/// # Safety
/// The caller must pass a valid window handle that belongs to this runtime.
///
/// # Returns
/// `BOOL::TRUE` if the window is minimized, `BOOL::FALSE` otherwise.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn IsIconic(hwnd: HWND) -> BOOL {
    common::is_iconic(hwnd)
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
/// `BOOL::TRUE` always (UpdateWindow is a notification, not a query).
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn UpdateWindow(hwnd: HWND) -> BOOL {
    common::update_window(hwnd)
}

/// Mark part of the window's client area as needing a redraw.
///
/// # Arguments
/// * `hwnd`: Handle of the window to invalidate.
/// * `lp_rect`: Pointer to a `Rect` structure, or null to invalidate the entire client area.
/// * `b_erase`: Whether to erase the background during the next `WM_PAINT`.
///
/// # Safety
/// The caller must pass a valid window handle that belongs to this runtime.
/// If `lp_rect` is non-null it must point to a valid `Rect`.
///
/// # Returns
/// `BOOL::TRUE` on success, `BOOL::FALSE` if the HWND is not found.
///
/// # Notes
/// The `lp_rect` parameter is accepted but ignored; the entire client area is
/// always invalidated. On error the `GetLastError` code should be set to indicate
/// the reason for failure, such as `ERROR_INVALID_WINDOW_HANDLE` if the specified
/// handle does not correspond to a valid window. Currently, we do not set `GetLastError`.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn InvalidateRect(
    hwnd: HWND,
    lp_rect: *const Rect,
    b_erase: BOOL,
) -> BOOL {
    common::invalidate_rect(hwnd, lp_rect.as_ref(), b_erase)
}

/// Retrieve the client-area coordinates of a window.
///
/// # Arguments
/// * `hwnd`: Handle of the window.
/// * `lp_rect`: Pointer to a `Rect` structure that receives the client coordinates.
///
/// # Safety
/// The caller must pass a valid window handle that belongs to this runtime.
/// `lp_rect` must point to a valid, writable `Rect`.
///
/// # Returns
/// `BOOL::TRUE` on success, `BOOL::FALSE` if the HWND is not found.
///
/// # Notes
/// The `left` and `top` members are always 0; `right` and `bottom` contain the
/// width and height of the client area. This value is updated by `MoveWindow`
/// and `CreateWindowEx`, but does **not** account for:
/// - Non-client areas (menus, scroll bars, title bars) — requires
///   `AdjustWindowRect[Ex]` which is not implemented yet.
/// - DPI scaling / per-monitor DPI awareness.
/// - Native compositor decorations on the underlying winit window.
/// - `WM_NCCALCSIZE` handling — custom non-client areas will show stale values.
/// - On error the `GetLastError` code should be set to `ERROR_INVALID_WINDOW_HANDLE`.
///   Currently, we do not set `GetLastError`.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetClientRect(hwnd: HWND, lp_rect: *mut Rect) -> BOOL {
    let mut rect = Rect::default();
    let result = common::get_client_rect(hwnd, &mut rect);
    if result == BOOL::TRUE {
        unsafe { *lp_rect = rect };
    }
    result
}

/// Convert screen coordinates to client-area coordinates.
///
/// # Arguments
/// * `hwnd`: Handle of the window whose client area defines the coordinate
///   space to convert into.
/// * `lp_point`: Pointer to a `Point` structure containing screen coordinates.
///   On success the structure is overwritten with the corresponding client-area
///   coordinates.
///
/// # Safety
/// The caller must pass a valid window handle that belongs to this runtime.
/// `lp_point` must point to a valid, readable and writable `Point`.
///
/// # Returns
/// `BOOL::TRUE` on success, `BOOL::FALSE` if the HWND is not found.
///
/// # Notes
/// The current implementation uses the window's `rect` origin as the client-area
/// origin on screen. It does **not** account for:
/// - Non-client areas (title bar, borders, scroll bars) — the offset between
///   the window origin and the client-area origin is not tracked.
/// - DPI scaling / per-monitor DPI awareness.
/// - Child-window coordinate chains.
/// - On error the `GetLastError` code should be set to
///   `ERROR_INVALID_WINDOW_HANDLE`. Currently, we do not set `GetLastError`.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn ScreenToClient(hwnd: HWND, lp_point: *mut Point) -> BOOL {
    let mut point = unsafe { *lp_point };
    let result = common::screen_to_client(hwnd, &mut point);
    if result == BOOL::TRUE {
        unsafe { *lp_point = point };
    }
    result
}

/// Enable or disable mouse and keyboard input to the specified window.
///
/// # Arguments
/// * `hwnd`: Handle of the window.
/// * `enable`: Whether to enable (`BOOL::TRUE`) or disable (`BOOL::FALSE`) the window.
///
/// # Safety
/// The caller must pass a valid window handle that belongs to this runtime.
///
/// # Returns
/// `BOOL::TRUE` if the window was previously enabled, `BOOL::FALSE` otherwise.
///
/// # Notes
/// Currently, we do not set `GetLastError`. On error (e.g. invalid HWND) the function
/// returns `BOOL::FALSE`, which is indistinguishable from a window that was already
/// disabled. The native backend is not notified of enable state changes, so disabled
/// windows may still receive native input events.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn EnableWindow(hwnd: HWND, enable: BOOL) -> BOOL {
    common::enable_window(hwnd, enable)
}

/// Move and optionally resize a window.
///
/// # Arguments
/// * `hwnd`: Handle of the window.
/// * `x`: New x position.
/// * `y`: New y position.
/// * `width`: New width.
/// * `height`: New height.
/// * `repaint`: Whether to repaint the window.
///
/// # Safety
/// The caller must pass a valid window handle that belongs to this runtime.
///
/// # Returns
/// `BOOL::TRUE` on success, `BOOL::FALSE` if the HWND is not found.
///
/// # Notes
/// Currently, we do not set `GetLastError`. The `repaint` parameter only controls
/// whether `WM_PAINT` is posted; native backend repaint is always performed.
/// Child-window coordinate translation (for `WS_CHILD` windows) is not implemented.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn MoveWindow(
    hwnd: HWND,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    repaint: BOOL,
) -> BOOL {
    common::move_window(hwnd, x, y, width, height, repaint)
}

/// ChildWindowFromPoint — determines which child window (if any) contains
/// the specified point.
///
/// # Arguments
/// * `hwnd_parent`: Handle of the parent window to search for children.
/// * `pt`: A `Point` structure containing the point to search for, in the
///   parent's client coordinates.
///
/// # Safety
/// The caller must pass a valid window handle that belongs to this runtime.
/// The `pt` structure is passed by value and does not require any pointer
/// validity guarantees.
///
/// # Returns
/// The handle of the child window that contains the point, or the handle of
/// the parent window if no child contains the point. Returns `HWND::NULL` if
/// the parent handle does not correspond to a valid window.
///
/// # Notes
/// Only visible, enabled, immediate children of the parent are considered.
/// Grandchildren are not searched. If multiple children contain the point,
/// the topmost child (highest in Z-order) is returned. The current
/// implementation does **not** account for:
/// - Transparent / layered child windows that should be skipped
///   (use `ChildWindowFromPointEx` with `CWP_SKIPTRANSPARENT` for that).
/// - Clipped children — children that are partially or fully outside the
///   parent's client area are still candidates.
/// - `GetLastError` is not set on failure.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn ChildWindowFromPoint(hwnd_parent: HWND, pt: Point) -> HWND {
    common::child_window_from_point(hwnd_parent, pt)
}
