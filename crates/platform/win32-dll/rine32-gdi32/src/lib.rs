#![allow(unsafe_op_in_unsafe_fn)]

#[cfg(not(target_pointer_width = "32"))]
compile_error!(
    "crate `rine32-gdi32` must be built for a 32-bit target (for example: --target i686-unknown-linux-gnu)"
);

use rine_dlls::{DllPlugin, Export, as_win_api};

mod ops;

pub struct Gdi32Plugin32;

impl DllPlugin for Gdi32Plugin32 {
    fn dll_names(&self) -> &[&str] {
        &["gdi32.dll"]
    }

    fn exports(&self) -> Vec<Export> {
        vec![
            Export::Func("CreateCompatibleDC", as_win_api!(ops::create_compatible_dc)),
            Export::Func("DeleteDC", as_win_api!(ops::delete_dc)),
            Export::Func(
                "CreateCompatibleBitmap",
                as_win_api!(ops::create_compatible_bitmap),
            ),
            Export::Func("CreateSolidBrush", as_win_api!(ops::create_solid_brush)),
            Export::Func("CreatePen", as_win_api!(ops::create_pen)),
            Export::Func("SelectObject", as_win_api!(ops::select_object)),
            Export::Func("DeleteObject", as_win_api!(ops::delete_object)),
            Export::Func("BitBlt", as_win_api!(ops::bit_blt)),
            Export::Func("TextOutA", as_win_api!(ops::text_out_a)),
            Export::Func("TextOutW", as_win_api!(ops::text_out_w)),
        ]
    }
}
