use rine_dlls::{DllPlugin, Export, as_win_api, win32_stub};

#[cfg(not(target_pointer_width = "32"))]
compile_error!(
    "crate `rine32-kernel32` must be built for a 32-bit target (for example: --target i686-unknown-linux-gnu)"
);

pub struct Kernel32Plugin32;

win32_stub!(ExitProcess, "kernel32");
win32_stub!(GetCommandLineA, "kernel32");
win32_stub!(GetCommandLineW, "kernel32");
win32_stub!(GetModuleHandleA, "kernel32");
win32_stub!(GetModuleHandleW, "kernel32");
win32_stub!(GetLastError, "kernel32");
win32_stub!(CreateFileA, "kernel32");
win32_stub!(CreateFileW, "kernel32");
win32_stub!(ReadFile, "kernel32");
win32_stub!(WriteFile, "kernel32");
win32_stub!(CloseHandle, "kernel32");
win32_stub!(GetStdHandle, "kernel32");
win32_stub!(VirtualProtect, "kernel32");
win32_stub!(VirtualQuery, "kernel32");
win32_stub!(TlsAlloc, "kernel32");
win32_stub!(TlsFree, "kernel32");
win32_stub!(TlsGetValue, "kernel32");
win32_stub!(TlsSetValue, "kernel32");
win32_stub!(Sleep, "kernel32");

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
