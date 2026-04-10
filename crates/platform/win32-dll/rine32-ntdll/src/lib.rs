pub mod file;
pub mod process;

use rine_dlls::{DllPlugin, Export, StubExport, as_win_api};

use crate::file::NtCreateFile;

#[cfg(not(target_pointer_width = "32"))]
compile_error!(
    "crate `rine32-ntdll` must be built for a 32-bit target (for example: --target i686-unknown-linux-gnu)"
);

pub struct NtdllPlugin32;

impl DllPlugin for NtdllPlugin32 {
    fn dll_names(&self) -> &[&str] {
        &["ntdll.dll"]
    }

    fn exports(&self) -> Vec<Export> {
        vec![
            Export::Func("NtCreateFile", as_win_api!(NtCreateFile)),
            Export::Func("NtReadFile", as_win_api!(file::NtReadFile)),
            Export::Func("NtWriteFile", as_win_api!(file::NtWriteFile)),
            Export::Func("NtClose", as_win_api!(file::NtClose)),
            Export::Func(
                "NtQueryInformationFile",
                as_win_api!(file::NtQueryInformationFile),
            ),
            Export::Func(
                "NtTerminateProcess",
                as_win_api!(process::NtTerminateProcess),
            ),
            Export::Func(
                "RtlInitUnicodeString",
                as_win_api!(process::RtlInitUnicodeString),
            ),
            Export::Func("RtlGetVersion", as_win_api!(process::RtlGetVersion)),
        ]
    }

    fn stubs(&self) -> Vec<StubExport> {
        vec![
            StubExport {
                name: "NtReadFile",
                func: as_win_api!(file::NtReadFile),
            },
            StubExport {
                name: "NtWriteFile",
                func: as_win_api!(file::NtWriteFile),
            },
            StubExport {
                name: "NtClose",
                func: as_win_api!(file::NtClose),
            },
            StubExport {
                name: "NtQueryInformationFile",
                func: as_win_api!(file::NtQueryInformationFile),
            },
            StubExport {
                name: "NtTerminateProcess",
                func: as_win_api!(process::NtTerminateProcess),
            },
            StubExport {
                name: "RtlInitUnicodeString",
                func: as_win_api!(process::RtlInitUnicodeString),
            },
            StubExport {
                name: "RtlGetVersion",
                func: as_win_api!(process::RtlGetVersion),
            },
        ]
    }
}
