use super::*;

pub const SRCCOPY: u32 = 0x00CC0020;
pub const TRANSPARENT: i32 = 1;
pub const OPAQUE: i32 = 2;

pub const MM_TEXT: i32 = 1;
pub const MM_LOMETRIC: i32 = 2;
pub const MM_HIMETRIC: i32 = 3;
pub const MM_LOENGLISH: i32 = 4;
pub const MM_HIENGLISH: i32 = 5;
pub const MM_TWIPS: i32 = 6;
pub const MM_ISOTROPIC: i32 = 7;
pub const MM_ANISOTROPIC: i32 = 8;

pub const WHITE_BRUSH: i32 = 0;
pub const LTGRAY_BRUSH: i32 = 1;
pub const GRAY_BRUSH: i32 = 2;
pub const DKGRAY_BRUSH: i32 = 3;
pub const BLACK_BRUSH: i32 = 4;
pub const NULL_BRUSH: i32 = 5;
pub const HOLLOW_BRUSH: i32 = NULL_BRUSH;
pub const WHITE_PEN: i32 = 6;
pub const BLACK_PEN: i32 = 7;
pub const NULL_PEN: i32 = 8;
pub const DC_BRUSH: i32 = 18;
pub const DC_PEN: i32 = 19;

pub(crate) fn bitmap_bytes(bitmap: &Bitmap) -> u64 {
    (bitmap.pixels.len() * std::mem::size_of::<u32>()) as u64
}

pub(super) fn is_valid_bk_mode(mode: i32) -> bool {
    mode == TRANSPARENT || mode == OPAQUE
}

pub(super) fn is_valid_map_mode(mode: i32) -> bool {
    matches!(
        mode,
        MM_TEXT
            | MM_LOMETRIC
            | MM_HIMETRIC
            | MM_LOENGLISH
            | MM_HIENGLISH
            | MM_TWIPS
            | MM_ISOTROPIC
            | MM_ANISOTROPIC
    )
}

pub(super) fn stock_object_for_index(object: i32) -> Option<GdiObject> {
    match object {
        WHITE_BRUSH => Some(GdiObject::Brush(Brush { color: 0x00FF_FFFF })),
        LTGRAY_BRUSH => Some(GdiObject::Brush(Brush { color: 0x00C0_C0C0 })),
        GRAY_BRUSH => Some(GdiObject::Brush(Brush { color: 0x0080_8080 })),
        DKGRAY_BRUSH => Some(GdiObject::Brush(Brush { color: 0x0040_4040 })),
        BLACK_BRUSH => Some(GdiObject::Brush(Brush { color: 0x0000_0000 })),
        NULL_BRUSH => Some(GdiObject::Brush(Brush { color: 0x0000_0000 })),
        WHITE_PEN => Some(GdiObject::Pen(Pen { color: 0x00FF_FFFF })),
        BLACK_PEN => Some(GdiObject::Pen(Pen { color: 0x0000_0000 })),
        NULL_PEN => Some(GdiObject::Pen(Pen { color: 0x0000_0000 })),
        // Current renderer does not support dynamic DC pen/brush colors yet.
        DC_BRUSH => Some(GdiObject::Brush(Brush { color: 0x0000_0000 })),
        DC_PEN => Some(GdiObject::Pen(Pen { color: 0x0000_0000 })),
        _ => None,
    }
}
