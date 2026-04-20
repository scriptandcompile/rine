pub mod file;
pub mod memory;
pub mod process;
pub mod rtl;

use rine_dlls::{DllPlugin, Export, PartialExport, StubExport, as_win_api};

pub struct NtdllPlugin;

impl DllPlugin for NtdllPlugin {
    fn dll_names(&self) -> &[&str] {
        &["ntdll.dll"]
    }

    fn exports(&self) -> Vec<Export> {
        vec![]
    }

    fn stubs(&self) -> Vec<StubExport> {
        vec![]
    }

    fn partials(&self) -> Vec<PartialExport> {
        vec![
            // file.rs
            PartialExport {
                name: "NtReadFile",
                func: as_win_api!(file::NtReadFile),
            },
            PartialExport {
                name: "NtWriteFile",
                func: as_win_api!(file::NtWriteFile),
            },
            PartialExport {
                name: "NtCreateFile",
                func: as_win_api!(file::NtCreateFile),
            },
            PartialExport {
                name: "NtClose",
                func: as_win_api!(file::NtClose),
            },
            PartialExport {
                name: "NtQueryInformationFile",
                func: as_win_api!(file::NtQueryInformationFile),
            },
            // process.rs
            PartialExport {
                name: "NtTerminateProcess",
                func: as_win_api!(process::NtTerminateProcess),
            },
            // rtl.rs
            PartialExport {
                name: "RtlInitUnicodeString",
                func: as_win_api!(rtl::RtlInitUnicodeString),
            },
            PartialExport {
                name: "RtlGetVersion",
                func: as_win_api!(rtl::RtlGetVersion),
            },
        ]
    }
}
