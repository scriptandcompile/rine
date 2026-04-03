use rine_dlls::{DllPlugin, Export, as_win_api, win32_stub};

#[cfg(not(target_pointer_width = "32"))]
compile_error!(
    "crate `rine32-ntdll` must be built for a 32-bit target (for example: --target i686-unknown-linux-gnu)"
);

pub struct NtdllPlugin32;

win32_stub!(NtCreateFile, "ntdll");
win32_stub!(NtReadFile, "ntdll");
win32_stub!(NtWriteFile, "ntdll");
win32_stub!(NtClose, "ntdll");
win32_stub!(NtQueryInformationFile, "ntdll");
win32_stub!(NtTerminateProcess, "ntdll");
win32_stub!(RtlInitUnicodeString, "ntdll");
win32_stub!(RtlGetVersion, "ntdll");

impl DllPlugin for NtdllPlugin32 {
    fn dll_names(&self) -> &[&str] {
        &["ntdll.dll"]
    }

    fn exports(&self) -> Vec<Export> {
        vec![
            Export::Func("NtCreateFile", as_win_api!(NtCreateFile)),
            Export::Func("NtReadFile", as_win_api!(NtReadFile)),
            Export::Func("NtWriteFile", as_win_api!(NtWriteFile)),
            Export::Func("NtClose", as_win_api!(NtClose)),
            Export::Func(
                "NtQueryInformationFile",
                as_win_api!(NtQueryInformationFile),
            ),
            Export::Func("NtTerminateProcess", as_win_api!(NtTerminateProcess)),
            Export::Func("RtlInitUnicodeString", as_win_api!(RtlInitUnicodeString)),
            Export::Func("RtlGetVersion", as_win_api!(RtlGetVersion)),
        ]
    }
}
