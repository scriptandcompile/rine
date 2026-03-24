pub mod file;
pub mod memory;
pub mod process;
pub mod rtl;

use rine_dlls::{DllPlugin, Export, as_win_api};

pub struct NtdllPlugin;

impl DllPlugin for NtdllPlugin {
    fn dll_names(&self) -> &[&str] {
        &["ntdll.dll"]
    }

    fn exports(&self) -> Vec<Export> {
        vec![
            Export::Func("NtCreateFile", as_win_api!(file::NtCreateFile)),
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
                as_win_api!(rtl::RtlInitUnicodeString),
            ),
        ]
    }
}
