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

use rine_dlls::{DllPlugin, Export, PartialExport, StubExport, as_win_api};

#[cfg(not(target_pointer_width = "32"))]
compile_error!(
    "crate `rine32-msvcrt` must be built for a 32-bit target (for example: --target i686-unknown-linux-gnu)"
);

pub mod crt_init;
pub mod crt_support;
pub mod memory;
pub mod stdio;
pub mod stdlib;
pub mod string;

pub struct MsvcrtPlugin32;
pub struct CrtForwarderPlugin32;

impl DllPlugin for MsvcrtPlugin32 {
    fn dll_names(&self) -> &[&str] {
        &["msvcrt.dll"]
    }

    fn exports(&self) -> Vec<Export> {
        vec![
            // stdio
            Export::Func("printf", as_win_api!(stdio::printf)),
            Export::Func("puts", as_win_api!(stdio::puts)),
            Export::Func("fwrite", as_win_api!(stdio::fwrite)),
            Export::Func("_cexit", as_win_api!(stdlib::_cexit)),
            // stdlib
            Export::Func("exit", as_win_api!(stdlib::exit)),
            // crt_init
            Export::Func("_initterm", as_win_api!(crt_init::_initterm)),
            Export::Func("_initterm_e", as_win_api!(crt_init::_initterm_e)),
            // crt_support — functions
            Export::Func("__iob_func", as_win_api!(crt_support::__iob_func)),
            Export::Func("abort", as_win_api!(crt_support::abort)),
            Export::Func("_lock", as_win_api!(crt_support::_lock)),
            Export::Func("_unlock", as_win_api!(crt_support::_unlock)),
            Export::Func("_errno", as_win_api!(crt_support::_errno)),
            Export::Func("__p__environ", as_win_api!(crt_support::__p__environ)),
            Export::Func("__p__fmode", as_win_api!(crt_support::__p__fmode)),
            Export::Func("__p__commode", as_win_api!(crt_support::__p__commode)),
            // crt_support — data exports
            Export::Data("__initenv", unsafe {
                crt_support::__initenv() as *const ()
            }),
            Export::Data("_commode", unsafe { crt_support::_commode() as *const () }),
            Export::Data("_fmode", unsafe { crt_support::_fmode() as *const () }),
            Export::Data("_iob", unsafe { crt_support::_iob() as *const () }),
            // memory
            Export::Func("malloc", as_win_api!(memory::malloc)),
            Export::Func("calloc", as_win_api!(memory::calloc)),
            Export::Func("realloc", as_win_api!(memory::realloc)),
            Export::Func("free", as_win_api!(memory::free)),
            Export::Func("memcpy", as_win_api!(memory::memcpy)),
            Export::Func("memset", as_win_api!(memory::memset)),
            // string
            Export::Func("strlen", as_win_api!(string::strlen)),
            Export::Func("strcmp", as_win_api!(string::strcmp)),
            Export::Func("strncmp", as_win_api!(string::strncmp)),
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

    fn partials(&self) -> Vec<PartialExport> {
        vec![
            // crt_init
            PartialExport {
                name: "__getmainargs",
                func: as_win_api!(crt_init::__getmainargs),
            },
            // stdio
            PartialExport {
                name: "fprintf",
                func: as_win_api!(stdio::fprintf),
            },
            PartialExport {
                name: "vfprintf",
                func: as_win_api!(stdio::vfprintf),
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
            Export::Func("printf", as_win_api!(stdio::printf)),
            Export::Func("puts", as_win_api!(stdio::puts)),
            Export::Func("exit", as_win_api!(stdlib::exit)),
            Export::Func("_cexit", as_win_api!(stdlib::_cexit)),
            Export::Func("_initterm", as_win_api!(crt_init::_initterm)),
            Export::Func("_initterm_e", as_win_api!(crt_init::_initterm_e)),
            // crt_support — data exports
            Export::Data("__initenv", unsafe {
                crt_support::__initenv() as *const ()
            }),
            Export::Data("_commode", unsafe { crt_support::_commode() as *const () }),
            Export::Data("_fmode", unsafe { crt_support::_fmode() as *const () }),
            Export::Data("_iob", unsafe { crt_support::_iob() as *const () }),
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
