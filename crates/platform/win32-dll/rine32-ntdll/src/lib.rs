use rine_dlls::{as_win_api, DllPlugin, Export};

pub struct NtdllPlugin32;

macro_rules! win32_stub {
    ($name:ident) => {
        #[allow(non_snake_case)]
        #[allow(clippy::missing_safety_doc)]
        pub unsafe extern "win64" fn $name() -> u32 {
            tracing::warn!(api = stringify!($name), "win32 ntdll stub called");
            0
        }
    };
}

win32_stub!(NtCreateFile);
win32_stub!(NtReadFile);
win32_stub!(NtWriteFile);
win32_stub!(NtClose);
win32_stub!(NtQueryInformationFile);
win32_stub!(NtTerminateProcess);
win32_stub!(RtlInitUnicodeString);
win32_stub!(RtlGetVersion);

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
