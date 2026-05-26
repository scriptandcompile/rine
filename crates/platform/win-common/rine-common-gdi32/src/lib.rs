#![allow(unsafe_op_in_unsafe_fn)]

pub mod objects;
pub mod ops;
pub mod state;
pub mod telemetry;
pub mod text;

pub use ops::{
    BLACK_BRUSH, BLACK_PEN, DC_BRUSH, DC_PEN, DKGRAY_BRUSH, GRAY_BRUSH, HOLLOW_BRUSH, LTGRAY_BRUSH,
    MM_ANISOTROPIC, MM_HIENGLISH, MM_HIMETRIC, MM_ISOTROPIC, MM_LOENGLISH, MM_LOMETRIC, MM_TEXT,
    MM_TWIPS, NULL_BRUSH, NULL_PEN, OPAQUE, SRCCOPY, TRANSPARENT, WHITE_BRUSH, WHITE_PEN, bit_blt,
    create_compatible_bitmap, create_compatible_dc, create_pen, create_solid_brush, delete_dc,
    delete_object, get_stock_object, select_object, set_bk_mode, set_map_mode, text_out,
};
