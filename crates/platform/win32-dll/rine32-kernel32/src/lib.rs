use std::alloc::Layout;
use std::collections::HashMap;
use std::ffi::CString;
use std::ptr;
use std::sync::LazyLock;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use rine_dlls::{DllPlugin, Export, as_win_api, win32_stub};
use rine_types::errors::WinBool;
use rine_types::handles::{
    Handle, HandleEntry, INVALID_HANDLE_VALUE, handle_table, handle_to_fd, std_handle_to_fd,
};
use rine_types::strings::{read_cstr, read_wstr, write_cstr, write_wstr};
use rine_types::threading::{self, TLS_OUT_OF_INDEXES};

#[cfg(not(target_pointer_width = "32"))]
compile_error!(
    "crate `rine32-kernel32` must be built for a 32-bit target (for example: --target i686-unknown-linux-gnu)"
);

pub struct Kernel32Plugin32;

win32_stub!(CreateFileA, "kernel32");
win32_stub!(CreateFileW, "kernel32");
win32_stub!(ReadFile, "kernel32");

const PAGE_NOACCESS: u32 = 0x01;
const PAGE_READONLY: u32 = 0x02;
const PAGE_READWRITE: u32 = 0x04;
const PAGE_EXECUTE: u32 = 0x10;
const PAGE_EXECUTE_READ: u32 = 0x20;
const PAGE_EXECUTE_READWRITE: u32 = 0x40;

const HEAP_ZERO_MEMORY: u32 = 0x0000_0008;
const MEM_COMMIT: u32 = 0x0000_1000;
const MEM_RESERVE: u32 = 0x0000_2000;
const MEM_RELEASE: u32 = 0x0000_8000;

struct CmdLineCache {
    ansi: CString,
    wide: Vec<u16>,
}

static CMD_LINE: OnceLock<CmdLineCache> = OnceLock::new();
static NEXT_THREAD_ID: AtomicU32 = AtomicU32::new(1000);
static DEFAULT_HEAP: LazyLock<Handle> = LazyLock::new(|| {
    handle_table().insert(HandleEntry::Heap(rine_types::handles::HeapState {
        allocations: Mutex::new(HashMap::new()),
        flags: 0,
    }))
});
static VIRTUAL_REGIONS: LazyLock<Mutex<HashMap<usize, usize>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

struct SyncPtr(*mut u16);
unsafe impl Send for SyncPtr {}
unsafe impl Sync for SyncPtr {}

