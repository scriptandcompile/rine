use super::*;

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
/// Returns `BOOL::TRUE` if the text was successfully drawn,
/// or `BOOL::FALSE` if the `hdc` was invalid or if no bitmap was selected into the DC.
pub fn text_out(hdc: usize, x: i32, y: i32, text: &str) -> BOOL {
    let mut state = gdi_state().lock().unwrap();
    if with_selected_bitmap_mut(&mut state, hdc, |bitmap| draw_text(bitmap, x, y, text)).is_none() {
        return BOOL::FALSE;
    }

    BOOL::TRUE
}

/// Computes the text extent for an ANSI string using the currently selected pseudo-font metrics.
///
/// # Arguments
/// * `hdc`: A handle to the device context used for text measurement.
/// * `text`: The ANSI text to measure.
/// * `size`: Pointer to a `Size` structure that receives the width (`cx`) and height (`cy`).
///
/// # Safety
/// The caller must ensure that `size` points to writable memory for a valid `Size` structure.
/// The caller must ensure that `hdc` is a valid device context handle created by this runtime.
///
/// # Returns
/// Returns `BOOL::TRUE` if measurement succeeds and `size` is populated,
/// or `BOOL::FALSE` if `hdc` is invalid or `size` is null.
///
/// # Notes
/// This is a partial implementation. It uses the software renderer's fixed pseudo-glyph metrics:
/// each character is measured as `PSEUDO_GLYPH_WIDTH` pixels wide and `PSEUDO_GLYPH_HEIGHT` pixels tall.
/// It does not yet account for selected font faces, kerning, code page specifics, or map mode scaling.
pub unsafe fn get_text_extent_point_a(hdc: usize, text: &str, size: *mut Size) -> BOOL {
    if size.is_null() {
        return BOOL::FALSE;
    }

    let state = gdi_state().lock().unwrap();
    if !state.dcs.contains_key(&hdc) {
        return BOOL::FALSE;
    }
    drop(state);

    let glyph_count = text.chars().count() as i32;

    unsafe {
        (*size).cx = glyph_count.saturating_mul(PSEUDO_GLYPH_WIDTH);
        (*size).cy = PSEUDO_GLYPH_HEIGHT;
    }

    BOOL::TRUE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_text_extent_point_a_reports_expected_dimensions() {
        unsafe {
            let dc = create_compatible_dc(0);
            let mut size = Size::default();

            let result = get_text_extent_point_a(dc, "Hello", &mut size as *mut Size);

            assert_eq!(result, BOOL::TRUE);
            assert_eq!(size.cx, 5 * PSEUDO_GLYPH_WIDTH);
            assert_eq!(size.cy, PSEUDO_GLYPH_HEIGHT);
            assert_eq!(delete_dc(dc), BOOL::TRUE);
        }
    }

    #[test]
    fn get_text_extent_point_a_rejects_invalid_inputs() {
        unsafe {
            let dc = create_compatible_dc(0);
            let mut size = Size::default();

            assert_eq!(
                get_text_extent_point_a(0xDEADBEEF, "abc", &mut size as *mut Size),
                BOOL::FALSE
            );
            assert_eq!(
                get_text_extent_point_a(dc, "abc", core::ptr::null_mut()),
                BOOL::FALSE
            );
            assert_eq!(delete_dc(dc), BOOL::TRUE);
        }
    }
}
