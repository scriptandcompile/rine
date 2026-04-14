#![allow(unsafe_op_in_unsafe_fn)]

mod ops;

use rine_dlls::{DllPlugin, Export, PartialExport, as_win_api};

#[cfg(not(target_pointer_width = "32"))]
compile_error!(
    "crate `rine32-gdi32` must be built for a 32-bit target (for example: --target i686-unknown-linux-gnu)"
);

pub struct Gdi32Plugin32;

impl DllPlugin for Gdi32Plugin32 {
    fn dll_names(&self) -> &[&str] {
        &["gdi32.dll"]
    }

    fn exports(&self) -> Vec<Export> {
        vec![
            Export::Func("DeleteDC", as_win_api!(ops::DeleteDC)),
            Export::Func("CreateSolidBrush", as_win_api!(ops::CreateSolidBrush)),
            Export::Func("SelectObject", as_win_api!(ops::SelectObject)),
            Export::Func("DeleteObject", as_win_api!(ops::DeleteObject)),
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
            PartialExport {
                name: "BitBlt",
                func: as_win_api!(ops::BitBlt),
            },
        ]
    }
}
