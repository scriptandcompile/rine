#![allow(unsafe_op_in_unsafe_fn)]

use rine_dlls::{DllPlugin, Export, PartialExport, as_win_api};
mod ops;

pub struct Gdi32Plugin;

impl DllPlugin for Gdi32Plugin {
    fn dll_names(&self) -> &[&str] {
        &["gdi32.dll"]
    }

    fn exports(&self) -> Vec<Export> {
        vec![
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

    fn partials(&self) -> Vec<rine_dlls::PartialExport> {
        vec![PartialExport {
            name: "CreateCompatibleDC",
            func: as_win_api!(ops::CreateCompatibleDC),
        }]
    }
}
