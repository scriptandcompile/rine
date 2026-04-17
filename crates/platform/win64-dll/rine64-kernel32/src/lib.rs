pub mod console;
pub mod environment;
pub mod file;
pub mod memory;
pub mod process;
pub mod sync;
pub mod thread;
pub mod version;

use rine_dlls::{DllPlugin, Export, PartialExport, StubExport, as_win_api};

pub struct Kernel32Plugin;

impl DllPlugin for Kernel32Plugin {
    fn dll_names(&self) -> &[&str] {
        &["kernel32.dll"]
    }
    fn exports(&self) -> Vec<Export> {
        vec![
            Export::Func("ExitProcess", as_win_api!(process::ExitProcess)),
            Export::Func("GetLastError", as_win_api!(process::GetLastError)),
            Export::Func("GetCommandLineA", as_win_api!(process::GetCommandLineA)),
            Export::Func("GetCommandLineW", as_win_api!(process::GetCommandLineW)),
            Export::Func("GetModuleHandleA", as_win_api!(process::GetModuleHandleA)),
            Export::Func("GetModuleHandleW", as_win_api!(process::GetModuleHandleW)),
            Export::Func("CreateProcessA", as_win_api!(process::CreateProcessA)),
            Export::Func("CreateProcessW", as_win_api!(process::CreateProcessW)),
            Export::Func(
                "GetCurrentProcessId",
                as_win_api!(process::GetCurrentProcessId),
            ),
            Export::Func("GetCurrentProcess", as_win_api!(process::GetCurrentProcess)),
            Export::Func(
                "GetExitCodeProcess",
                as_win_api!(process::GetExitCodeProcess),
            ),
            Export::Func("DeleteFileA", as_win_api!(file::DeleteFileA)),
            Export::Func("DeleteFileW", as_win_api!(file::DeleteFileW)),
            Export::Func("GetFileSize", as_win_api!(file::GetFileSize)),
            Export::Func("FindFirstFileA", as_win_api!(file::FindFirstFileA)),
            Export::Func("FindFirstFileW", as_win_api!(file::FindFirstFileW)),
            Export::Func("FindNextFileA", as_win_api!(file::FindNextFileA)),
            Export::Func("FindNextFileW", as_win_api!(file::FindNextFileW)),
            Export::Func("GetStdHandle", as_win_api!(console::GetStdHandle)),
            Export::Func("GetProcessHeap", as_win_api!(memory::GetProcessHeap)),
            Export::Func("HeapDestroy", as_win_api!(memory::HeapDestroy)),
            Export::Func(
                "InitializeCriticalSection",
                as_win_api!(sync::InitializeCriticalSection),
            ),
            Export::Func(
                "InitializeCriticalSectionAndSpinCount",
                as_win_api!(sync::InitializeCriticalSectionAndSpinCount),
            ),
            Export::Func(
                "TryEnterCriticalSection",
                as_win_api!(sync::TryEnterCriticalSection),
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
            Export::Func("CreateEventA", as_win_api!(sync::CreateEventA)),
            Export::Func("CreateEventW", as_win_api!(sync::CreateEventW)),
            Export::Func("SetEvent", as_win_api!(sync::SetEvent)),
            Export::Func(
                "SetUnhandledExceptionFilter",
                as_win_api!(process::SetUnhandledExceptionFilter),
            ),
            Export::Func("ResetEvent", as_win_api!(sync::ResetEvent)),
            Export::Func("CreateMutexA", as_win_api!(sync::CreateMutexA)),
            Export::Func("CreateMutexW", as_win_api!(sync::CreateMutexW)),
            Export::Func("ReleaseMutex", as_win_api!(sync::ReleaseMutex)),
            Export::Func("CreateSemaphoreA", as_win_api!(sync::CreateSemaphoreA)),
            Export::Func("CreateSemaphoreW", as_win_api!(sync::CreateSemaphoreW)),
            Export::Func("ReleaseSemaphore", as_win_api!(sync::ReleaseSemaphore)),
            Export::Func("TlsAlloc", as_win_api!(thread::TlsAlloc)),
            Export::Func("TlsFree", as_win_api!(thread::TlsFree)),
            Export::Func("TlsGetValue", as_win_api!(thread::TlsGetValue)),
            Export::Func("TlsSetValue", as_win_api!(thread::TlsSetValue)),
            Export::Func("CreateThread", as_win_api!(thread::CreateThread)),
            Export::Func("GetCurrentThread", as_win_api!(thread::GetCurrentThread)),
            Export::Func(
                "GetCurrentThreadId",
                as_win_api!(thread::GetCurrentThreadId),
            ),
            Export::Func("GetExitCodeThread", as_win_api!(thread::GetExitCodeThread)),
            Export::Func("Sleep", as_win_api!(thread::Sleep)),
            Export::Func(
                "WaitForSingleObject",
                as_win_api!(thread::WaitForSingleObject),
            ),
            Export::Func(
                "WaitForMultipleObjects",
                as_win_api!(thread::WaitForMultipleObjects),
            ),
            Export::Func(
                "SetEnvironmentVariableA",
                as_win_api!(environment::SetEnvironmentVariableA),
            ),
            Export::Func(
                "SetEnvironmentVariableW",
                as_win_api!(environment::SetEnvironmentVariableW),
            ),
            Export::Func("GetVersion", as_win_api!(version::GetVersion)),
            Export::Func("GetVersionExA", as_win_api!(version::GetVersionExA)),
            Export::Func("GetVersionExW", as_win_api!(version::GetVersionExW)),
        ]
    }

    fn stubs(&self) -> Vec<StubExport> {
        vec![
            StubExport {
                name: "FindClose",
                func: as_win_api!(file::FindClose),
            },
            StubExport {
                name: "VirtualQuery",
                func: as_win_api!(memory::VirtualQuery),
            },
            StubExport {
                name: "FreeEnvironmentStringsA",
                func: as_win_api!(environment::FreeEnvironmentStringsA),
            },
            StubExport {
                name: "FreeEnvironmentStringsW",
                func: as_win_api!(environment::FreeEnvironmentStringsW),
            },
        ]
    }

    fn partials(&self) -> Vec<PartialExport> {
        vec![
            PartialExport {
                name: "SetFilePointer",
                func: as_win_api!(file::SetFilePointer),
            },
            PartialExport {
                name: "CloseHandle",
                func: as_win_api!(file::CloseHandle),
            },
            PartialExport {
                name: "CreateFileA",
                func: as_win_api!(file::CreateFileA),
            },
            PartialExport {
                name: "CreateFileW",
                func: as_win_api!(file::CreateFileW),
            },
            PartialExport {
                name: "ReadFile",
                func: as_win_api!(file::ReadFile),
            },
            PartialExport {
                name: "WriteFile",
                func: as_win_api!(file::WriteFile),
            },
            PartialExport {
                name: "FlushFileBuffers",
                func: as_win_api!(file::FlushFileBuffers),
            },
            PartialExport {
                name: "GetEnvironmentStrings",
                func: as_win_api!(environment::GetEnvironmentStrings),
            },
            PartialExport {
                name: "GetEnvironmentStringsW",
                func: as_win_api!(environment::GetEnvironmentStringsW),
            },
            PartialExport {
                name: "HeapCreate",
                func: as_win_api!(memory::HeapCreate),
            },
            PartialExport {
                name: "HeapAlloc",
                func: as_win_api!(memory::HeapAlloc),
            },
            PartialExport {
                name: "HeapSize",
                func: as_win_api!(memory::HeapSize),
            },
            PartialExport {
                name: "HeapFree",
                func: as_win_api!(memory::HeapFree),
            },
            PartialExport {
                name: "HeapReAlloc",
                func: as_win_api!(memory::HeapReAlloc),
            },
            PartialExport {
                name: "VirtualAlloc",
                func: as_win_api!(memory::VirtualAlloc),
            },
            PartialExport {
                name: "VirtualFree",
                func: as_win_api!(memory::VirtualFree),
            },
            PartialExport {
                name: "VirtualProtect",
                func: as_win_api!(memory::VirtualProtect),
            },
            PartialExport {
                name: "WriteConsoleA",
                func: as_win_api!(console::WriteConsoleA),
            },
            PartialExport {
                name: "WriteConsoleW",
                func: as_win_api!(console::WriteConsoleW),
            },
            PartialExport {
                name: "GetEnvironmentVariableA",
                func: as_win_api!(environment::GetEnvironmentVariableA),
            },
            PartialExport {
                name: "GetEnvironmentVariableW",
                func: as_win_api!(environment::GetEnvironmentVariableW),
            },
            PartialExport {
                name: "ExpandEnvironmentStringsA",
                func: as_win_api!(environment::ExpandEnvironmentStringsA),
            },
            PartialExport {
                name: "ExpandEnvironmentStringsW",
                func: as_win_api!(environment::ExpandEnvironmentStringsW),
            },
        ]
    }
}
