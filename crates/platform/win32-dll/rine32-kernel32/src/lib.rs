pub mod console;
pub mod environment;
pub mod file;
pub mod memory;
pub mod process;
pub mod sync;
pub mod thread;

use rine_dlls::{DllPlugin, Export, as_win_api, win32_stub};

#[cfg(not(target_pointer_width = "32"))]
compile_error!(
    "crate `rine32-kernel32` must be built for a 32-bit target (for example: --target i686-unknown-linux-gnu)"
);

pub struct Kernel32Plugin32;

win32_stub!(GetModuleHandleA, "kernel32");
win32_stub!(GetModuleHandleW, "kernel32");
win32_stub!(GetLastError, "kernel32");
win32_stub!(LoadLibraryA, "kernel32");
win32_stub!(GetProcAddress, "kernel32");
win32_stub!(FreeLibrary, "kernel32");
win32_stub!(GetCurrentProcess, "kernel32");
win32_stub!(VirtualQuery, "kernel32");

impl DllPlugin for Kernel32Plugin32 {
    fn dll_names(&self) -> &[&str] {
        &["kernel32.dll"]
    }

    fn exports(&self) -> Vec<Export> {
        vec![
            Export::Func("ExitProcess", as_win_api!(process::ExitProcess)),
            Export::Func("GetCommandLineA", as_win_api!(process::GetCommandLineA)),
            Export::Func("GetCommandLineW", as_win_api!(process::GetCommandLineW)),
            Export::Func("CreateProcessA", as_win_api!(process::CreateProcessA)),
            Export::Func("CreateProcessW", as_win_api!(process::CreateProcessW)),
            Export::Func(
                "GetCurrentProcessId",
                as_win_api!(process::GetCurrentProcessId),
            ),
            Export::Func(
                "GetExitCodeProcess",
                as_win_api!(process::GetExitCodeProcess),
            ),
            Export::Func("CreateFileA", as_win_api!(file::CreateFileA)),
            Export::Func("CreateFileW", as_win_api!(file::CreateFileW)),
            Export::Func("DeleteFileA", as_win_api!(file::DeleteFileA)),
            Export::Func("DeleteFileW", as_win_api!(file::DeleteFileW)),
            Export::Func("GetFileSize", as_win_api!(file::GetFileSize)),
            Export::Func("ReadFile", as_win_api!(file::ReadFile)),
            Export::Func("WriteFile", as_win_api!(file::WriteFile)),
            Export::Func("FlushFileBuffers", as_win_api!(file::FlushFileBuffers)),
            Export::Func("WriteConsoleA", as_win_api!(console::WriteConsoleA)),
            Export::Func("WriteConsoleW", as_win_api!(console::WriteConsoleW)),
            Export::Func("CloseHandle", as_win_api!(file::CloseHandle)),
            Export::Func("SetFilePointer", as_win_api!(file::SetFilePointer)),
            Export::Func("FindClose", as_win_api!(file::FindClose)),
            Export::Func("GetStdHandle", as_win_api!(console::GetStdHandle)),
            Export::Func("GetProcessHeap", as_win_api!(memory::GetProcessHeap)),
            Export::Func("HeapCreate", as_win_api!(memory::HeapCreate)),
            Export::Func("HeapDestroy", as_win_api!(memory::HeapDestroy)),
            Export::Func("HeapAlloc", as_win_api!(memory::HeapAlloc)),
            Export::Func("HeapFree", as_win_api!(memory::HeapFree)),
            Export::Func("HeapReAlloc", as_win_api!(memory::HeapReAlloc)),
            Export::Func("VirtualAlloc", as_win_api!(memory::VirtualAlloc)),
            Export::Func("VirtualFree", as_win_api!(memory::VirtualFree)),
            Export::Func(
                "InitializeCriticalSection",
                as_win_api!(sync::InitializeCriticalSection),
            ),
            Export::Func(
                "InitializeCriticalSectionAndSpinCount",
                as_win_api!(sync::InitializeCriticalSectionAndSpinCount),
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
            Export::Func("VirtualProtect", as_win_api!(memory::VirtualProtect)),
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
            Export::Func(
                "WaitForSingleObject",
                as_win_api!(thread::WaitForSingleObject),
            ),
            Export::Func(
                "WaitForMultipleObjects",
                as_win_api!(thread::WaitForMultipleObjects),
            ),
            Export::Func(
                "GetEnvironmentVariableA",
                as_win_api!(environment::GetEnvironmentVariableA),
            ),
            Export::Func(
                "GetEnvironmentVariableW",
                as_win_api!(environment::GetEnvironmentVariableW),
            ),
            Export::Func(
                "SetEnvironmentVariableA",
                as_win_api!(environment::SetEnvironmentVariableA),
            ),
            Export::Func(
                "SetEnvironmentVariableW",
                as_win_api!(environment::SetEnvironmentVariableW),
            ),
            Export::Func(
                "ExpandEnvironmentStringsA",
                as_win_api!(environment::ExpandEnvironmentStringsA),
            ),
            Export::Func(
                "ExpandEnvironmentStringsW",
                as_win_api!(environment::ExpandEnvironmentStringsW),
            ),
            Export::Func(
                "GetEnvironmentStringsW",
                as_win_api!(environment::GetEnvironmentStringsW),
            ),
            Export::Func(
                "FreeEnvironmentStringsW",
                as_win_api!(environment::FreeEnvironmentStringsW),
            ),
            Export::Func("Sleep", as_win_api!(thread::Sleep)),
        ]
    }
}
