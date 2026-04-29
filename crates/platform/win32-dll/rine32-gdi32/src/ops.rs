use rine_common_gdi32 as common;
use rine_types::errors::WinBool;
use rine_types::strings::{LPCSTR, LPCWSTR};
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
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn CreateCompatibleDC(_hdc: usize) -> usize {
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
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn DeleteDC(hdc: usize) -> WinBool {
    unsafe { common::delete_dc(hdc) }
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
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn CreateCompatibleBitmap(
    _hdc: usize,
    width: i32,
    height: i32,
) -> usize {
    unsafe { common::ops::create_compatible_bitmap(_hdc, width, height) }
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
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn CreateSolidBrush(color: u32) -> usize {
    unsafe { common::create_solid_brush(color) }
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
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn CreatePen(style: i32, width: i32, color: u32) -> usize {
    unsafe { common::create_pen(style, width, color) }
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
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn SelectObject(hdc: usize, object: usize) -> usize {
    unsafe { common::select_object(hdc, object) }
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
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn DeleteObject(object: usize) -> WinBool {
    unsafe { common::delete_object(object) }
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
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn BitBlt(
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

/// Writes a character string at the specified location, using the currently selected font, text color, and background color.
///
/// # Arguments
/// * `hdc`: A handle to the device context.
/// * `x`: The x-coordinate of the reference point that the system uses to position the text.
///   The reference point is the upper-left corner of the first character.
/// * `y`: The y-coordinate of the reference point that the system uses to position the text.
///   The reference point is the upper-left corner of the first character.
/// * `text`: A pointer to a buffer that contains the ANSI string to be drawn.
///   The string is not null-terminated; the `count` parameter specifies the number of characters to draw.
/// * `count`: The number of characters in the string pointed to by `text`.
///
/// # Safety
/// The caller must ensure that `hdc` is a valid device context handle that belongs to this runtime,
/// and that `text` points to a valid buffer of at least `count` characters.
/// The function will fail if the buffer is invalid or if the device context does not have a bitmap selected into it.
///
/// # Returns
/// Returns `WinBool::TRUE` if the function succeeds, or `WinBool::FALSE` if it fails.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn TextOutA(
    hdc: usize,
    x: i32,
    y: i32,
    text: LPCSTR,
    count: i32,
) -> WinBool {
    unsafe {
        let Some(text) = text.read_string_counted(count) else {
            return WinBool::FALSE;
        };

        common::ops::text_out(hdc, x, y, &text)
    }
}

/// Writes a character string at the specified location, using the currently selected font, text color, and background color.
///
/// # Arguments
/// * `hdc`: A handle to the device context.
/// * `x`: The x-coordinate of the reference point that the system uses to position the text.
///   The reference point is the upper-left corner of the first character.
/// * `y`: The y-coordinate of the reference point that the system uses to position the text.
///   The reference point is the upper-left corner of the first character.
/// * `text`: A pointer to a buffer that contains the UTF-16LE string to be drawn.
///   The string is not null-terminated; the `count` parameter specifies the number of characters to draw.
/// * `count`: The number of characters in the string pointed to by `text`.
///
/// # Safety
/// The caller must ensure that `hdc` is a valid device context handle that belongs to this runtime,
/// and that `text` points to a valid buffer of at least `count` characters.
/// The function will fail if the buffer is invalid or if the device context does not have a bitmap selected into it.
///
/// # Returns
/// Returns `WinBool::TRUE` if the function succeeds, or `WinBool::FALSE` if it fails.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn TextOutW(
    hdc: usize,
    x: i32,
    y: i32,
    text: LPCWSTR,
    count: i32,
) -> WinBool {
    unsafe {
        let Some(text) = text.read_string_counted(count) else {
            return WinBool::FALSE;
        };

        common::ops::text_out(hdc, x, y, &text)
    }
}
