#![allow(unsafe_op_in_unsafe_fn)]

pub mod objects;
pub mod ops;
pub mod state;
pub mod telemetry;
pub mod text;

pub use ops::{
    SRCCOPY, bit_blt, create_compatible_bitmap, create_compatible_dc, create_pen,
    create_solid_brush, delete_dc, delete_object, select_object, text_out,
};
