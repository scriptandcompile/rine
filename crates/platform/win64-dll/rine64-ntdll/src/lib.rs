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
            Export::Func("NtWriteFile", as_win_api!(file::NtWriteFile)),
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
