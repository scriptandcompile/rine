use std::ffi::CString;
use std::sync::OnceLock;

use rine_dlls::win32_stub;
use rine_types::errors::WinBool;
use rine_types::threading;

struct CmdLineCache {
    ansi: CString,
    wide: Vec<u16>,
}

static CMD_LINE: OnceLock<CmdLineCache> = OnceLock::new();

win32_stub!(GetModuleHandleA, "kernel32");
win32_stub!(GetModuleHandleW, "kernel32");
win32_stub!(GetLastError, "kernel32");
win32_stub!(SetUnhandledExceptionFilter, "kernel32");
win32_stub!(LoadLibraryA, "kernel32");
win32_stub!(GetProcAddress, "kernel32");
win32_stub!(FreeLibrary, "kernel32");

fn cached_cmd_line() -> &'static CmdLineCache {
    CMD_LINE.get_or_init(|| {
        let args: Vec<String> = std::env::args().collect();
        let joined = args
            .iter()
            .map(|arg| {
                if arg.contains(' ') {
                    format!("\"{arg}\"")
                } else {
                    arg.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        let ansi = CString::new(joined.clone()).unwrap_or_default();
        let mut wide: Vec<u16> = joined.encode_utf16().collect();
        wide.push(0);

        CmdLineCache { ansi, wide }
    })
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn ExitProcess(exit_code: u32) -> ! {
    let tid = unsafe { libc::syscall(libc::SYS_gettid) as u32 };
    rine_types::dev_notify!(on_thread_exited(tid, exit_code));
    rine_types::dev_notify!(on_process_exiting(exit_code as i32));
    std::process::exit(exit_code as i32);
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn GetCommandLineA() -> *const u8 {
    cached_cmd_line().ansi.as_ptr().cast()
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn GetCommandLineW() -> *const u16 {
    cached_cmd_line().wide.as_ptr()
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn GetCurrentProcessId() -> u32 {
    unsafe { libc::getpid() as u32 }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn GetCurrentProcess() -> isize {
    -1
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn GetExitCodeProcess(
    _process_handle: isize,
    exit_code_out: *mut u32,
) -> WinBool {
    if exit_code_out.is_null() {
        return WinBool::FALSE;
    }
    unsafe { *exit_code_out = threading::STILL_ACTIVE };
    WinBool::TRUE
}
