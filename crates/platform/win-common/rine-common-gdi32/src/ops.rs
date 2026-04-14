use rine_types::errors::WinBool;
use rine_types::windows::Rect;

use crate::objects::{Bitmap, Brush, DeviceContext, GdiObject, Pen};
use crate::state::{alloc_handle, gdi_state, object_selected_by_any_dc, with_selected_bitmap_mut};
use crate::telemetry::{notify_bitmap_alloc, notify_bitmap_free};
use crate::text::draw_text;

pub const SRCCOPY: u32 = 0x00CC0020;

pub(crate) fn bitmap_bytes(bitmap: &Bitmap) -> u64 {
    (bitmap.pixels.len() * std::mem::size_of::<u32>()) as u64
}

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
pub unsafe fn create_compatible_dc(_hdc: usize) -> usize {
    let mut state = gdi_state().lock().unwrap();
    let dc_handle = alloc_handle();

    let mut dc = DeviceContext::default();

    let default_bitmap_handle = alloc_handle();
    dc.selected_bitmap = Some(default_bitmap_handle);
    dc.owned_objects.push(default_bitmap_handle);
    let default_bitmap = Bitmap::new(1, 1).unwrap();
    notify_bitmap_alloc(default_bitmap_handle, &default_bitmap);
    state
        .objects
        .insert(default_bitmap_handle, GdiObject::Bitmap(default_bitmap));

    state.dcs.insert(dc_handle, dc);

    let detail = format!(r#"{{"hdc":{}}}"#, dc_handle);
    rine_types::dev_notify!(on_handle_created(dc_handle as i64, "GdiDc", &detail));

    dc_handle
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
/// or `WinBool::FALSE` if the handle was invalid or if any selected objects are still in use.
pub unsafe fn delete_dc(hdc: usize) -> WinBool {
    let mut state = gdi_state().lock().unwrap();
    let Some(dc) = state.dcs.remove(&hdc) else {
        return WinBool::FALSE;
    };

    for object in dc.owned_objects {
        if let Some(gdi_object) = state.objects.remove(&object) {
            if let GdiObject::Bitmap(bitmap) = &gdi_object {
                notify_bitmap_free(bitmap);
            }
            rine_types::dev_notify!(on_handle_closed(object as i64));
        }
    }

    rine_types::dev_notify!(on_handle_closed(hdc as i64));
    WinBool::TRUE
}

/// Creates a bitmap compatible with the device that is associated with the specified device context (DC).
///
/// # Arguments
/// * `_hdc`: A handle to a DC. The bitmap created will be compatible with the device associated with this DC.
///   If this handle is NULL, the bitmap will be compatible with the application's current screen.
///   Currently, this parameter is ignored and the created bitmap is always compatible with the application's current screen.
/// * `width`: The width of the bitmap, in pixels. Must be greater than 0.
/// * `height`: The height of the bitmap, in pixels. Must be greater than 0.
///
/// # Safety
/// The caller must ensure that `_hdc` is a valid device context handle or NULL.
/// The returned handle must be deleted with `delete_object` when no longer needed to avoid resource leaks.
/// The caller is responsible for ensuring that the width and height are positive to avoid unexpected results.
///
/// # Returns
/// A handle to the compatible bitmap, or 0 if the function fails
/// (e.g., if the dimensions are invalid or if there are insufficient resources to create the bitmap).
pub unsafe fn create_compatible_bitmap(_hdc: usize, width: i32, height: i32) -> usize {
    let Some(bitmap) = Bitmap::new(width, height) else {
        return 0;
    };

    let mut state = gdi_state().lock().unwrap();
    let handle = alloc_handle();
    notify_bitmap_alloc(handle, &bitmap);
    state.objects.insert(handle, GdiObject::Bitmap(bitmap));
    handle
}

/// Creates a solid brush with the specified color.
///
/// # Arguments
/// * `color`: The color of the brush, specified as an RGB value in the lower 24 bits (0x00BBGGRR).
///
/// # Safety
/// The caller must ensure that the color value is valid (i.e., does not have bits set outside the lower 24 bits).
/// The returned handle must be deleted with `delete_object` when no longer needed to avoid resource leaks.
/// The function will fail if there are insufficient resources to create the brush.
///
/// # Returns
/// A handle to the solid brush, or 0 if the function fails.
pub unsafe fn create_solid_brush(color: u32) -> usize {
    let mut state = gdi_state().lock().unwrap();
    let handle = alloc_handle();
    let detail = format!(
        r##"{{"handle":{},"color":"#{:06X}"}}"##,
        handle,
        color & 0x00FF_FFFF
    );
    rine_types::dev_notify!(on_handle_created(handle as i64, "GdiBrush", &detail));
    state
        .objects
        .insert(handle, GdiObject::Brush(Brush { color }));
    handle
}

/// Creates a logical pen that has the specified style, width, and color.
///
/// # Arguments
/// * `_style`: The pen style. This parameter can be one of the following values:
///   - `PS_SOLID`: The pen is solid.
///   - `PS_DASH`: The pen is dashed.
///   - `PS_DOT`: The pen is dotted.
///   - `PS_DASHDOT`: The pen is dashed and dotted.
///   - `PS_DASHDOTDOT`: The pen is dashed and double-dotted.
///     Currently, the style parameter is ignored and the created pen is always solid.
/// * `_width`: The width of the pen, in logical units. The pen is always drawn centered on the perimeter of a shape.
///   Therefore, when you draw a line with a pen that has a width of 1, the line is always one pixel wide.
///   When you draw a line with a pen that has a width of 5, the line is 5 pixels wide, with 2 pixels on either side of the theoretical center line.
///   If `width` is zero, the pen is 1 pixel wide.
///   If `width` is greater than 0, the pen is the specified width.
///   If `width` is less than 0, the pen is the absolute value of `width`, but the pen width is always at least 1 pixel.
///   Currently, the width parameter is ignored and the created pen always has a width of 1 pixel.
///
/// * `color`: The color of the pen, specified as an RGB value in the lower 24 bits (0x00BBGGRR).
///
/// # Safety
/// The caller must ensure that `style` is a valid pen style value, and that `color` is a valid RGB color value
/// (i.e., does not have bits set outside the lower 24 bits).
/// The returned handle must be deleted with `delete_object` when no longer needed to avoid resource leaks.
/// The function will fail if there are insufficient resources to create the pen.
/// The caller is responsible for ensuring that the width is not negative to avoid unexpected results.
///
/// # Returns
/// A handle to the logical pen, or 0 if the function fails.
pub unsafe fn create_pen(_style: i32, _width: i32, color: u32) -> usize {
    let mut state = gdi_state().lock().unwrap();
    let handle = alloc_handle();
    let detail = format!(
        r##"{{"handle":{},"style":{},"width":{},"color":"#{:06X}"}}"##,
        handle,
        _style,
        _width,
        color & 0x00FF_FFFF
    );
    rine_types::dev_notify!(on_handle_created(handle as i64, "GdiPen", &detail));
    state.objects.insert(handle, GdiObject::Pen(Pen { color }));
    handle
}

/// Selects an object into the specified device context (DC). The new object replaces the previous object of the same type.
///
/// # Arguments
/// * `hdc`: A handle to the DC into which the object will be selected.
///   This handle must have been returned by a previous call to `create_compatible_dc`.
/// * `object`: A handle to the object to be selected.
///   This can be a bitmap, brush, or pen handle that was returned by a previous call to `create_compatible_bitmap`,
///   `create_solid_brush`, or `create_pen`, respectively.
///
/// # Safety
/// The caller must ensure that `hdc` is a valid device context handle that belongs to this runtime, and that `object`
/// is a valid handle to a GDI object of the appropriate type.
/// The returned handle is the handle to the object being replaced, or 0 if there was no previous object of the same type selected in the DC.
/// If the function fails (e.g., if the handles are invalid), the return value is also 0,
/// so the caller must check for errors before using the return value.
/// The caller is responsible for ensuring that the selected objects are not deleted while they are still selected
/// in any DC (including the one they are selected into) to avoid resource leaks and undefined behavior.
///
/// # Returns
/// The return value is a handle to the object being replaced, or 0 if there was no previous object of the same type selected in the DC.
/// If the function fails (e.g., if the handles are invalid), the return value is also 0.
pub unsafe fn select_object(hdc: usize, object: usize) -> usize {
    let mut state = gdi_state().lock().unwrap();

    let object_kind = match state.objects.get(&object) {
        Some(GdiObject::Bitmap(_)) => 0_u8,
        Some(GdiObject::Brush(brush)) => {
            let _ = brush.color;
            1_u8
        }
        Some(GdiObject::Pen(pen)) => {
            let _ = pen.color;
            2_u8
        }
        None => return 0,
    };

    let Some(dc) = state.dcs.get_mut(&hdc) else {
        return 0;
    };

    match object_kind {
        0 => {
            let old = dc.selected_bitmap.unwrap_or(0);
            dc.selected_bitmap = Some(object);
            old
        }
        1 => {
            let old = dc.selected_brush.unwrap_or(0);
            dc.selected_brush = Some(object);
            old
        }
        _ => {
            let old = dc.selected_pen.unwrap_or(0);
            dc.selected_pen = Some(object);
            old
        }
    }
}

/// Deletes a GDI object.
///
/// # Arguments
/// * `object`: A handle to the GDI object to be deleted.
///   This can be a bitmap, brush, or pen handle that was returned by a previous call to `create_compatible_bitmap`,
///   `create_solid_brush`, or `create_pen`, respectively.
///   This function will fail if the object is currently selected into any device context (DC),
///   including the one it was created with, to prevent resource leaks and undefined behavior.
///
/// # Safety
/// The caller must ensure that `object` is a valid handle to a GDI object that belongs to this runtime.
/// After this call, the handle must not be used, as it has been freed. This function will fail if the object is
/// currently selected into any device context (DC), including the one it was created with, to prevent resource leaks and undefined behavior.
/// The caller is responsible for ensuring that the object is not selected into any DC when this function is called
/// to avoid resource leaks and undefined behavior.
/// The caller is also responsible for ensuring that the object handle is not used after it has been deleted to avoid undefined behavior.
///
/// # Returns
/// The function will return `WinBool::FALSE` if the object is currently selected into any device context (DC), including the one it was
/// created with, to prevent resource leaks and undefined behavior.
pub unsafe fn delete_object(object: usize) -> WinBool {
    let mut state = gdi_state().lock().unwrap();
    if object_selected_by_any_dc(&state, object) {
        return WinBool::FALSE;
    }

    if let Some(gdi_object) = state.objects.remove(&object) {
        if let GdiObject::Bitmap(bitmap) = &gdi_object {
            notify_bitmap_free(bitmap);
        }
        rine_types::dev_notify!(on_handle_closed(object as i64));
        WinBool::TRUE
    } else {
        WinBool::FALSE
    }
}

/// Performs a bit-block transfer of the color data corresponding to a rectangle of pixels from the specified
/// source device context into a destination device context.
///
/// # Arguments
/// * `hdc_dest`: A handle to the destination DC.
/// * `x_dest`: The x-coordinate, in logical units, of the upper-left corner of the destination rectangle.
/// * `y_dest`: The y-coordinate, in logical units, of the upper-left corner of the destination rectangle.
/// * `width`: The width, in logical units, of the source and destination rectangles.
/// * `height`: The height, in logical units, of the source and destination rectangles.
/// * `hdc_src`: A handle to the source DC.
/// * `x_src`: The x-coordinate, in logical units, of the upper-left corner of the source rectangle.
/// * `y_src`: The y-coordinate, in logical units, of the upper-left corner of the source rectangle.
/// * `rop`: A raster-operation code that specifies the combinationn of source and destination colors. This parameter can be one of the following values:
///   - `SRCCOPY`: Copies the source rectangle directly to the destination rectangle.
///   - `SRCPAINT`: Combines the colors of the source and destination rectangles by using the Boolean OR operator.
///   - `SRCAND`: Combines the colors of the source and destination rectangles by using the Boolean AND operator.
///   - `SRCINVERT`: Combines the colors of the source and destination rectangles by using the Boolean XOR operator.
///   - `SRCERASE`: Combines the inverted colors of the source rectangle with the colors of the destination rectangle by using the Boolean AND operator.
///   - `NOTSRCCOPY`: Copies the inverted source rectangle to the destination rectangle.
///   - `NOTSRCERASE`: Combines the colors of the source rectangle with the inverted colors of the destination rectangle by using the Boolean OR operator.
///   - `MERGECOPY`: Combines the colors of the source rectangle with the colors of the destination rectangle by using the Boolean AND operator, and then copies the result to the destination rectangle.
///   - `MERGEPAINT`: Combines the inverted colors of the source rectangle with the colors of the destination rectangle by using the Boolean OR operator, and then copies the result to the destination rectangle.
///   - `PATCOPY`: Copies the brush currently selected in the destination DC to the destination rectangle using the specified raster operation.
///   - `PATPAINT`: Combines the brush currently selected in the destination DC with the colors of the source rectangle by using the Boolean OR operator, and then copies the result to the destination rectangle using the specified raster operation.
///   - `PATINVERT`: Combines the brush currently selected in the destination DC with the colors of the source rectangle by using the Boolean XOR operator, and then copies the result to the destination rectangle using the specified raster operation.
///   - `DSTINVERT`: Inverts the colors of the destination rectangle.
///   - `BLACKNESS`: Fills the destination rectangle with black.
///   - `WHITENESS`: Fills the destination rectangle with white.
///     Currently, only `SRCCOPY` is supported, and the function will ignore the `rop` parameter if it is set to any other value.
///
/// # Safety
/// The caller must ensure that `hdc_dest` and `hdc_src` are valid device context handles that belong to this runtime, and that the specified rectangles are within the bounds of the respective device contexts.
/// The function will fail if the handles are invalid, if the rectangles are out of bounds, or if there are insufficient resources to perform the operation.
/// The caller is responsible for ensuring that the source and destination rectangles are properly defined to avoid unexpected results.
/// The caller is also responsible for ensuring that the raster operation code is valid to avoid unexpected results.
/// The caller must also ensure that the device contexts are not currently in an error state to avoid unexpected results.
/// The caller is responsible for checking the return value to determine if the operation succeeded or failed.
///
/// # Returns
/// The function returns `WinBool::TRUE` if the operation succeeded, or `WinBool::FALSE` if it failed.
pub unsafe fn bit_blt(
    hdc_dest: usize,
    dest_rect: Rect,
    hdc_src: usize,
    src_rect: Rect,
    rop: u32,
) -> WinBool {
    let width = dest_rect.right.saturating_sub(dest_rect.left);
    let height = dest_rect.bottom.saturating_sub(dest_rect.top);
    let src_width = src_rect.right.saturating_sub(src_rect.left);
    let src_height = src_rect.bottom.saturating_sub(src_rect.top);

    if width <= 0 || height <= 0 || rop != SRCCOPY {
        return WinBool::FALSE;
    }

    if src_width != width || src_height != height {
        return WinBool::FALSE;
    }

    let mut state = gdi_state().lock().unwrap();

    let src_bitmap = match with_selected_bitmap_mut(&mut state, hdc_src, |bmp| bmp.clone()) {
        Some(bitmap) => bitmap,
        None => return WinBool::FALSE,
    };

    let Some(result) = with_selected_bitmap_mut(&mut state, hdc_dest, |dest| {
        for dy in 0..height {
            for dx in 0..width {
                let src_x = src_rect.left + dx;
                let src_y = src_rect.top + dy;
                let dest_x = dest_rect.left + dx;
                let dest_y = dest_rect.top + dy;

                let Some(src_idx) = src_bitmap.index(src_x, src_y) else {
                    continue;
                };
                let Some(dest_idx) = dest.index(dest_x, dest_y) else {
                    continue;
                };

                dest.pixels[dest_idx] = src_bitmap.pixels[src_idx];
            }
        }

        WinBool::TRUE
    }) else {
        return WinBool::FALSE;
    };

    result
}

/// Draws the specified text string at the given coordinates in the device context identified by `hdc`.
/// # Arguments
/// * `hdc`: A handle to the device context in which to draw the text. This DC must have a bitmap selected into it.
/// * `x`: The x-coordinate of the reference point for the text. The interpretation of this coordinate depends on the current mapping mode of the DC.
/// * `y`: The y-coordinate of the reference point for the text. The interpretation of this coordinate depends on the current mapping mode of the DC.
/// * `text`: The text string to be drawn.
///
/// # Safety
/// The caller must ensure that `hdc` is a valid device context handle that belongs to this runtime and has a bitmap selected into it.
/// The caller is responsible for ensuring that the coordinates and text are appropriate for the current mapping mode and selected bitmap
/// to avoid unexpected results.
///
/// # Returns
/// Returns `WinBool::TRUE` if the text was successfully drawn,
/// or `WinBool::FALSE` if the `hdc` was invalid or if no bitmap was selected into the DC.
pub fn text_out(hdc: usize, x: i32, y: i32, text: &str) -> WinBool {
    let mut state = gdi_state().lock().unwrap();
    if with_selected_bitmap_mut(&mut state, hdc, |bitmap| draw_text(bitmap, x, y, text)).is_none() {
        return WinBool::FALSE;
    }

    WinBool::TRUE
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::GdiObject;
    use crate::ops::text_out;
    use crate::state::gdi_state;

    #[test]
    fn bitblt_copies_surface_pixels() {
        unsafe {
            let src_dc = create_compatible_dc(0);
            let dst_dc = create_compatible_dc(0);

            let src_bitmap = create_compatible_bitmap(src_dc, 32, 32);
            let dst_bitmap = create_compatible_bitmap(dst_dc, 32, 32);
            assert_ne!(src_bitmap, 0);
            assert_ne!(dst_bitmap, 0);

            assert_ne!(select_object(src_dc, src_bitmap), 0);
            assert_ne!(select_object(dst_dc, dst_bitmap), 0);

            let hello = "Hello";
            assert_eq!(text_out(src_dc, 0, 0, hello), WinBool::TRUE);
            assert_eq!(
                bit_blt(
                    dst_dc,
                    Rect {
                        left: 0,
                        top: 0,
                        right: 32,
                        bottom: 32,
                    },
                    src_dc,
                    Rect {
                        left: 0,
                        top: 0,
                        right: 32,
                        bottom: 32,
                    },
                    SRCCOPY,
                ),
                WinBool::TRUE
            );

            let state = gdi_state().lock().unwrap();
            let src_pixels = match state.objects.get(&src_bitmap) {
                Some(GdiObject::Bitmap(bitmap)) => bitmap.pixels.clone(),
                _ => panic!("source bitmap missing"),
            };
            let dst_pixels = match state.objects.get(&dst_bitmap) {
                Some(GdiObject::Bitmap(bitmap)) => bitmap.pixels.clone(),
                _ => panic!("dest bitmap missing"),
            };
            drop(state);

            assert_eq!(src_pixels, dst_pixels);
            assert_eq!(delete_dc(src_dc), WinBool::TRUE);
            assert_eq!(delete_dc(dst_dc), WinBool::TRUE);
        }
    }

    #[test]
    fn delete_object_fails_while_selected() {
        unsafe {
            let dc = create_compatible_dc(0);
            let bitmap = create_compatible_bitmap(dc, 4, 4);
            assert_ne!(select_object(dc, bitmap), 0);
            assert_eq!(delete_object(bitmap), WinBool::FALSE);
            assert_eq!(delete_dc(dc), WinBool::TRUE);
            assert_eq!(delete_object(bitmap), WinBool::TRUE);
        }
    }
}
