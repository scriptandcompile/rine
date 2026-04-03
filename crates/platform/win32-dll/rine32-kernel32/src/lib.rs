use rine_dlls::{DllPlugin, Export, as_win_api};

pub struct Kernel32Plugin32;

macro_rules! win32_stub {
    ($name:ident) => {
        #[allow(non_snake_case)]
        #[allow(clippy::missing_safety_doc)]
        pub unsafe extern "win64" fn $name() -> u32 {
            tracing::warn!(api = stringify!($name), "win32 kernel32 stub called");
            0
        }
    };
}

win32_stub!(ExitProcess);
win32_stub!(GetCommandLineA);
win32_stub!(GetCommandLineW);
win32_stub!(GetModuleHandleA);
win32_stub!(GetModuleHandleW);
win32_stub!(GetLastError);
win32_stub!(CreateFileA);
win32_stub!(CreateFileW);
win32_stub!(ReadFile);
win32_stub!(WriteFile);
win32_stub!(CloseHandle);
win32_stub!(GetStdHandle);
win32_stub!(VirtualProtect);
win32_stub!(VirtualQuery);
win32_stub!(TlsAlloc);
win32_stub!(TlsFree);
win32_stub!(TlsGetValue);
win32_stub!(TlsSetValue);
win32_stub!(Sleep);

impl DllPlugin for Kernel32Plugin32 {
    fn dll_names(&self) -> &[&str] {
        &["kernel32.dll"]
    }

    fn exports(&self) -> Vec<Export> {
        vec![
            Export::Func("ExitProcess", as_win_api!(ExitProcess)),
            Export::Func("GetCommandLineA", as_win_api!(GetCommandLineA)),
            Export::Func("GetCommandLineW", as_win_api!(GetCommandLineW)),
            Export::Func("GetModuleHandleA", as_win_api!(GetModuleHandleA)),
            Export::Func("GetModuleHandleW", as_win_api!(GetModuleHandleW)),
            Export::Func("GetLastError", as_win_api!(GetLastError)),
            Export::Func("CreateFileA", as_win_api!(CreateFileA)),
            Export::Func("CreateFileW", as_win_api!(CreateFileW)),
            Export::Func("ReadFile", as_win_api!(ReadFile)),
            Export::Func("WriteFile", as_win_api!(WriteFile)),
            Export::Func("CloseHandle", as_win_api!(CloseHandle)),
            Export::Func("GetStdHandle", as_win_api!(GetStdHandle)),
            Export::Func("VirtualProtect", as_win_api!(VirtualProtect)),
            Export::Func("VirtualQuery", as_win_api!(VirtualQuery)),
            Export::Func("TlsAlloc", as_win_api!(TlsAlloc)),
            Export::Func("TlsFree", as_win_api!(TlsFree)),
            Export::Func("TlsGetValue", as_win_api!(TlsGetValue)),
            Export::Func("TlsSetValue", as_win_api!(TlsSetValue)),
            Export::Func("Sleep", as_win_api!(Sleep)),
        ]
    }
}
