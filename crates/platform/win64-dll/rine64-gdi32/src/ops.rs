use rine_common_gdi32 as common;
use rine_types::errors::WinBool;
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
pub(crate) unsafe extern "win64" fn CreateCompatibleDC(_hdc: usize) -> usize {
    unsafe { common::create_compatible_dc(_hdc) }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn delete_dc(hdc: usize) -> WinBool {
    unsafe { common::delete_dc(hdc) }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn create_compatible_bitmap(
    _hdc: usize,
    width: i32,
    height: i32,
) -> usize {
    unsafe { common::create_compatible_bitmap(_hdc, width, height) }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn create_solid_brush(color: u32) -> usize {
    unsafe { common::create_solid_brush(color) }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn create_pen(_style: i32, _width: i32, color: u32) -> usize {
    unsafe { common::create_pen(_style, _width, color) }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn select_object(hdc: usize, object: usize) -> usize {
    unsafe { common::select_object(hdc, object) }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn delete_object(object: usize) -> WinBool {
    unsafe { common::delete_object(object) }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn bit_blt(
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
pub(crate) unsafe extern "win64" fn text_out_a(
    hdc: usize,
    x: i32,
    y: i32,
    text: *const u8,
    count: i32,
) -> WinBool {
    unsafe { common::text_out_a(hdc, x, y, text, count) }
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "win64" fn text_out_w(
    hdc: usize,
    x: i32,
    y: i32,
    text: *const u16,
    count: i32,
) -> WinBool {
    unsafe { common::text_out_w(hdc, x, y, text, count) }
}
