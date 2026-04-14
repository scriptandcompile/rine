#![allow(unsafe_op_in_unsafe_fn)]

pub mod error;
pub mod open;
pub mod save;

use rine_dlls::{DllPlugin, Export, PartialExport, as_win_api};

#[cfg(not(target_pointer_width = "32"))]
compile_error!(
    "crate `rine32-comdlg32` must be built for a 32-bit target (for example: --target i686-unknown-linux-gnu)"
);

pub struct Comdlg32Plugin32;

impl DllPlugin for Comdlg32Plugin32 {
    fn dll_names(&self) -> &[&str] {
        &["comdlg32.dll"]
    }

    fn exports(&self) -> Vec<Export> {
        vec![]
    }

    fn partials(&self) -> Vec<PartialExport> {
        vec![
            PartialExport {
                name: "GetOpenFileNameA",
                func: as_win_api!(open::GetOpenFileNameA),
            },
            PartialExport {
                name: "GetOpenFileNameW",
                func: as_win_api!(open::GetOpenFileNameW),
            },
            PartialExport {
                name: "GetSaveFileNameA",
                func: as_win_api!(save::GetSaveFileNameA),
            },
            PartialExport {
                name: "GetSaveFileNameW",
                func: as_win_api!(save::GetSaveFileNameW),
            },
            PartialExport {
                name: "CommDlgExtendedError",
                func: as_win_api!(error::CommDlgExtendedError),
            },
        ]
    }
}
