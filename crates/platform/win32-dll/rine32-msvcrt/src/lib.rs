//! MSVCRT (Microsoft Visual C Runtime) support for 32-bit Windows DLLs.
//!
//! This crate provides a 32-bit implementation of the MSVCRT DLL by forwarding
//! calls to shared logic in `rine-common-msvcrt`. Functions are organized into modules:
//! - `crt_init`: CRT initialization (__getmainargs, _initterm)
//! - `crt_support`: Exception handling, signal, locks, file descriptor tables
//! - `memory`: malloc, calloc, realloc, free
//! - `stdlib`: exit, _cexit
//! - `string`: string and memory operations
//! - `stdio`: formatted I/O

use rine_common_msvcrt::{commode_ptr, fake_iob_32_ptr, fmode_ptr, initenv_ptr};
use rine_dlls::{DllPlugin, Export, StubExport, as_win_api};

#[cfg(not(target_pointer_width = "32"))]
compile_error!(
    "crate `rine32-msvcrt` must be built for a 32-bit target (for example: --target i686-unknown-linux-gnu)"
);

mod crt_init;
mod crt_support;
mod memory;
mod stdio;
mod stdlib;
mod string;

pub use crt_init::{__getmainargs, _initterm, _initterm_e};
pub use crt_support::{
    __C_specific_handler, __iob_func, __p__commode, __p__environ, __p__fmode, __set_app_type,
    __setusermatherr, _amsg_exit, _errno, _lock, _onexit, _unlock, abort, signal,
};
pub use memory::{calloc, free, malloc, realloc};
pub use stdio::{fprintf, fwrite, printf, puts, vfprintf};
pub use stdlib::{_cexit, exit};
pub use string::{memcpy, memset, strcmp, strlen, strncmp};

pub struct MsvcrtPlugin32;
pub struct CrtForwarderPlugin32;

impl DllPlugin for MsvcrtPlugin32 {
    fn dll_names(&self) -> &[&str] {
        &["msvcrt.dll"]
    }

    fn exports(&self) -> Vec<Export> {
        vec![
            // stdio
            Export::Func("printf", as_win_api!(printf)),
            Export::Func("puts", as_win_api!(puts)),
            Export::Func("fprintf", as_win_api!(fprintf)),
            Export::Func("vfprintf", as_win_api!(vfprintf)),
            Export::Func("fwrite", as_win_api!(fwrite)),
            // stdlib
            Export::Func("exit", as_win_api!(exit)),
            Export::Func("_cexit", as_win_api!(_cexit)),
            // crt_init
            Export::Func("__getmainargs", as_win_api!(__getmainargs)),
            Export::Func("_initterm", as_win_api!(_initterm)),
            Export::Func("_initterm_e", as_win_api!(_initterm_e)),
            // crt_support — functions
            Export::Func("__iob_func", as_win_api!(__iob_func)),
            Export::Func("abort", as_win_api!(abort)),
            Export::Func("_lock", as_win_api!(_lock)),
            Export::Func("_unlock", as_win_api!(_unlock)),
            Export::Func("_errno", as_win_api!(_errno)),
            Export::Func("__p__environ", as_win_api!(__p__environ)),
            Export::Func("__p__fmode", as_win_api!(__p__fmode)),
            Export::Func("__p__commode", as_win_api!(crt_support::__p__commode)),
            // crt_support — data exports
            Export::Data("_commode", commode_ptr() as *const ()),
            Export::Data("_fmode", fmode_ptr() as *const ()),
            Export::Data("_iob", fake_iob_32_ptr() as *const ()),
            Export::Data("__initenv", initenv_ptr() as *const ()),
            // memory
            Export::Func("malloc", as_win_api!(malloc)),
            Export::Func("calloc", as_win_api!(calloc)),
            Export::Func("realloc", as_win_api!(realloc)),
            Export::Func("free", as_win_api!(free)),
            Export::Func("memcpy", as_win_api!(memcpy)),
            Export::Func("memset", as_win_api!(memset)),
            // string
            Export::Func("strlen", as_win_api!(strlen)),
            Export::Func("strcmp", as_win_api!(strcmp)),
            Export::Func("strncmp", as_win_api!(strncmp)),
        ]
    }

    fn stubs(&self) -> Vec<StubExport> {
        vec![
            // crt_support — functions
            StubExport {
                name: "__set_app_type",
                func: as_win_api!(crt_support::__set_app_type),
            },
            StubExport {
                name: "__setusermatherr",
                func: as_win_api!(crt_support::__setusermatherr),
            },
            StubExport {
                name: "__C_specific_handler",
                func: as_win_api!(crt_support::__C_specific_handler),
            },
            StubExport {
                name: "_onexit",
                func: as_win_api!(crt_support::_onexit),
            },
            StubExport {
                name: "_amsg_exit",
                func: as_win_api!(crt_support::_amsg_exit),
            },
            StubExport {
                name: "signal",
                func: as_win_api!(crt_support::signal),
            },
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

    fn stubs(&self) -> Vec<StubExport> {
        vec![
            // crt_support — functions
            StubExport {
                name: "__set_app_type",
                func: as_win_api!(crt_support::__set_app_type),
            },
            StubExport {
                name: "__setusermatherr",
                func: as_win_api!(crt_support::__setusermatherr),
            },
        ]
    }
}
