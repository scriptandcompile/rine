#![allow(unsafe_op_in_unsafe_fn)]

mod error;
mod open;
mod save;

use rine_dlls::{DllPlugin, Export, PartialExport, as_win_api};

pub struct Comdlg32Plugin;

impl DllPlugin for Comdlg32Plugin {
    fn dll_names(&self) -> &[&str] {
        &["comdlg32.dll"]
    }

    fn exports(&self) -> Vec<Export> {
        vec![]
    }

    fn partials(&self) -> Vec<rine_dlls::PartialExport> {
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
