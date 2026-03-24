pub mod console;
pub mod file;
pub mod memory;
pub mod process;
pub mod sync;
pub mod thread;

use rine_dlls::{DllPlugin, Export, as_win_api};

pub struct Kernel32Plugin;

impl DllPlugin for Kernel32Plugin {
    fn dll_names(&self) -> &[&str] {
        &["kernel32.dll"]
    }

    fn exports(&self) -> Vec<Export> {
        vec![
            Export::Func("GetStdHandle", as_win_api!(console::GetStdHandle)),
            Export::Func("WriteConsoleA", as_win_api!(console::WriteConsoleA)),
            Export::Func("WriteConsoleW", as_win_api!(console::WriteConsoleW)),
            Export::Func("CreateFileA", as_win_api!(file::CreateFileA)),
            Export::Func("CreateFileW", as_win_api!(file::CreateFileW)),
            Export::Func("ReadFile", as_win_api!(file::ReadFile)),
            Export::Func("WriteFile", as_win_api!(file::WriteFile)),
            Export::Func("CloseHandle", as_win_api!(file::CloseHandle)),
            Export::Func("GetFileSize", as_win_api!(file::GetFileSize)),
            Export::Func("SetFilePointer", as_win_api!(file::SetFilePointer)),
            Export::Func("FindFirstFileA", as_win_api!(file::FindFirstFileA)),
            Export::Func("FindFirstFileW", as_win_api!(file::FindFirstFileW)),
            Export::Func("FindNextFileA", as_win_api!(file::FindNextFileA)),
            Export::Func("FindNextFileW", as_win_api!(file::FindNextFileW)),
            Export::Func("FindClose", as_win_api!(file::FindClose)),
            Export::Func("ExitProcess", as_win_api!(process::ExitProcess)),
            Export::Func("GetCommandLineA", as_win_api!(process::GetCommandLineA)),
            Export::Func("GetCommandLineW", as_win_api!(process::GetCommandLineW)),
            Export::Func("GetModuleHandleA", as_win_api!(process::GetModuleHandleA)),
            Export::Func("GetModuleHandleW", as_win_api!(process::GetModuleHandleW)),
            Export::Func("GetLastError", as_win_api!(process::GetLastError)),
            Export::Func(
                "SetUnhandledExceptionFilter",
                as_win_api!(process::SetUnhandledExceptionFilter),
            ),
            Export::Func(
                "InitializeCriticalSection",
                as_win_api!(sync::InitializeCriticalSection),
            ),
            Export::Func(
                "EnterCriticalSection",
                as_win_api!(sync::EnterCriticalSection),
            ),
            Export::Func(
                "LeaveCriticalSection",
                as_win_api!(sync::LeaveCriticalSection),
            ),
            Export::Func(
                "DeleteCriticalSection",
                as_win_api!(sync::DeleteCriticalSection),
            ),
            Export::Func("TlsGetValue", as_win_api!(thread::TlsGetValue)),
            Export::Func("Sleep", as_win_api!(thread::Sleep)),
            Export::Func("VirtualProtect", as_win_api!(memory::VirtualProtect)),
            Export::Func("VirtualQuery", as_win_api!(memory::VirtualQuery)),
        ]
    }
}
