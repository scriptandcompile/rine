use std::ffi::c_void;
use std::sync::LazyLock;

use rine_common_msvcrt::{
    AllocationTracker, abort_process, amsg_exit, c_specific_handler_result, cached_main_args,
    commode_ptr, errno_location, fake_iob_32_ptr, fmode_ptr, initenv_ptr, lock, onexit,
    run_initterm, run_initterm_e, set_app_type, set_usermatherr, signal_default, unlock,
};
use rine_dlls::{DllPlugin, Export, as_win_api, win32_stub};

#[cfg(not(target_pointer_width = "32"))]
compile_error!(
    "crate `rine32-msvcrt` must be built for a 32-bit target (for example: --target i686-unknown-linux-gnu)"
);

pub struct MsvcrtPlugin32;
pub struct CrtForwarderPlugin32;

win32_stub!(printf, "msvcrt");
win32_stub!(puts, "msvcrt");
win32_stub!(fprintf, "msvcrt");
win32_stub!(vfprintf, "msvcrt");
win32_stub!(fwrite, "msvcrt");

static CRT_ALLOCATIONS: LazyLock<AllocationTracker> = LazyLock::new(AllocationTracker::new);

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn exit(code: i32) {
    rine_types::dev_notify!(on_process_exiting(code));
    std::process::exit(code);
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn _cexit() {
    unsafe { libc::fflush(std::ptr::null_mut()) };
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __getmainargs(
    p_argc: *mut i32,
    p_argv: *mut *mut *mut i8,
    p_envp: *mut *mut *mut i8,
    _do_wildcard: i32,
    _start_info: *mut core::ffi::c_void,
) -> i32 {
    let args = cached_main_args();

    if !p_argc.is_null() {
        unsafe { *p_argc = args.argc() };
    }
    if !p_argv.is_null() {
        unsafe { *p_argv = args.argv_ptr() };
    }
    if !p_envp.is_null() {
        unsafe { *p_envp = args.envp_ptr() };
    }

    0
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn _initterm(
    start: *const Option<unsafe extern "C" fn()>,
    end: *const Option<unsafe extern "C" fn()>,
) {
    unsafe {
        run_initterm(start, end, |func| {
            func();
        });
    }
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn _initterm_e(
    start: *const Option<unsafe extern "C" fn() -> i32>,
    end: *const Option<unsafe extern "C" fn() -> i32>,
) -> i32 {
    let result = unsafe { run_initterm_e(start, end, |func| func()) };
    if result != 0 {
        tracing::warn!(result, "msvcrt::_initterm_e: initializer failed");
    }
    result
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __set_app_type(app_type: i32) {
    set_app_type(app_type);
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __setusermatherr(handler: usize) {
    set_usermatherr(handler);
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "C" fn __C_specific_handler(
    _exception_record: usize,
    _establisher_frame: usize,
    _context_record: usize,
    _dispatcher_context: usize,
) -> i32 {
    c_specific_handler_result()
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __iob_func() -> *mut u8 {
    fake_iob_32_ptr()
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn _onexit(func: usize) -> usize {
    onexit(func)
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn _amsg_exit(msg_num: i32) {
    amsg_exit(msg_num)
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn abort() {
    abort_process()
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn signal(sig: i32, handler: usize) -> usize {
    signal_default(sig, handler)
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn _lock(locknum: i32) {
    lock(locknum);
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn _unlock(locknum: i32) {
    unlock(locknum);
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn _errno() -> *mut i32 {
    errno_location()
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "C" fn __p__environ() -> *const *const *const i8 {
    initenv_ptr() as *const *const *const i8
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "C" fn __p__fmode() -> *mut i32 {
    fmode_ptr()
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "C" fn __p__commode() -> *mut i32 {
    commode_ptr()
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn malloc(size: usize) -> *mut c_void {
    let ptr = unsafe { libc::malloc(size) };
    if !ptr.is_null() {
        CRT_ALLOCATIONS.record(ptr, size);
        rine_types::dev_notify!(on_memory_allocated(ptr as u64, size as u64, "malloc"));
    }
    ptr
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn calloc(count: usize, size: usize) -> *mut c_void {
    let ptr = unsafe { libc::calloc(count, size) };
    if !ptr.is_null() {
        let total = count.saturating_mul(size);
        CRT_ALLOCATIONS.record(ptr, total);
        rine_types::dev_notify!(on_memory_allocated(ptr as u64, total as u64, "calloc"));
    }
    ptr
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void {
    let old_size = CRT_ALLOCATIONS.forget(ptr);

    let new_ptr = unsafe { libc::realloc(ptr, size) };
    if new_ptr.is_null() {
        if let Some(sz) = old_size {
            CRT_ALLOCATIONS.restore(ptr, sz);
        }
        return new_ptr;
    }

    if let Some(sz) = old_size {
        rine_types::dev_notify!(on_memory_freed(ptr as u64, sz as u64, "realloc"));
    }
    CRT_ALLOCATIONS.record(new_ptr, size);
    rine_types::dev_notify!(on_memory_allocated(new_ptr as u64, size as u64, "realloc"));
    new_ptr
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn free(ptr: *mut c_void) {
    if let Some(sz) = CRT_ALLOCATIONS.forget(ptr) {
        rine_types::dev_notify!(on_memory_freed(ptr as u64, sz as u64, "free"));
    }
    unsafe { libc::free(ptr) };
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void {
    unsafe { libc::memcpy(dest, src, n) }
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn memset(dest: *mut c_void, c: i32, n: usize) -> *mut c_void {
    unsafe { libc::memset(dest, c, n) }
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn strlen(s: *const i8) -> usize {
    if s.is_null() {
        return 0;
    }
    unsafe { libc::strlen(s) }
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn strcmp(lhs: *const i8, rhs: *const i8) -> i32 {
    unsafe { libc::strcmp(lhs, rhs) }
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn strncmp(lhs: *const i8, rhs: *const i8, n: usize) -> i32 {
    unsafe { libc::strncmp(lhs, rhs, n) }
}

impl DllPlugin for MsvcrtPlugin32 {
    fn dll_names(&self) -> &[&str] {
        &["msvcrt.dll"]
    }

    fn exports(&self) -> Vec<Export> {
        vec![
            Export::Func("printf", as_win_api!(printf)),
            Export::Func("puts", as_win_api!(puts)),
            Export::Func("fprintf", as_win_api!(fprintf)),
            Export::Func("vfprintf", as_win_api!(vfprintf)),
            Export::Func("fwrite", as_win_api!(fwrite)),
            Export::Func("exit", as_win_api!(exit)),
            Export::Func("_cexit", as_win_api!(_cexit)),
            Export::Func("__getmainargs", as_win_api!(__getmainargs)),
            Export::Func("_initterm", as_win_api!(_initterm)),
            Export::Func("_initterm_e", as_win_api!(_initterm_e)),
            Export::Func("__set_app_type", as_win_api!(__set_app_type)),
            Export::Func("__setusermatherr", as_win_api!(__setusermatherr)),
            Export::Func("__C_specific_handler", as_win_api!(__C_specific_handler)),
            Export::Func("__iob_func", as_win_api!(__iob_func)),
            Export::Func("_onexit", as_win_api!(_onexit)),
            Export::Func("_amsg_exit", as_win_api!(_amsg_exit)),
            Export::Func("abort", as_win_api!(abort)),
            Export::Func("signal", as_win_api!(signal)),
            Export::Func("_lock", as_win_api!(_lock)),
            Export::Func("_unlock", as_win_api!(_unlock)),
            Export::Func("_errno", as_win_api!(_errno)),
            Export::Func("__p__environ", as_win_api!(__p__environ)),
            Export::Func("__p__fmode", as_win_api!(__p__fmode)),
            Export::Func("__p__commode", as_win_api!(__p__commode)),
            Export::Data("_commode", commode_ptr() as *const ()),
            Export::Data("_fmode", fmode_ptr() as *const ()),
            Export::Data("_iob", fake_iob_32_ptr() as *const ()),
            Export::Data("__initenv", initenv_ptr() as *const ()),
            Export::Func("malloc", as_win_api!(malloc)),
            Export::Func("calloc", as_win_api!(calloc)),
            Export::Func("realloc", as_win_api!(realloc)),
            Export::Func("free", as_win_api!(free)),
            Export::Func("memcpy", as_win_api!(memcpy)),
            Export::Func("memset", as_win_api!(memset)),
            Export::Func("strlen", as_win_api!(strlen)),
            Export::Func("strcmp", as_win_api!(strcmp)),
            Export::Func("strncmp", as_win_api!(strncmp)),
        ]
    }
}

impl DllPlugin for CrtForwarderPlugin32 {
    fn dll_names(&self) -> &[&str] {
        &[
            "api-ms-win-crt-runtime-l1-1-0.dll",
            "api-ms-win-crt-stdio-l1-1-0.dll",
            "api-ms-win-crt-math-l1-1-0.dll",
            "api-ms-win-crt-locale-l1-1-0.dll",
            "api-ms-win-crt-heap-l1-1-0.dll",
            "api-ms-win-crt-string-l1-1-0.dll",
            "api-ms-win-crt-convert-l1-1-0.dll",
            "api-ms-win-crt-environment-l1-1-0.dll",
            "api-ms-win-crt-time-l1-1-0.dll",
            "api-ms-win-crt-filesystem-l1-1-0.dll",
            "api-ms-win-crt-utility-l1-1-0.dll",
            "vcruntime140.dll",
        ]
    }

    fn exports(&self) -> Vec<Export> {
        vec![
            Export::Func("printf", as_win_api!(printf)),
            Export::Func("puts", as_win_api!(puts)),
            Export::Func("exit", as_win_api!(exit)),
            Export::Func("_cexit", as_win_api!(_cexit)),
            Export::Func("_initterm", as_win_api!(_initterm)),
            Export::Func("_initterm_e", as_win_api!(_initterm_e)),
        ]
    }
}