static ENV_BLOCK_W: OnceLock<SyncPtr> = OnceLock::new();

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
pub unsafe extern "stdcall" fn GetStdHandle(nstd_handle: u32) -> isize {
    match std_handle_to_fd(nstd_handle) {
        Some(fd) => (fd as isize) + 0x1000,
        None => INVALID_HANDLE_VALUE.as_raw(),
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn WriteFile(
    file: isize,
    buffer: *const u8,
    bytes_to_write: u32,
    bytes_written: *mut u32,
    _overlapped: *mut core::ffi::c_void,
) -> WinBool {
    let handle = Handle::from_raw(file);
    let Some(fd) = handle_to_fd(handle) else {
        return WinBool::FALSE;
    };

    let written = unsafe { libc::write(fd, buffer.cast(), bytes_to_write as usize) };
    if written < 0 {
        return WinBool::FALSE;
    }

    if !bytes_written.is_null() {
        unsafe { *bytes_written = written as u32 };
    }
    WinBool::TRUE
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn WriteConsoleA(
    console_output: isize,
    buffer: *const u8,
    chars_to_write: u32,
    chars_written: *mut u32,
    _reserved: *const core::ffi::c_void,
) -> WinBool {
    let handle = Handle::from_raw(console_output);
    let Some(fd) = handle_to_fd(handle) else {
        return WinBool::FALSE;
    };

    let written = unsafe { libc::write(fd, buffer.cast(), chars_to_write as usize) };

    if written < 0 {
        return WinBool::FALSE;
    }

    if !chars_written.is_null() {
        unsafe { *chars_written = written as u32 };
    }
    WinBool::TRUE
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn WriteConsoleW(
    console_output: isize,
    buffer: *const u16,
    chars_to_write: u32,
    chars_written: *mut u32,
    _reserved: *const core::ffi::c_void,
) -> WinBool {
    let handle = Handle::from_raw(console_output);
    let Some(fd) = handle_to_fd(handle) else {
        return WinBool::FALSE;
    };

    let wide_slice = unsafe { core::slice::from_raw_parts(buffer, chars_to_write as usize) };
    let utf8: String = String::from_utf16_lossy(wide_slice);
    let written = unsafe { libc::write(fd, utf8.as_ptr().cast(), utf8.len()) };

    if written < 0 {
        return WinBool::FALSE;
    }
    if !chars_written.is_null() {
        unsafe { *chars_written = chars_to_write };
    }
    WinBool::TRUE
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn CloseHandle(object: isize) -> WinBool {
    let handle = Handle::from_raw(object);
    match handle_table().remove(handle) {
        Some(HandleEntry::Thread(_)) => WinBool::TRUE,
        Some(HandleEntry::Event(_)) => WinBool::TRUE,
        Some(HandleEntry::Process(_)) => WinBool::TRUE,
        Some(HandleEntry::Mutex(_)) => WinBool::TRUE,
        Some(HandleEntry::Semaphore(_)) => WinBool::TRUE,
        Some(HandleEntry::Heap(_)) => WinBool::TRUE,
        Some(HandleEntry::RegistryKey(_)) => WinBool::TRUE,
        Some(HandleEntry::FindData(_)) => WinBool::TRUE,
        Some(HandleEntry::File(fd)) => {
            if fd <= 2 {
                WinBool::TRUE
            } else {
                unsafe { libc::close(fd) };
                WinBool::TRUE
            }
        }
        Some(HandleEntry::Window(_)) => WinBool::FALSE,
        None => WinBool::FALSE,
    }
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
pub unsafe extern "stdcall" fn GetModuleHandleA(_module_name: *const u8) -> usize {
    0
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn GetModuleHandleW(_module_name: *const u16) -> usize {
    0
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

type ThreadStartRoutine = unsafe extern "stdcall" fn(usize) -> u32;

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn CreateThread(
    _security_attrs: usize,
    _stack_size: usize,
    start_address: usize,
    parameter: usize,
    _creation_flags: u32,
    thread_id_out: *mut u32,
) -> isize {
    if start_address == 0 {
        return INVALID_HANDLE_VALUE.as_raw();
    }

    let exit_code = Arc::new(AtomicU32::new(threading::STILL_ACTIVE));
    let completed = Arc::new((Mutex::new(false), Condvar::new()));
    let tid = NEXT_THREAD_ID.fetch_add(1, Ordering::Relaxed);

    let waitable = threading::ThreadWaitable {
        exit_code: Arc::clone(&exit_code),
        completed: Arc::clone(&completed),
    };
    let h = handle_table().insert(HandleEntry::Thread(waitable));

    if !thread_id_out.is_null() {
        unsafe { ptr::write(thread_id_out, tid) };
    }

    let result = std::thread::Builder::new().spawn(move || {
        let start_fn: ThreadStartRoutine = unsafe { core::mem::transmute(start_address) };
        let code = unsafe { start_fn(parameter) };
        exit_code.store(code, Ordering::Release);
        let (lock, cvar) = &*completed;
        *lock.lock().unwrap() = true;
        cvar.notify_all();
    });

    match result {
        Ok(join_handle) => {
            drop(join_handle);
            h.as_raw()
        }
        Err(_) => {
            handle_table().remove(h);
            INVALID_HANDLE_VALUE.as_raw()
        }
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn GetCurrentThread() -> isize {
    -2
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn GetCurrentThreadId() -> u32 {
    unsafe { libc::syscall(libc::SYS_gettid) as u32 }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn GetExitCodeThread(
    thread_handle: isize,
    exit_code_out: *mut u32,
) -> WinBool {
    if exit_code_out.is_null() {
        return WinBool::FALSE;
    }
    let h = Handle::from_raw(thread_handle);
    match handle_table().get_thread_exit_code(h) {
        Some(code) => {
            unsafe { ptr::write(exit_code_out, code) };
            WinBool::TRUE
        }
        None => WinBool::FALSE,
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn WaitForSingleObject(handle: isize, timeout_ms: u32) -> u32 {
    let h = Handle::from_raw(handle);
    match handle_table().get_waitable(h) {
        Some(waitable) => threading::wait_on(&waitable, timeout_ms),
        None => threading::WaitStatus::WAIT_FAILED.0,
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn WaitForMultipleObjects(
    count: u32,
    handles_ptr: *const isize,
    wait_all: WinBool,
    timeout_ms: u32,
) -> u32 {
    if handles_ptr.is_null() || count == 0 || count > 64 {
        return threading::WaitStatus::WAIT_FAILED.0;
    }

    let raw_handles: Vec<isize> = (0..count as usize)
        .map(|i| unsafe { *handles_ptr.add(i) })
        .collect();

    let waitables: Vec<Option<threading::Waitable>> = raw_handles
        .iter()
        .map(|&raw| handle_table().get_waitable(Handle::from_raw(raw)))
        .collect();

    if waitables.iter().any(|w| w.is_none()) {
        return threading::WaitStatus::WAIT_FAILED.0;
    }
    let waitables: Vec<threading::Waitable> = waitables.into_iter().flatten().collect();

    if wait_all.is_true() {
        let start = std::time::Instant::now();
        for w in &waitables {
            let remaining = if timeout_ms == threading::INFINITE {
                threading::INFINITE
            } else {
                let elapsed = start.elapsed().as_millis() as u32;
                if elapsed >= timeout_ms {
                    return threading::WaitStatus::WAIT_TIMEOUT.0;
                }
                timeout_ms - elapsed
            };
            let result = threading::wait_on(w, remaining);
            if result != threading::WaitStatus::WAIT_OBJECT_0.0 {
                return result;
            }
        }
        threading::WaitStatus::WAIT_OBJECT_0.0
    } else {
        let start = std::time::Instant::now();
        loop {
            for (i, w) in waitables.iter().enumerate() {
                if threading::wait_on(w, 0) == threading::WaitStatus::WAIT_OBJECT_0.0 {
                    return threading::WaitStatus(
                        threading::WaitStatus::WAIT_OBJECT_0.0 + i as u32,
                    )
                    .0;
                }
            }
            if timeout_ms != threading::INFINITE {
                let elapsed = start.elapsed().as_millis() as u32;
                if elapsed >= timeout_ms {
                    return threading::WaitStatus::WAIT_TIMEOUT.0;
                }
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn GetEnvironmentVariableA(
    name: *const u8,
    buffer: *mut u8,
    size: u32,
) -> u32 {
    let var_name = match unsafe { read_cstr(name) } {
        Some(n) => n,
        None => return 0,
    };

    match rine_types::environment::get_var(&var_name) {
        Some(val) => unsafe { write_cstr(buffer, size, &val) },
        None => 0,
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn GetEnvironmentVariableW(
    name: *const u16,
    buffer: *mut u16,
    size: u32,
) -> u32 {
    let var_name = match unsafe { read_wstr(name) } {
        Some(n) => n,
        None => return 0,
    };

    match rine_types::environment::get_var(&var_name) {
        Some(val) => unsafe { write_wstr(buffer, size, &val) },
        None => 0,
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn SetEnvironmentVariableA(
    name: *const u8,
    value: *const u8,
) -> WinBool {
    let var_name = match unsafe { read_cstr(name) } {
        Some(n) => n,
        None => return WinBool::FALSE,
    };
    let var_value = unsafe { read_cstr(value) };
    rine_types::environment::set_var(&var_name, var_value.as_deref());
    WinBool::TRUE
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn SetEnvironmentVariableW(
    name: *const u16,
    value: *const u16,
) -> WinBool {
    let var_name = match unsafe { read_wstr(name) } {
        Some(n) => n,
        None => return WinBool::FALSE,
    };
    let var_value = unsafe { read_wstr(value) };
    rine_types::environment::set_var(&var_name, var_value.as_deref());
    WinBool::TRUE
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn ExpandEnvironmentStringsA(
    src: *const u8,
    dst: *mut u8,
    dst_size: u32,
) -> u32 {
    let input = match unsafe { read_cstr(src) } {
        Some(s) => s,
        None => return 0,
    };

    let expanded = rine_types::environment::expand_vars(&input);
    let needed = expanded.len() as u32 + 1;

    if dst.is_null() || dst_size < needed {
        return needed;
    }

    unsafe {
        core::ptr::copy_nonoverlapping(expanded.as_ptr(), dst, expanded.len());
        *dst.add(expanded.len()) = 0;
    }
    needed
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn ExpandEnvironmentStringsW(
    src: *const u16,
    dst: *mut u16,
    dst_size: u32,
) -> u32 {
    let input = match unsafe { read_wstr(src) } {
        Some(s) => s,
        None => return 0,
    };

    let expanded = rine_types::environment::expand_vars(&input);
    let encoded: Vec<u16> = expanded.encode_utf16().collect();
    let needed = encoded.len() as u32 + 1;

    if dst.is_null() || dst_size < needed {
        return needed;
    }

    unsafe {
        core::ptr::copy_nonoverlapping(encoded.as_ptr(), dst, encoded.len());
        *dst.add(encoded.len()) = 0;
    }
    needed
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn GetEnvironmentStringsW() -> *mut u16 {
    ENV_BLOCK_W
        .get_or_init(|| {
            let block = rine_types::environment::build_wide_block();
            let boxed = block.into_boxed_slice();
            SyncPtr(Box::into_raw(boxed) as *mut u16)
        })
        .0
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn FreeEnvironmentStringsW(_block: *mut u16) -> WinBool {
    WinBool::TRUE
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn GetProcessHeap() -> isize {
    DEFAULT_HEAP.as_raw()
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn HeapCreate(
    options: u32,
    _initial_size: usize,
    _maximum_size: usize,
) -> isize {
    let heap = rine_types::handles::HeapState {
        allocations: Mutex::new(HashMap::new()),
        flags: options,
    };
    handle_table().insert(HandleEntry::Heap(heap)).as_raw()
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn HeapDestroy(heap_handle: isize) -> WinBool {
    let handle = Handle::from_raw(heap_handle);
    if heap_handle == DEFAULT_HEAP.as_raw() {
        return WinBool::FALSE;
    }

    match handle_table().remove(handle) {
        Some(HandleEntry::Heap(state)) => {
            let allocs = state.allocations.lock().unwrap();
            for (&addr, &(size, align)) in allocs.iter() {
                if let Ok(layout) = Layout::from_size_align(size, align) {
                    unsafe { std::alloc::dealloc(addr as *mut u8, layout) };
                }
            }
            WinBool::TRUE
        }
        Some(HandleEntry::Window(_)) => WinBool::FALSE,
        Some(other) => {
            let _ = handle_table().insert(other);
            WinBool::FALSE
        }
        None => WinBool::FALSE,
    }
}

fn heap_alloc_inner(heap_handle: isize, flags: u32, size: usize) -> *mut u8 {
    let align = std::mem::align_of::<usize>();
    let layout = match Layout::from_size_align(size, align) {
        Ok(l) => l,
        Err(_) => return std::ptr::null_mut(),
    };

    let ptr = unsafe { std::alloc::alloc(layout) };
    if ptr.is_null() {
        return std::ptr::null_mut();
    }

    if flags & HEAP_ZERO_MEMORY != 0 {
        unsafe { std::ptr::write_bytes(ptr, 0, size) };
    }

    let handle = Handle::from_raw(heap_handle);
    handle_table().with_heap(handle, |state| {
        state
            .allocations
            .lock()
            .unwrap()
            .insert(ptr as usize, (size, align));
    });

    ptr
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn HeapAlloc(heap_handle: isize, flags: u32, size: usize) -> *mut u8 {
    if size == 0 {
        return heap_alloc_inner(heap_handle, flags, 1);
    }
    heap_alloc_inner(heap_handle, flags, size)
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn HeapFree(heap_handle: isize, _flags: u32, ptr: *mut u8) -> WinBool {
    if ptr.is_null() {
        return WinBool::TRUE;
    }

    let handle = Handle::from_raw(heap_handle);
    let removed = handle_table().with_heap(handle, |state| {
        state.allocations.lock().unwrap().remove(&(ptr as usize))
    });

    match removed {
        Some(Some((size, align))) => {
            if let Ok(layout) = Layout::from_size_align(size, align) {
                unsafe { std::alloc::dealloc(ptr, layout) };
            }
            WinBool::TRUE
        }
        _ => WinBool::FALSE,
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn HeapReAlloc(
    heap_handle: isize,
    flags: u32,
    ptr: *mut u8,
    new_size: usize,
) -> *mut u8 {
    if ptr.is_null() {
        return unsafe { HeapAlloc(heap_handle, flags, new_size) };
    }

    let handle = Handle::from_raw(heap_handle);
    let actual_new_size = if new_size == 0 { 1 } else { new_size };
    let old_info = handle_table().with_heap(handle, |state| {
        state
            .allocations
            .lock()
            .unwrap()
            .get(&(ptr as usize))
            .copied()
    });

    let (old_size, old_align) = match old_info {
        Some(Some(info)) => info,
        _ => return std::ptr::null_mut(),
    };

    let old_layout = match Layout::from_size_align(old_size, old_align) {
        Ok(l) => l,
        Err(_) => return std::ptr::null_mut(),
    };

    let new_ptr = unsafe { std::alloc::realloc(ptr, old_layout, actual_new_size) };
    if new_ptr.is_null() {
        return std::ptr::null_mut();
    }

    if flags & HEAP_ZERO_MEMORY != 0 && actual_new_size > old_size {
        unsafe { std::ptr::write_bytes(new_ptr.add(old_size), 0, actual_new_size - old_size) };
    }

    handle_table().with_heap(handle, |state| {
        let mut allocs = state.allocations.lock().unwrap();
        allocs.remove(&(ptr as usize));
        allocs.insert(new_ptr as usize, (actual_new_size, old_align));
    });

    new_ptr
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn VirtualAlloc(
    address: *mut u8,
    size: usize,
    alloc_type: u32,
    protect: u32,
) -> *mut u8 {
    if alloc_type & (MEM_COMMIT | MEM_RESERVE) == 0 {
        return std::ptr::null_mut();
    }

    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize };
    let alloc_size = (size + page_size - 1) & !(page_size - 1);
    if alloc_size == 0 {
        return std::ptr::null_mut();
    }

    let prot = win_protect_to_linux(protect);
    let addr_hint = if address.is_null() {
        std::ptr::null_mut()
    } else {
        address.cast()
    };

    let mut flags = libc::MAP_PRIVATE | libc::MAP_ANONYMOUS;
    if !address.is_null() {
        flags |= libc::MAP_FIXED;
    }

    let result = unsafe { libc::mmap(addr_hint, alloc_size, prot, flags, -1, 0) };
    if result == libc::MAP_FAILED {
        return std::ptr::null_mut();
    }

    let ptr = result as *mut u8;
    VIRTUAL_REGIONS
        .lock()
        .unwrap()
        .insert(ptr as usize, alloc_size);
    ptr
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn VirtualFree(
    address: *mut u8,
    _size: usize,
    free_type: u32,
) -> WinBool {
    if address.is_null() {
        return WinBool::FALSE;
    }

    if free_type & MEM_RELEASE != 0 {
        let region_size = match VIRTUAL_REGIONS.lock().unwrap().remove(&(address as usize)) {
            Some(s) => s,
            None => return WinBool::FALSE,
        };
        let result = unsafe { libc::munmap(address.cast(), region_size) };
        return if result == 0 {
            WinBool::TRUE
        } else {
            WinBool::FALSE
        };
    }

    WinBool::TRUE
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn CreateEventA(
    _security_attrs: usize,
    manual_reset: WinBool,
    initial_state: WinBool,
    _name: *const u8,
) -> isize {
    let waitable = threading::EventWaitable {
        inner: Arc::new(threading::EventInner {
            signaled: Mutex::new(initial_state.is_true()),
            condvar: Condvar::new(),
            manual_reset: manual_reset.is_true(),
        }),
    };
    handle_table().insert(HandleEntry::Event(waitable)).as_raw()
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn SetEvent(event_handle: isize) -> WinBool {
    let h = Handle::from_raw(event_handle);
    let waitable = match handle_table().get_waitable(h) {
        Some(threading::Waitable::Event(e)) => e,
        _ => return WinBool::FALSE,
    };
    let mut signaled = waitable.inner.signaled.lock().unwrap();
    *signaled = true;
    if waitable.inner.manual_reset {
        waitable.inner.condvar.notify_all();
    } else {
        waitable.inner.condvar.notify_one();
    }
    WinBool::TRUE
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn ResetEvent(event_handle: isize) -> WinBool {
    let h = Handle::from_raw(event_handle);
    let waitable = match handle_table().get_waitable(h) {
        Some(threading::Waitable::Event(e)) => e,
        _ => return WinBool::FALSE,
    };
    let mut signaled = waitable.inner.signaled.lock().unwrap();
    *signaled = false;
    WinBool::TRUE
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn CreateMutexA(
    _security_attrs: usize,
    initial_owner: WinBool,
    _name: *const u8,
) -> isize {
    let (owner, count) = if initial_owner.is_true() {
        (Some(std::thread::current().id()), 1)
    } else {
        (None, 0)
    };

    let waitable = threading::MutexWaitable {
        inner: Arc::new(threading::MutexInner {
            state: Mutex::new(threading::MutexState { owner, count }),
            condvar: Condvar::new(),
        }),
    };
    handle_table().insert(HandleEntry::Mutex(waitable)).as_raw()
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn ReleaseMutex(mutex_handle: isize) -> WinBool {
    let h = Handle::from_raw(mutex_handle);
    let waitable = match handle_table().get_waitable(h) {
        Some(threading::Waitable::Mutex(m)) => m,
        _ => return WinBool::FALSE,
    };
    let tid = std::thread::current().id();
    let mut state = waitable.inner.state.lock().unwrap();
    if state.owner != Some(tid) {
        return WinBool::FALSE;
    }
    state.count -= 1;
    if state.count == 0 {
        state.owner = None;
        waitable.inner.condvar.notify_one();
    }
    WinBool::TRUE
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn CreateSemaphoreA(
    _security_attrs: usize,
    initial_count: i32,
    maximum_count: i32,
    _name: *const u8,
) -> isize {
    if maximum_count <= 0 || initial_count < 0 || initial_count > maximum_count {
        return 0;
    }

    let waitable = threading::SemaphoreWaitable {
        inner: Arc::new(threading::SemaphoreInner {
            count: Mutex::new(initial_count),
            max_count: maximum_count,
            condvar: Condvar::new(),
        }),
    };
    handle_table()
        .insert(HandleEntry::Semaphore(waitable))
        .as_raw()
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn ReleaseSemaphore(
    semaphore_handle: isize,
    release_count: i32,
    previous_count: *mut i32,
) -> WinBool {
    if release_count <= 0 {
        return WinBool::FALSE;
    }

    let h = Handle::from_raw(semaphore_handle);
    let waitable = match handle_table().get_waitable(h) {
        Some(threading::Waitable::Semaphore(s)) => s,
        _ => return WinBool::FALSE,
    };

    let mut count = waitable.inner.count.lock().unwrap();
    let prev = *count;

    if prev + release_count > waitable.inner.max_count {
        return WinBool::FALSE;
    }

    if !previous_count.is_null() {
        unsafe { ptr::write(previous_count, prev) };
    }

    *count = prev + release_count;
    for _ in 0..release_count {
        waitable.inner.condvar.notify_one();
    }

    WinBool::TRUE
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
            Export::Func("GetCurrentProcessId", as_win_api!(GetCurrentProcessId)),
            Export::Func("GetCurrentProcess", as_win_api!(GetCurrentProcess)),
            Export::Func("GetExitCodeProcess", as_win_api!(GetExitCodeProcess)),
            Export::Func(
                "SetUnhandledExceptionFilter",
                as_win_api!(SetUnhandledExceptionFilter),
            ),
            Export::Func("CreateFileA", as_win_api!(CreateFileA)),
            Export::Func("CreateFileW", as_win_api!(CreateFileW)),
            Export::Func("ReadFile", as_win_api!(ReadFile)),
            Export::Func("WriteFile", as_win_api!(WriteFile)),
            Export::Func("WriteConsoleA", as_win_api!(WriteConsoleA)),
            Export::Func("WriteConsoleW", as_win_api!(WriteConsoleW)),
            Export::Func("CloseHandle", as_win_api!(CloseHandle)),
            Export::Func("GetStdHandle", as_win_api!(GetStdHandle)),
            Export::Func("GetProcessHeap", as_win_api!(GetProcessHeap)),
            Export::Func("HeapCreate", as_win_api!(HeapCreate)),
            Export::Func("HeapDestroy", as_win_api!(HeapDestroy)),
            Export::Func("HeapAlloc", as_win_api!(HeapAlloc)),
            Export::Func("HeapFree", as_win_api!(HeapFree)),
            Export::Func("HeapReAlloc", as_win_api!(HeapReAlloc)),
            Export::Func("VirtualAlloc", as_win_api!(VirtualAlloc)),
            Export::Func("VirtualFree", as_win_api!(VirtualFree)),
            Export::Func(
                "InitializeCriticalSection",
                as_win_api!(InitializeCriticalSection),
            ),
            Export::Func("EnterCriticalSection", as_win_api!(EnterCriticalSection)),
            Export::Func("LeaveCriticalSection", as_win_api!(LeaveCriticalSection)),
            Export::Func("DeleteCriticalSection", as_win_api!(DeleteCriticalSection)),
            Export::Func("CreateEventA", as_win_api!(CreateEventA)),
            Export::Func("SetEvent", as_win_api!(SetEvent)),
            Export::Func("ResetEvent", as_win_api!(ResetEvent)),
            Export::Func("CreateMutexA", as_win_api!(CreateMutexA)),
            Export::Func("ReleaseMutex", as_win_api!(ReleaseMutex)),
            Export::Func("CreateSemaphoreA", as_win_api!(CreateSemaphoreA)),
            Export::Func("ReleaseSemaphore", as_win_api!(ReleaseSemaphore)),
            Export::Func("LoadLibraryA", as_win_api!(LoadLibraryA)),
            Export::Func("GetProcAddress", as_win_api!(GetProcAddress)),
            Export::Func("FreeLibrary", as_win_api!(FreeLibrary)),
            Export::Func("VirtualProtect", as_win_api!(VirtualProtect)),
            Export::Func("VirtualQuery", as_win_api!(VirtualQuery)),
            Export::Func("TlsAlloc", as_win_api!(TlsAlloc)),
            Export::Func("TlsFree", as_win_api!(TlsFree)),
            Export::Func("TlsGetValue", as_win_api!(TlsGetValue)),
            Export::Func("TlsSetValue", as_win_api!(TlsSetValue)),
            Export::Func("CreateThread", as_win_api!(CreateThread)),
            Export::Func("GetCurrentThread", as_win_api!(GetCurrentThread)),
            Export::Func("GetCurrentThreadId", as_win_api!(GetCurrentThreadId)),
            Export::Func("GetExitCodeThread", as_win_api!(GetExitCodeThread)),
            Export::Func("WaitForSingleObject", as_win_api!(WaitForSingleObject)),
            Export::Func(
                "WaitForMultipleObjects",
                as_win_api!(WaitForMultipleObjects),
            ),
            Export::Func(
                "GetEnvironmentVariableA",
                as_win_api!(GetEnvironmentVariableA),
            ),
            Export::Func(
                "GetEnvironmentVariableW",
                as_win_api!(GetEnvironmentVariableW),
            ),
            Export::Func(
                "SetEnvironmentVariableA",
                as_win_api!(SetEnvironmentVariableA),
            ),
            Export::Func(
                "SetEnvironmentVariableW",
                as_win_api!(SetEnvironmentVariableW),
            ),
            Export::Func(
                "ExpandEnvironmentStringsA",
                as_win_api!(ExpandEnvironmentStringsA),
            ),
            Export::Func(
                "ExpandEnvironmentStringsW",
                as_win_api!(ExpandEnvironmentStringsW),
            ),
            Export::Func(
                "GetEnvironmentStringsW",
                as_win_api!(GetEnvironmentStringsW),
            ),
            Export::Func(
                "FreeEnvironmentStringsW",
                as_win_api!(FreeEnvironmentStringsW),
            ),
            Export::Func("Sleep", as_win_api!(Sleep)),
        ]
    }
}
