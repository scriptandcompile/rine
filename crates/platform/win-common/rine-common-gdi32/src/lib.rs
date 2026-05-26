#![allow(unsafe_op_in_unsafe_fn)]

pub mod objects;
pub mod ops;
pub mod state;
pub mod telemetry;
pub mod text;

pub use ops::{
    MM_ANISOTROPIC, MM_HIENGLISH, MM_HIMETRIC, MM_ISOTROPIC, MM_LOENGLISH, MM_LOMETRIC,
    MM_TEXT, MM_TWIPS, OPAQUE, SRCCOPY, TRANSPARENT, bit_blt, create_compatible_bitmap,
    create_compatible_dc, create_pen, create_solid_brush, delete_dc, delete_object,
    select_object, set_bk_mode, set_map_mode, text_out,
};
