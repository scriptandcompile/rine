pub mod crt_init;
pub mod crt_support;
pub mod memory;
pub mod stdio;
pub mod stdlib;
pub mod string;

use rine_dlls::{DllPlugin, Export, StubExport, as_win_api};

/// Primary msvcrt.dll plugin.
pub struct MsvcrtPlugin;

impl DllPlugin for MsvcrtPlugin {
    fn dll_names(&self) -> &[&str] {
        &["msvcrt.dll"]
    }

    fn exports(&self) -> Vec<Export> {
        vec![
            // stdio
            Export::Func("printf", as_win_api!(stdio::printf)),
            Export::Func("puts", as_win_api!(stdio::puts)),
            Export::Func("fprintf", as_win_api!(stdio::fprintf)),
            Export::Func("vfprintf", as_win_api!(stdio::vfprintf)),
            Export::Func("fwrite", as_win_api!(stdio::fwrite)),
            // stdlib
            Export::Func("exit", as_win_api!(stdlib::exit)),
            Export::Func("_cexit", as_win_api!(stdlib::_cexit)),
            // crt_init
            Export::Func("__getmainargs", as_win_api!(crt_init::__getmainargs)),
            Export::Func("_initterm", as_win_api!(crt_init::_initterm)),
            Export::Func("_initterm_e", as_win_api!(crt_init::_initterm_e)),
            // crt_support — functions
            Export::Func(
                "__setusermatherr",
                as_win_api!(crt_support::__setusermatherr),
            ),
            Export::Func(
                "__C_specific_handler",
                as_win_api!(crt_support::__C_specific_handler),
            ),
            Export::Func("__iob_func", as_win_api!(crt_support::__iob_func)),
            Export::Func("_onexit", as_win_api!(crt_support::_onexit)),
            Export::Func("_amsg_exit", as_win_api!(crt_support::_amsg_exit)),
            Export::Func("abort", as_win_api!(crt_support::abort)),
            Export::Func("signal", as_win_api!(crt_support::signal)),
            Export::Func("_lock", as_win_api!(crt_support::_lock)),
            Export::Func("_unlock", as_win_api!(crt_support::_unlock)),
            Export::Func("_errno", as_win_api!(crt_support::_errno)),
            Export::Func("__p__environ", as_win_api!(crt_support::__p__environ)),
            Export::Func("__p__fmode", as_win_api!(crt_support::__p__fmode)),
            Export::Func("__p__commode", as_win_api!(crt_support::__p__commode)),
            // crt_support — data exports
            Export::Data("_commode", crt_support::commode_data_ptr() as *const ()),
            Export::Data("_fmode", crt_support::fmode_data_ptr() as *const ()),
            Export::Data("__initenv", crt_support::initenv_data_ptr() as *const ()),
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
        ]
    }
}

/// CRT API-set forwarder plugin. Registers the same functions under the
/// `api-ms-win-crt-*` DLL names used by MinGW-w64 and UCRT-based executables.
pub struct CrtForwarderPlugin;

impl DllPlugin for CrtForwarderPlugin {
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
        ]
    }

    fn stubs(&self) -> Vec<StubExport> {
        vec![
            // crt_support — functions
            StubExport {
                name: "__set_app_type",
                func: as_win_api!(crt_support::__set_app_type),
            },
        ]
    }
}
