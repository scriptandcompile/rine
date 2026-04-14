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
            Export::Func("DeleteDC", as_win_api!(ops::DeleteDC)),
            Export::Func("CreateSolidBrush", as_win_api!(ops::CreateSolidBrush)),
            Export::Func("SelectObject", as_win_api!(ops::select_object)),
            Export::Func("DeleteObject", as_win_api!(ops::delete_object)),
            Export::Func("BitBlt", as_win_api!(ops::bit_blt)),
            Export::Func("TextOutA", as_win_api!(ops::TextOutA)),
            Export::Func("TextOutW", as_win_api!(ops::TextOutW)),
        ]
    }

    fn partials(&self) -> Vec<rine_dlls::PartialExport> {
        vec![
            PartialExport {
                name: "CreateCompatibleDC",
                func: as_win_api!(ops::CreateCompatibleDC),
            },
            PartialExport {
                name: "CreateCompatibleBitmap",
                func: as_win_api!(ops::CreateCompatibleBitmap),
            },
            PartialExport {
                name: "CreatePen",
                func: as_win_api!(ops::CreatePen),
            },
        ]
    }
}
