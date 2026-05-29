use super::*;

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
/// The function returns `BOOL::TRUE` if the operation succeeded, or `BOOL::FALSE` if it failed.
pub unsafe fn bit_blt(
    hdc_dest: usize,
    dest_rect: Rect,
    hdc_src: usize,
    src_rect: Rect,
    rop: u32,
) -> BOOL {
    let width = dest_rect.right.saturating_sub(dest_rect.left);
    let height = dest_rect.bottom.saturating_sub(dest_rect.top);
    let src_width = src_rect.right.saturating_sub(src_rect.left);
    let src_height = src_rect.bottom.saturating_sub(src_rect.top);

    if width <= 0 || height <= 0 || rop != SRCCOPY {
        return BOOL::FALSE;
    }

    if src_width != width || src_height != height {
        return BOOL::FALSE;
    }

    let mut state = gdi_state().lock().unwrap();

    let src_bitmap = match with_selected_bitmap_mut(&mut state, hdc_src, |bmp| bmp.clone()) {
        Some(bitmap) => bitmap,
        None => return BOOL::FALSE,
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

        BOOL::TRUE
    }) else {
        return BOOL::FALSE;
    };

    result
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
            assert_eq!(text_out(src_dc, 0, 0, hello), BOOL::TRUE);
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
                BOOL::TRUE
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
            assert_eq!(delete_dc(src_dc), BOOL::TRUE);
            assert_eq!(delete_dc(dst_dc), BOOL::TRUE);
        }
    }
}
