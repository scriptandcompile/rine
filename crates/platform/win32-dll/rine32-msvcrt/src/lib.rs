use std::sync::OnceLock;

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
win32_stub!(exit, "msvcrt");
win32_stub!(_cexit, "msvcrt");
win32_stub!(__getmainargs, "msvcrt");
win32_stub!(_initterm, "msvcrt");
win32_stub!(_initterm_e, "msvcrt");
win32_stub!(__set_app_type, "msvcrt");
win32_stub!(__setusermatherr, "msvcrt");
win32_stub!(__C_specific_handler, "msvcrt");
win32_stub!(__iob_func, "msvcrt");
win32_stub!(_onexit, "msvcrt");
win32_stub!(_amsg_exit, "msvcrt");
win32_stub!(abort, "msvcrt");
win32_stub!(signal, "msvcrt");
win32_stub!(_lock, "msvcrt");
win32_stub!(_unlock, "msvcrt");
win32_stub!(_errno, "msvcrt");
win32_stub!(__p__environ, "msvcrt");
win32_stub!(__p__fmode, "msvcrt");
win32_stub!(__p__commode, "msvcrt");
win32_stub!(malloc, "msvcrt");
win32_stub!(calloc, "msvcrt");
win32_stub!(realloc, "msvcrt");
win32_stub!(free, "msvcrt");
win32_stub!(memcpy, "msvcrt");
win32_stub!(memset, "msvcrt");
win32_stub!(strlen, "msvcrt");
win32_stub!(strcmp, "msvcrt");
win32_stub!(strncmp, "msvcrt");

fn leaked_i32(initial: i32) -> *mut i32 {
    Box::into_raw(Box::new(initial))
}

fn leaked_usize(initial: usize) -> *mut usize {
    Box::into_raw(Box::new(initial))
}

fn fmode_cell() -> *mut i32 {
    static CELL: OnceLock<usize> = OnceLock::new();
    let ptr = *CELL.get_or_init(|| leaked_i32(0) as usize);
    ptr as *mut i32
}

fn commode_cell() -> *mut i32 {
    static CELL: OnceLock<usize> = OnceLock::new();
    let ptr = *CELL.get_or_init(|| leaked_i32(0) as usize);
    ptr as *mut i32
}

fn initenv_cell() -> *mut usize {
    static CELL: OnceLock<usize> = OnceLock::new();
    let ptr = *CELL.get_or_init(|| leaked_usize(0) as usize);
    ptr as *mut usize
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
            Export::Data("_commode", commode_cell() as *const ()),
            Export::Data("_fmode", fmode_cell() as *const ()),
            Export::Data("__initenv", initenv_cell() as *const ()),
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
