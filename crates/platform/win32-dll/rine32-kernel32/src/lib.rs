use std::ffi::CString;
use std::ptr;
use std::sync::OnceLock;
use std::time::Duration;

use rine_dlls::{DllPlugin, Export, as_win_api, win32_stub};
use rine_types::errors::WinBool;
use rine_types::threading::{self, TLS_OUT_OF_INDEXES};

#[cfg(not(target_pointer_width = "32"))]
compile_error!(
    "crate `rine32-kernel32` must be built for a 32-bit target (for example: --target i686-unknown-linux-gnu)"
);

pub struct Kernel32Plugin32;

win32_stub!(CreateFileA, "kernel32");
win32_stub!(CreateFileW, "kernel32");
win32_stub!(ReadFile, "kernel32");
win32_stub!(WriteFile, "kernel32");
win32_stub!(CloseHandle, "kernel32");
win32_stub!(GetStdHandle, "kernel32");

const PAGE_NOACCESS: u32 = 0x01;
const PAGE_READONLY: u32 = 0x02;
const PAGE_READWRITE: u32 = 0x04;
const PAGE_EXECUTE: u32 = 0x10;
const PAGE_EXECUTE_READ: u32 = 0x20;
const PAGE_EXECUTE_READWRITE: u32 = 0x40;

struct CmdLineCache {
    ansi: CString,
    wide: Vec<u16>,
}

static CMD_LINE: OnceLock<CmdLineCache> = OnceLock::new();

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

unsafe fn init_cs(cs: *mut u8) {
    unsafe { ptr::write_bytes(cs, 0, 24) };

    let mutex = Box::into_raw(Box::new(unsafe {
        core::mem::zeroed::<libc::pthread_mutex_t>()
    }));
    let mut attr: libc::pthread_mutexattr_t = unsafe { core::mem::zeroed() };
    unsafe {
        libc::pthread_mutexattr_init(&mut attr);
        libc::pthread_mutexattr_settype(&mut attr, libc::PTHREAD_MUTEX_RECURSIVE);
        libc::pthread_mutex_init(mutex, &mut attr);
        libc::pthread_mutexattr_destroy(&mut attr);
    }

    unsafe { ptr::write(cs as *mut *mut libc::pthread_mutex_t, mutex) };
}

unsafe fn get_mutex(cs: *const u8) -> *mut libc::pthread_mutex_t {
    unsafe { ptr::read(cs as *const *mut libc::pthread_mutex_t) }
}

fn win_protect_to_linux(protect: u32) -> i32 {
    match protect {
        PAGE_NOACCESS => libc::PROT_NONE,
        PAGE_READONLY => libc::PROT_READ,
        PAGE_READWRITE => libc::PROT_READ | libc::PROT_WRITE,
        PAGE_EXECUTE => libc::PROT_EXEC,
        PAGE_EXECUTE_READ => libc::PROT_READ | libc::PROT_EXEC,
        PAGE_EXECUTE_READWRITE => libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
        _ => libc::PROT_READ | libc::PROT_WRITE,
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn ExitProcess(exit_code: u32) -> ! {
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
pub unsafe extern "stdcall" fn GetModuleHandleA(_module_name: *const u8) -> usize {
    0
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn GetModuleHandleW(_module_name: *const u16) -> usize {
    0
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn GetLastError() -> u32 {
    0
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn SetUnhandledExceptionFilter(_filter: usize) -> usize {
    0
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn InitializeCriticalSection(cs: *mut u8) {
    if cs.is_null() {
        return;
    }
    unsafe { init_cs(cs) };
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn EnterCriticalSection(cs: *mut u8) {
    if cs.is_null() {
        return;
    }
    let mut mutex = unsafe { get_mutex(cs) };
    if mutex.is_null() {
        unsafe { init_cs(cs) };
        mutex = unsafe { get_mutex(cs) };
    }
    unsafe { libc::pthread_mutex_lock(mutex) };
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn LeaveCriticalSection(cs: *mut u8) {
    if cs.is_null() {
        return;
    }
    let mutex = unsafe { get_mutex(cs) };
    if mutex.is_null() {
        return;
    }
    unsafe { libc::pthread_mutex_unlock(mutex) };
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn DeleteCriticalSection(cs: *mut u8) {
    if cs.is_null() {
        return;
    }
    let mutex = unsafe { get_mutex(cs) };
    if mutex.is_null() {
        return;
    }
    unsafe {
        libc::pthread_mutex_destroy(mutex);
        drop(Box::from_raw(mutex));
        ptr::write(cs as *mut *mut libc::pthread_mutex_t, ptr::null_mut());
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn LoadLibraryA(_file_name: *const u8) -> usize {
    0
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn GetProcAddress(_module: usize, _name: *const u8) -> usize {
    0
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn FreeLibrary(_module: usize) -> WinBool {
    WinBool::FALSE
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn VirtualProtect(
    address: *mut u8,
    size: usize,
    new_protect: u32,
    old_protect: *mut u32,
) -> WinBool {
    if !old_protect.is_null() {
        unsafe { *old_protect = new_protect };
    }

    let result = unsafe { libc::mprotect(address.cast(), size, win_protect_to_linux(new_protect)) };
    if result == 0 {
        WinBool::TRUE
    } else {
        WinBool::FALSE
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn VirtualQuery(
    _address: *const u8,
    _buffer: *mut u8,
    _length: usize,
) -> usize {
    0
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn TlsAlloc() -> u32 {
    threading::tls_alloc().unwrap_or(TLS_OUT_OF_INDEXES)
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn TlsFree(tls_index: u32) -> WinBool {
    if threading::tls_free(tls_index) {
        WinBool::TRUE
    } else {
        WinBool::FALSE
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn TlsGetValue(tls_index: u32) -> usize {
    threading::tls_get_value(tls_index)
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn TlsSetValue(tls_index: u32, value: usize) -> WinBool {
    if threading::tls_set_value(tls_index, value) {
        WinBool::TRUE
    } else {
        WinBool::FALSE
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn Sleep(milliseconds: u32) {
    std::thread::sleep(Duration::from_millis(milliseconds as u64));
}

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
            Export::Func(
                "SetUnhandledExceptionFilter",
                as_win_api!(SetUnhandledExceptionFilter),
            ),
            Export::Func("CreateFileA", as_win_api!(CreateFileA)),
            Export::Func("CreateFileW", as_win_api!(CreateFileW)),
            Export::Func("ReadFile", as_win_api!(ReadFile)),
            Export::Func("WriteFile", as_win_api!(WriteFile)),
            Export::Func("CloseHandle", as_win_api!(CloseHandle)),
            Export::Func("GetStdHandle", as_win_api!(GetStdHandle)),
            Export::Func(
                "InitializeCriticalSection",
                as_win_api!(InitializeCriticalSection),
            ),
            Export::Func("EnterCriticalSection", as_win_api!(EnterCriticalSection)),
            Export::Func("LeaveCriticalSection", as_win_api!(LeaveCriticalSection)),
            Export::Func("DeleteCriticalSection", as_win_api!(DeleteCriticalSection)),
            Export::Func("LoadLibraryA", as_win_api!(LoadLibraryA)),
            Export::Func("GetProcAddress", as_win_api!(GetProcAddress)),
            Export::Func("FreeLibrary", as_win_api!(FreeLibrary)),
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
