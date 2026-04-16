pub mod file;
pub mod process;

use rine_dlls::{DllPlugin, Export, StubExport, as_win_api};

pub struct NtdllPlugin32;

impl DllPlugin for NtdllPlugin32 {
    fn dll_names(&self) -> &[&str] {
        &["ntdll.dll"]
    }

    fn exports(&self) -> Vec<Export> {
        vec![
            Export::Func("NtCreateFile", as_win_api!(file::nt_create_file)),
        ]
    }

    fn stubs(&self) -> Vec<StubExport> {
        vec![
            StubExport {
                name: "NtReadFile",
                func: as_win_api!(file::nt_read_file),
            },
            StubExport {
                name: "NtWriteFile",
                func: as_win_api!(file::nt_write_file),
            },
            StubExport {
                name: "NtClose",
                func: as_win_api!(file::nt_close),
            },
            StubExport {
                name: "NtQueryInformationFile",
                func: as_win_api!(file::nt_query_information_file),
            },
            StubExport {
                name: "NtTerminateProcess",
                func: as_win_api!(process::nt_terminate_process),
            },
            StubExport {
                name: "RtlInitUnicodeString",
                func: as_win_api!(process::rtl_init_unicode_string),
            },
            StubExport {
                name: "RtlGetVersion",
                func: as_win_api!(process::rtl_get_version),
            },
        ]
    }
}
