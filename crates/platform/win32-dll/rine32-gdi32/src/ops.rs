use rine_common_gdi32 as common;
use rine_types::errors::WinBool;
use rine_types::strings::{read_cstr_counted, read_wstr_counted};
use rine_types::windows::Rect;

/// Creates a memory device context (DC) compatible with the specified device.
///
/// # Arguments
/// * `_hdc`: A handle to an existing DC.
///   If this handle is NULL, the function creates a memory DC compatible with the application's current screen.
///   Currently, this parameter is ignored and the created DC is always compatible with the application's current screen.
///
/// # Safety
/// The caller must ensure that `_hdc` is a valid device context handle or NULL.
/// The returned handle must be deleted with `delete_dc` when no longer needed to avoid resource leaks.
///
/// # Returns
/// A handle to the compatible memory DC, or 0 if the function fails.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn CreateCompatibleDC(_hdc: usize) -> usize {
    unsafe { common::create_compatible_dc(_hdc) }
}

/// Deletes a device context (DC) and all GDI objects owned by it.
///
/// # Arguments
/// * `hdc`: A handle to the DC to be deleted. This handle must have been returned by a previous call to `create_compatible_dc`.
///
/// # Safety
/// The caller must pass a valid DC handle that belongs to this runtime.
/// After this call, the handle and any GDI objects owned by it must not be used, as they have been freed.
/// This function will fail if any of the DC's selected objects are still selected in any DC (including itself).
///
/// # Returns
/// Returns `WinBool::TRUE` if the DC was successfully deleted,
/// or `WinBool::FALSE` if the handle was invalid or if any selected objects are still in use.///
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn DeleteDC(hdc: usize) -> WinBool {
    unsafe { common::delete_dc(hdc) }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn create_compatible_bitmap(
    hdc: usize,
    width: i32,
    height: i32,
) -> usize {
    unsafe { common::create_compatible_bitmap(hdc, width, height) }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn create_solid_brush(color: u32) -> usize {
    unsafe { common::create_solid_brush(color) }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn create_pen(style: i32, width: i32, color: u32) -> usize {
    unsafe { common::create_pen(style, width, color) }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn select_object(hdc: usize, object: usize) -> usize {
    unsafe { common::select_object(hdc, object) }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn delete_object(object: usize) -> WinBool {
    unsafe { common::delete_object(object) }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn bit_blt(
    hdc_dest: usize,
    x_dest: i32,
    y_dest: i32,
    width: i32,
    height: i32,
    hdc_src: usize,
    x_src: i32,
    y_src: i32,
    rop: u32,
) -> WinBool {
    let dest_rect = Rect {
        left: x_dest,
        top: y_dest,
        right: x_dest.saturating_add(width),
        bottom: y_dest.saturating_add(height),
    };
    let src_rect = Rect {
        left: x_src,
        top: y_src,
        right: x_src.saturating_add(width),
        bottom: y_src.saturating_add(height),
    };

    unsafe { common::bit_blt(hdc_dest, dest_rect, hdc_src, src_rect, rop) }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn TextOutA(
    hdc: usize,
    x: i32,
    y: i32,
    text: *const u8,
    count: i32,
) -> WinBool {
    unsafe {
        let Some(text) = read_cstr_counted(text, count) else {
            return WinBool::FALSE;
        };

        common::ops::text_out(hdc, x, y, &text)
    }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "stdcall" fn TextOutW(
    hdc: usize,
    x: i32,
    y: i32,
    text: *const u16,
    count: i32,
) -> WinBool {
    unsafe {
        let Some(text) = read_wstr_counted(text, count) else {
            return WinBool::FALSE;
        };

        common::ops::text_out(hdc, x, y, &text)
    }
}
