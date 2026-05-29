use super::*;

#[derive(Debug, Clone)]
pub struct LogicalFontInfo {
    pub height: i32,
    pub width: i32,
    pub weight: i32,
    pub italic: bool,
    pub face_name: Option<String>,
}

pub fn get_logical_font_info(font_handle: usize) -> Option<LogicalFontInfo> {
    let state = gdi_state().lock().unwrap();
    match state.objects.get(&font_handle) {
        Some(GdiObject::Font(font)) => Some(LogicalFontInfo {
            height: font.height,
            width: font.width,
            weight: font.weight,
            italic: font.italic != 0,
            face_name: font.face_name.clone(),
        }),
        _ => None,
    }
}

/// Retrieves a handle to one of the predefined stock pens or brushes.
///
/// # Arguments
/// * `object`: The stock object index. This implementation supports brush/pen indices:
///   `WHITE_BRUSH`, `LTGRAY_BRUSH`, `GRAY_BRUSH`, `DKGRAY_BRUSH`, `BLACK_BRUSH`,
///   `NULL_BRUSH`/`HOLLOW_BRUSH`, `WHITE_PEN`, `BLACK_PEN`, `NULL_PEN`, `DC_BRUSH`, and `DC_PEN`.
///
/// # Safety
/// The caller must pass a valid stock object index expected by the Win32 API contract.
/// Returned handles are owned by the runtime and must not be freed with `delete_object`.
///
/// # Returns
/// Returns a stable handle for the requested stock object when supported, or 0 for unsupported indices.
///
/// # Notes
/// This is a partial implementation. Font, palette, and other stock-object indices are not yet supported.
/// `NULL_PEN`/`NULL_BRUSH` and `DC_PEN`/`DC_BRUSH` currently map to basic placeholder pen/brush objects;
/// null-object drawing semantics and dynamic DC pen/brush color semantics are not fully implemented yet.
pub unsafe fn get_stock_object(object: i32) -> usize {
    let mut state = gdi_state().lock().unwrap();

    if let Some(existing) = state.stock_objects.get(&object) {
        return *existing;
    }

    let Some(gdi_object) = stock_object_for_index(object) else {
        return 0;
    };

    let handle = alloc_handle();
    state.objects.insert(handle, gdi_object);
    state.stock_objects.insert(object, handle);

    let detail = format!(r#"{{"handle":{},"stock_index":{}}}"#, handle, object);
    rine_types::dev_notify!(on_handle_created(handle as i64, "GdiStockObject", &detail));

    handle
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
/// Returns `BOOL::TRUE` if the DC was successfully deleted,
/// or `BOOL::FALSE` if the handle was invalid or if any selected objects are still in use.
pub unsafe fn delete_dc(hdc: usize) -> BOOL {
    let mut state = gdi_state().lock().unwrap();
    let Some(dc) = state.dcs.remove(&hdc) else {
        return BOOL::FALSE;
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
    BOOL::TRUE
}

/// Sets the background mix mode for text, hatched brushes, and pen styles in the specified device context.
///
/// # Arguments
/// * `hdc`: A handle to the device context whose background mode will be updated.
/// * `mode`: The requested background mode. Supported values are `TRANSPARENT` and `OPAQUE`.
///
/// # Safety
/// The caller must ensure that `hdc` is a valid device context handle that belongs to this runtime.
/// Passing an invalid handle or unsupported `mode` value causes this function to fail and return 0.
///
/// # Returns
/// Returns the previous background mode on success, or 0 if `hdc` is invalid or `mode` is unsupported.
pub unsafe fn set_bk_mode(hdc: usize, mode: i32) -> i32 {
    if !is_valid_bk_mode(mode) {
        return 0;
    }

    let mut state = gdi_state().lock().unwrap();
    let Some(dc) = state.dcs.get_mut(&hdc) else {
        return 0;
    };

    let previous = dc.bk_mode;
    dc.bk_mode = mode;
    previous
}

/// Sets the mapping mode for the specified device context.
///
/// # Arguments
/// * `hdc`: A handle to the device context whose mapping mode will be updated.
/// * `mode`: The requested mapping mode. Supported values are `MM_TEXT`, `MM_LOMETRIC`, `MM_HIMETRIC`,
///   `MM_LOENGLISH`, `MM_HIENGLISH`, `MM_TWIPS`, `MM_ISOTROPIC`, and `MM_ANISOTROPIC`.
///
/// # Safety
/// The caller must ensure that `hdc` is a valid device context handle that belongs to this runtime.
/// Passing an invalid handle or unsupported `mode` value causes this function to fail and return 0.
///
/// # Returns
/// Returns the previous mapping mode on success, or 0 if `hdc` is invalid or `mode` is unsupported.
pub unsafe fn set_map_mode(hdc: usize, mode: i32) -> i32 {
    if !is_valid_map_mode(mode) {
        return 0;
    }

    let mut state = gdi_state().lock().unwrap();
    let Some(dc) = state.dcs.get_mut(&hdc) else {
        return 0;
    };

    let previous = dc.map_mode;
    dc.map_mode = mode;
    previous
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

fn sanitize_face_name(face_name: Option<String>) -> Option<String> {
    face_name.and_then(|name| {
        let trimmed = name.trim_matches(char::from(0)).trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

#[allow(clippy::too_many_arguments)]
unsafe fn create_font_internal(
    height: i32,
    width: i32,
    escapement: i32,
    orientation: i32,
    weight: i32,
    italic: u32,
    underline: u32,
    strike_out: u32,
    char_set: u32,
    out_precision: u32,
    clip_precision: u32,
    quality: u32,
    pitch_and_family: u32,
    face_name: Option<String>,
) -> usize {
    let mut state = gdi_state().lock().unwrap();
    let handle = alloc_handle();
    let face_name = sanitize_face_name(face_name);

    let detail = format!(
        r#"{{"handle":{},"height":{},"width":{},"weight":{},"italic":{},"underline":{},"strike_out":{},"char_set":{},"quality":{},"face_name":{:?}}}"#,
        handle, height, width, weight, italic, underline, strike_out, char_set, quality, face_name
    );
    rine_types::dev_notify!(on_handle_created(handle as i64, "GdiFont", &detail));

    state.objects.insert(
        handle,
        GdiObject::Font(Font {
            height,
            width,
            escapement,
            orientation,
            weight,
            italic,
            underline,
            strike_out,
            char_set,
            out_precision,
            clip_precision,
            quality,
            pitch_and_family,
            face_name,
        }),
    );

    handle
}

/// Creates a logical font from ANSI input parameters.
///
/// # Arguments
/// * `height` - Requested font height, in logical units.
/// * `width` - Requested average character width, in logical units.
/// * `escapement` - Angle between the escapement vector and the x-axis, in tenths of a degree.
/// * `orientation` - Angle between each character's baseline and the x-axis, in tenths of a degree.
/// * `weight` - Requested font weight from 0 through 1000.
/// * `italic` - Nonzero to request an italic font.
/// * `underline` - Nonzero to request an underlined font.
/// * `strike_out` - Nonzero to request a strikeout font.
/// * `char_set` - Character-set identifier.
/// * `out_precision` - Output-precision hint.
/// * `clip_precision` - Clipping-precision hint.
/// * `quality` - Output quality hint.
/// * `pitch_and_family` - Pitch and family flags.
/// * `face_name` - Optional ANSI face-name pointer. Null means no preferred face name.
///
/// # Safety
/// The caller must ensure that `face_name` is null or points to a valid null-terminated ANSI string.
/// The returned handle must be released with `delete_object` when no longer needed.
///
/// # Returns
/// Returns a handle to a logical font object, or 0 if creation fails.
///
/// # Notes
/// This is a partial implementation. Font matching and glyph rasterization are not implemented yet;
/// stored font attributes are currently metadata for API compatibility and future rendering work.
#[allow(clippy::too_many_arguments)]
pub unsafe fn create_font_a(
    height: i32,
    width: i32,
    escapement: i32,
    orientation: i32,
    weight: i32,
    italic: u32,
    underline: u32,
    strike_out: u32,
    char_set: u32,
    out_precision: u32,
    clip_precision: u32,
    quality: u32,
    pitch_and_family: u32,
    face_name: LPCSTR,
) -> usize {
    let face_name = unsafe { face_name.read_string() };
    unsafe {
        create_font_internal(
            height,
            width,
            escapement,
            orientation,
            weight,
            italic,
            underline,
            strike_out,
            char_set,
            out_precision,
            clip_precision,
            quality,
            pitch_and_family,
            face_name,
        )
    }
}

/// Creates a logical font from UTF-16 input parameters.
///
/// # Arguments
/// * `height` - Requested font height, in logical units.
/// * `width` - Requested average character width, in logical units.
/// * `escapement` - Angle between the escapement vector and the x-axis, in tenths of a degree.
/// * `orientation` - Angle between each character's baseline and the x-axis, in tenths of a degree.
/// * `weight` - Requested font weight from 0 through 1000.
/// * `italic` - Nonzero to request an italic font.
/// * `underline` - Nonzero to request an underlined font.
/// * `strike_out` - Nonzero to request a strikeout font.
/// * `char_set` - Character-set identifier.
/// * `out_precision` - Output-precision hint.
/// * `clip_precision` - Clipping-precision hint.
/// * `quality` - Output quality hint.
/// * `pitch_and_family` - Pitch and family flags.
/// * `face_name` - Optional UTF-16 face-name pointer. Null means no preferred face name.
///
/// # Safety
/// The caller must ensure that `face_name` is null or points to a valid null-terminated UTF-16 string.
/// The returned handle must be released with `delete_object` when no longer needed.
///
/// # Returns
/// Returns a handle to a logical font object, or 0 if creation fails.
///
/// # Notes
/// This is a partial implementation. Font matching and glyph rasterization are not implemented yet;
/// stored font attributes are currently metadata for API compatibility and future rendering work.
#[allow(clippy::too_many_arguments)]
pub unsafe fn create_font_w(
    height: i32,
    width: i32,
    escapement: i32,
    orientation: i32,
    weight: i32,
    italic: u32,
    underline: u32,
    strike_out: u32,
    char_set: u32,
    out_precision: u32,
    clip_precision: u32,
    quality: u32,
    pitch_and_family: u32,
    face_name: LPCWSTR,
) -> usize {
    let face_name = unsafe { face_name.read_string() };
    unsafe {
        create_font_internal(
            height,
            width,
            escapement,
            orientation,
            weight,
            italic,
            underline,
            strike_out,
            char_set,
            out_precision,
            clip_precision,
            quality,
            pitch_and_family,
            face_name,
        )
    }
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

    enum SelectedObjectKind {
        Bitmap,
        Brush,
        Pen,
        Font,
    }

    let object_kind = match state.objects.get(&object) {
        Some(GdiObject::Bitmap(_)) => SelectedObjectKind::Bitmap,
        Some(GdiObject::Brush(_)) => SelectedObjectKind::Brush,
        Some(GdiObject::Pen(_)) => SelectedObjectKind::Pen,
        Some(GdiObject::Font(_)) => SelectedObjectKind::Font,
        None => return 0,
    };

    let Some(dc) = state.dcs.get_mut(&hdc) else {
        return 0;
    };

    match object_kind {
        SelectedObjectKind::Bitmap => dc.selected_bitmap.replace(object).unwrap_or(0),
        SelectedObjectKind::Brush => dc.selected_brush.replace(object).unwrap_or(0),
        SelectedObjectKind::Pen => dc.selected_pen.replace(object).unwrap_or(0),
        SelectedObjectKind::Font => dc.selected_font.replace(object).unwrap_or(0),
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
/// The function will return `BOOL::FALSE` if the object is currently selected into any device context (DC), including the one it was
/// created with, to prevent resource leaks and undefined behavior.
pub unsafe fn delete_object(object: usize) -> BOOL {
    let mut state = gdi_state().lock().unwrap();
    if state
        .stock_objects
        .values()
        .any(|stock_handle| *stock_handle == object)
    {
        return BOOL::FALSE;
    }

    if object_selected_by_any_dc(&state, object) {
        return BOOL::FALSE;
    }

    if let Some(gdi_object) = state.objects.remove(&object) {
        if let GdiObject::Bitmap(bitmap) = &gdi_object {
            notify_bitmap_free(bitmap);
        }
        rine_types::dev_notify!(on_handle_closed(object as i64));
        BOOL::TRUE
    } else {
        BOOL::FALSE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::GdiObject;
    use crate::state::gdi_state;
    use std::ffi::CString;

    fn lpcstr_from_ptr(ptr: *const u8) -> LPCSTR {
        unsafe { std::mem::transmute(ptr) }
    }

    fn lpcwstr_from_ptr(ptr: *const u16) -> LPCWSTR {
        unsafe { std::mem::transmute(ptr) }
    }

    #[test]
    fn delete_object_fails_while_selected() {
        unsafe {
            let dc = create_compatible_dc(0);
            let bitmap = create_compatible_bitmap(dc, 4, 4);
            assert_ne!(select_object(dc, bitmap), 0);
            assert_eq!(delete_object(bitmap), BOOL::FALSE);
            assert_eq!(delete_dc(dc), BOOL::TRUE);
            assert_eq!(delete_object(bitmap), BOOL::TRUE);
        }
    }

    #[test]
    fn set_bk_mode_updates_state_and_returns_previous_mode() {
        unsafe {
            let dc = create_compatible_dc(0);
            assert_eq!(set_bk_mode(dc, TRANSPARENT), OPAQUE);
            assert_eq!(set_bk_mode(dc, OPAQUE), TRANSPARENT);
            assert_eq!(delete_dc(dc), BOOL::TRUE);
        }
    }

    #[test]
    fn set_bk_mode_rejects_invalid_mode_and_invalid_dc() {
        unsafe {
            let dc = create_compatible_dc(0);
            assert_eq!(set_bk_mode(dc, 99), 0);
            assert_eq!(set_bk_mode(0xDEADBEEF, OPAQUE), 0);
            assert_eq!(delete_dc(dc), BOOL::TRUE);
        }
    }

    #[test]
    fn set_map_mode_updates_state_and_returns_previous_mode() {
        unsafe {
            let dc = create_compatible_dc(0);
            assert_eq!(set_map_mode(dc, MM_ISOTROPIC), MM_TEXT);
            assert_eq!(set_map_mode(dc, MM_ANISOTROPIC), MM_ISOTROPIC);
            assert_eq!(delete_dc(dc), BOOL::TRUE);
        }
    }

    #[test]
    fn set_map_mode_rejects_invalid_mode_and_invalid_dc() {
        unsafe {
            let dc = create_compatible_dc(0);
            assert_eq!(set_map_mode(dc, 0), 0);
            assert_eq!(set_map_mode(0xDEADBEEF, MM_TEXT), 0);
            assert_eq!(delete_dc(dc), BOOL::TRUE);
        }
    }

    #[test]
    fn get_stock_object_returns_stable_handle_for_supported_index() {
        unsafe {
            let first = get_stock_object(WHITE_BRUSH);
            let second = get_stock_object(WHITE_BRUSH);
            assert_ne!(first, 0);
            assert_eq!(first, second);
        }
    }

    #[test]
    fn get_stock_object_returns_zero_for_unsupported_index() {
        unsafe {
            assert_eq!(get_stock_object(10), 0);
        }
    }

    #[test]
    fn delete_object_rejects_stock_object_handles() {
        unsafe {
            let stock = get_stock_object(BLACK_PEN);
            assert_ne!(stock, 0);
            assert_eq!(delete_object(stock), BOOL::FALSE);
        }
    }

    #[test]
    fn create_font_a_creates_font_object_with_face_name() {
        unsafe {
            let face = CString::new("Arial").unwrap();
            let font = create_font_a(
                18,
                0,
                0,
                0,
                400,
                0,
                0,
                0,
                1,
                0,
                0,
                0,
                0,
                lpcstr_from_ptr(face.as_ptr() as *const u8),
            );
            assert_ne!(font, 0);

            let state = gdi_state().lock().unwrap();
            match state.objects.get(&font) {
                Some(GdiObject::Font(f)) => {
                    assert_eq!(f.height, 18);
                    assert_eq!(f.weight, 400);
                    assert_eq!(f.face_name.as_deref(), Some("Arial"));
                }
                _ => panic!("font object missing"),
            }
            drop(state);

            assert_eq!(delete_object(font), BOOL::TRUE);
        }
    }

    #[test]
    fn create_font_w_allows_select_and_delete_lifecycle() {
        unsafe {
            let dc = create_compatible_dc(0);

            let face1: Vec<u16> = "Consolas".encode_utf16().chain([0]).collect();
            let face2: Vec<u16> = "Tahoma".encode_utf16().chain([0]).collect();

            let font_a = create_font_w(
                16,
                0,
                0,
                0,
                500,
                1,
                0,
                0,
                1,
                0,
                0,
                0,
                0,
                lpcwstr_from_ptr(face1.as_ptr()),
            );
            let font_b = create_font_w(
                14,
                0,
                0,
                0,
                300,
                0,
                0,
                0,
                1,
                0,
                0,
                0,
                0,
                lpcwstr_from_ptr(face2.as_ptr()),
            );
            assert_ne!(font_a, 0);
            assert_ne!(font_b, 0);

            assert_eq!(select_object(dc, font_a), 0);
            assert_eq!(delete_object(font_a), BOOL::FALSE);
            assert_eq!(select_object(dc, font_b), font_a);
            assert_eq!(delete_object(font_a), BOOL::TRUE);

            assert_eq!(delete_dc(dc), BOOL::TRUE);
            assert_eq!(delete_object(font_b), BOOL::TRUE);
        }
    }
}
