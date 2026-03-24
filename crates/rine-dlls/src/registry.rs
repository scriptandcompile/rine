//! DLL function registry — maps Windows DLL names and function names/ordinals
//! to Rust function pointers that implement the corresponding Windows API.

use std::collections::HashMap;

/// A function pointer stored in the registry, castable to the appropriate signature.
///
/// Uses `extern "win64"` because PE code calls through the IAT using
/// the Windows x64 calling convention.
pub type WinApiFunc = unsafe extern "win64" fn();

/// Type-erase a function pointer to `WinApiFunc` for registry storage.
///
/// The PE code calls through the IAT with the correct Windows x64 calling
/// convention, so the true signature is recovered at call-site.
macro_rules! as_win_api {
    ($f:expr) => {
        // SAFETY: all function pointers are pointer-sized. The PE caller
        // will invoke through the IAT with the matching argument layout.
        unsafe { core::mem::transmute::<*const (), WinApiFunc>($f as *const ()) }
    };
}

/// Holds the function lookup tables for all reimplemented DLLs.
///
/// DLL names are normalized to lowercase. Function names are stored as-is
/// (Windows API names are case-sensitive).
pub struct DllRegistry {
    /// Map from lowercase DLL name → per-DLL function table.
    dlls: HashMap<String, DllModule>,
}

/// A single reimplemented DLL module with its exported functions.
struct DllModule {
    /// Map from function name → function pointer.
    by_name: HashMap<&'static str, WinApiFunc>,
    /// Map from ordinal → function pointer.
    by_ordinal: HashMap<u16, WinApiFunc>,
}

impl DllModule {
    fn new() -> Self {
        Self {
            by_name: HashMap::new(),
            by_ordinal: HashMap::new(),
        }
    }

    fn register_name(&mut self, name: &'static str, func: WinApiFunc) {
        self.by_name.insert(name, func);
    }

    fn register_ordinal(&mut self, ordinal: u16, func: WinApiFunc) {
        self.by_ordinal.insert(ordinal, func);
    }
}

/// Result of looking up a single import.
#[derive(Debug, Clone, Copy)]
pub enum LookupResult {
    /// Found a Rust implementation for this import.
    Found(WinApiFunc),
    /// No implementation exists; a stub was returned that will log and abort.
    Stub(WinApiFunc),
}

impl LookupResult {
    /// Get the function pointer regardless of whether it's a real implementation or stub.
    pub fn as_ptr(self) -> WinApiFunc {
        match self {
            LookupResult::Found(f) | LookupResult::Stub(f) => f,
        }
    }
}

impl DllRegistry {
    /// Build the registry with all currently implemented DLL functions.
    pub fn new() -> Self {
        let mut reg = Self {
            dlls: HashMap::new(),
        };
        reg.register_all();
        reg
    }

    /// Look up a function by DLL name and function name.
    pub fn resolve_by_name(&self, dll: &str, name: &str) -> LookupResult {
        let key = normalize_dll_name(dll);
        if let Some(module) = self.dlls.get(key.as_str()) {
            if let Some(&func) = module.by_name.get(name) {
                return LookupResult::Found(func);
            }
        }
        LookupResult::Stub(stub_function)
    }

    /// Look up a function by DLL name and ordinal number.
    pub fn resolve_by_ordinal(&self, dll: &str, ordinal: u16) -> LookupResult {
        let key = normalize_dll_name(dll);
        if let Some(module) = self.dlls.get(key.as_str()) {
            if let Some(&func) = module.by_ordinal.get(&ordinal) {
                return LookupResult::Found(func);
            }
        }
        LookupResult::Stub(stub_function)
    }

    /// Returns the list of DLL names this registry knows about.
    pub fn known_dlls(&self) -> Vec<&str> {
        self.dlls.keys().map(|s| s.as_str()).collect()
    }

    /// Returns true if the registry has any implementation for the given DLL.
    pub fn has_dll(&self, dll: &str) -> bool {
        let key = normalize_dll_name(dll);
        self.dlls.contains_key(key.as_str())
    }

    // ------------------------------------------------------------------
    // Internal registration
    // ------------------------------------------------------------------

    fn get_or_create_module(&mut self, dll_name: &str) -> &mut DllModule {
        let key = normalize_dll_name(dll_name);
        self.dlls.entry(key).or_insert_with(DllModule::new)
    }

    fn register_func(&mut self, dll: &str, name: &'static str, func: WinApiFunc) {
        self.get_or_create_module(dll).register_name(name, func);
    }

    /// Register a data export. The IAT slot will contain the raw address
    /// (e.g. pointer to a variable), not a function pointer. The PE code
    /// reads this value directly as a pointer.
    fn register_data(&mut self, dll: &str, name: &'static str, addr: *const ()) {
        let func = unsafe { core::mem::transmute::<*const (), WinApiFunc>(addr) };
        self.get_or_create_module(dll).register_name(name, func);
    }

    fn register_func_ordinal(&mut self, dll: &str, ordinal: u16, func: WinApiFunc) {
        self.get_or_create_module(dll)
            .register_ordinal(ordinal, func);
    }

    /// Register all currently implemented DLL functions.
    /// As DLL stubs are filled in with real implementations, add them here.
    fn register_all(&mut self) {
        // Placeholder modules so the resolver can distinguish "known DLL,
        // unimplemented function" from "completely unknown DLL".
        for dll in &[
            "ntdll.dll",
            "kernel32.dll",
            "msvcrt.dll",
            "advapi32.dll",
            "user32.dll",
            "gdi32.dll",
            "ws2_32.dll",
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
        ] {
            self.get_or_create_module(dll);
        }

        // ----- ntdll.dll -----
        self.register_func(
            "ntdll.dll",
            "NtWriteFile",
            as_win_api!(crate::ntdll::file::NtWriteFile),
        );
        self.register_func(
            "ntdll.dll",
            "NtTerminateProcess",
            as_win_api!(crate::ntdll::process::NtTerminateProcess),
        );
        self.register_func(
            "ntdll.dll",
            "RtlInitUnicodeString",
            as_win_api!(crate::ntdll::rtl::RtlInitUnicodeString),
        );

        // ----- msvcrt.dll -----
        self.register_func(
            "msvcrt.dll",
            "printf",
            as_win_api!(crate::msvcrt::stdio::printf),
        );
        self.register_func(
            "msvcrt.dll",
            "puts",
            as_win_api!(crate::msvcrt::stdio::puts),
        );
        self.register_func(
            "msvcrt.dll",
            "fprintf",
            as_win_api!(crate::msvcrt::stdio::fprintf),
        );
        self.register_func(
            "msvcrt.dll",
            "vfprintf",
            as_win_api!(crate::msvcrt::stdio::vfprintf),
        );
        self.register_func(
            "msvcrt.dll",
            "fwrite",
            as_win_api!(crate::msvcrt::stdio::fwrite),
        );
        self.register_func(
            "msvcrt.dll",
            "exit",
            as_win_api!(crate::msvcrt::stdlib::exit),
        );
        self.register_func(
            "msvcrt.dll",
            "_cexit",
            as_win_api!(crate::msvcrt::stdlib::_cexit),
        );
        self.register_func(
            "msvcrt.dll",
            "__getmainargs",
            as_win_api!(crate::msvcrt::crt_init::__getmainargs),
        );
        self.register_func(
            "msvcrt.dll",
            "_initterm",
            as_win_api!(crate::msvcrt::crt_init::_initterm),
        );
        self.register_func(
            "msvcrt.dll",
            "_initterm_e",
            as_win_api!(crate::msvcrt::crt_init::_initterm_e),
        );
        self.register_func(
            "msvcrt.dll",
            "__set_app_type",
            as_win_api!(crate::msvcrt::crt_support::__set_app_type),
        );
        self.register_func(
            "msvcrt.dll",
            "__setusermatherr",
            as_win_api!(crate::msvcrt::crt_support::__setusermatherr),
        );
        self.register_func(
            "msvcrt.dll",
            "__C_specific_handler",
            as_win_api!(crate::msvcrt::crt_support::__C_specific_handler),
        );
        self.register_data(
            "msvcrt.dll",
            "_commode",
            crate::msvcrt::crt_support::commode_data_ptr() as *const (),
        );
        self.register_data(
            "msvcrt.dll",
            "_fmode",
            crate::msvcrt::crt_support::fmode_data_ptr() as *const (),
        );
        self.register_data(
            "msvcrt.dll",
            "__initenv",
            crate::msvcrt::crt_support::initenv_data_ptr() as *const (),
        );
        self.register_func(
            "msvcrt.dll",
            "__iob_func",
            as_win_api!(crate::msvcrt::crt_support::__iob_func),
        );
        self.register_func(
            "msvcrt.dll",
            "_onexit",
            as_win_api!(crate::msvcrt::crt_support::_onexit),
        );
        self.register_func(
            "msvcrt.dll",
            "_amsg_exit",
            as_win_api!(crate::msvcrt::crt_support::_amsg_exit),
        );
        self.register_func(
            "msvcrt.dll",
            "abort",
            as_win_api!(crate::msvcrt::crt_support::abort),
        );
        self.register_func(
            "msvcrt.dll",
            "signal",
            as_win_api!(crate::msvcrt::crt_support::signal),
        );
        self.register_func(
            "msvcrt.dll",
            "_lock",
            as_win_api!(crate::msvcrt::crt_support::_lock),
        );
        self.register_func(
            "msvcrt.dll",
            "_unlock",
            as_win_api!(crate::msvcrt::crt_support::_unlock),
        );
        self.register_func(
            "msvcrt.dll",
            "_errno",
            as_win_api!(crate::msvcrt::crt_support::_errno),
        );
        self.register_func(
            "msvcrt.dll",
            "__p__environ",
            as_win_api!(crate::msvcrt::crt_support::__p__environ),
        );
        self.register_func(
            "msvcrt.dll",
            "__p__fmode",
            as_win_api!(crate::msvcrt::crt_support::__p__fmode),
        );
        self.register_func(
            "msvcrt.dll",
            "__p__commode",
            as_win_api!(crate::msvcrt::crt_support::__p__commode),
        );
        self.register_func(
            "msvcrt.dll",
            "malloc",
            as_win_api!(crate::msvcrt::memory::malloc),
        );
        self.register_func(
            "msvcrt.dll",
            "calloc",
            as_win_api!(crate::msvcrt::memory::calloc),
        );
        self.register_func(
            "msvcrt.dll",
            "realloc",
            as_win_api!(crate::msvcrt::memory::realloc),
        );
        self.register_func(
            "msvcrt.dll",
            "free",
            as_win_api!(crate::msvcrt::memory::free),
        );
        self.register_func(
            "msvcrt.dll",
            "memcpy",
            as_win_api!(crate::msvcrt::memory::memcpy),
        );
        self.register_func(
            "msvcrt.dll",
            "memset",
            as_win_api!(crate::msvcrt::memory::memset),
        );
        self.register_func(
            "msvcrt.dll",
            "strlen",
            as_win_api!(crate::msvcrt::string::strlen),
        );
        self.register_func(
            "msvcrt.dll",
            "strncmp",
            as_win_api!(crate::msvcrt::string::strncmp),
        );

        // Register the same functions under CRT forwarder DLL names used
        // by MinGW-w64 and UCRT-based executables.
        self.register_func(
            "api-ms-win-crt-stdio-l1-1-0.dll",
            "printf",
            as_win_api!(crate::msvcrt::stdio::printf),
        );
        self.register_func(
            "api-ms-win-crt-stdio-l1-1-0.dll",
            "puts",
            as_win_api!(crate::msvcrt::stdio::puts),
        );
        self.register_func(
            "api-ms-win-crt-runtime-l1-1-0.dll",
            "exit",
            as_win_api!(crate::msvcrt::stdlib::exit),
        );
        self.register_func(
            "api-ms-win-crt-runtime-l1-1-0.dll",
            "_cexit",
            as_win_api!(crate::msvcrt::stdlib::_cexit),
        );
        self.register_func(
            "api-ms-win-crt-runtime-l1-1-0.dll",
            "_initterm",
            as_win_api!(crate::msvcrt::crt_init::_initterm),
        );
        self.register_func(
            "api-ms-win-crt-runtime-l1-1-0.dll",
            "_initterm_e",
            as_win_api!(crate::msvcrt::crt_init::_initterm_e),
        );

        // ----- kernel32.dll -----
        self.register_func(
            "kernel32.dll",
            "GetStdHandle",
            as_win_api!(crate::kernel32::console::GetStdHandle),
        );
        self.register_func(
            "kernel32.dll",
            "WriteConsoleA",
            as_win_api!(crate::kernel32::console::WriteConsoleA),
        );
        self.register_func(
            "kernel32.dll",
            "WriteConsoleW",
            as_win_api!(crate::kernel32::console::WriteConsoleW),
        );
        self.register_func(
            "kernel32.dll",
            "WriteFile",
            as_win_api!(crate::kernel32::file::WriteFile),
        );
        self.register_func(
            "kernel32.dll",
            "ExitProcess",
            as_win_api!(crate::kernel32::process::ExitProcess),
        );
        self.register_func(
            "kernel32.dll",
            "GetCommandLineA",
            as_win_api!(crate::kernel32::process::GetCommandLineA),
        );
        self.register_func(
            "kernel32.dll",
            "GetCommandLineW",
            as_win_api!(crate::kernel32::process::GetCommandLineW),
        );
        self.register_func(
            "kernel32.dll",
            "GetModuleHandleA",
            as_win_api!(crate::kernel32::process::GetModuleHandleA),
        );
        self.register_func(
            "kernel32.dll",
            "GetModuleHandleW",
            as_win_api!(crate::kernel32::process::GetModuleHandleW),
        );
        self.register_func(
            "kernel32.dll",
            "GetLastError",
            as_win_api!(crate::kernel32::process::GetLastError),
        );
        self.register_func(
            "kernel32.dll",
            "SetUnhandledExceptionFilter",
            as_win_api!(crate::kernel32::process::SetUnhandledExceptionFilter),
        );
        self.register_func(
            "kernel32.dll",
            "InitializeCriticalSection",
            as_win_api!(crate::kernel32::sync::InitializeCriticalSection),
        );
        self.register_func(
            "kernel32.dll",
            "EnterCriticalSection",
            as_win_api!(crate::kernel32::sync::EnterCriticalSection),
        );
        self.register_func(
            "kernel32.dll",
            "LeaveCriticalSection",
            as_win_api!(crate::kernel32::sync::LeaveCriticalSection),
        );
        self.register_func(
            "kernel32.dll",
            "DeleteCriticalSection",
            as_win_api!(crate::kernel32::sync::DeleteCriticalSection),
        );
        self.register_func(
            "kernel32.dll",
            "TlsGetValue",
            as_win_api!(crate::kernel32::thread::TlsGetValue),
        );
        self.register_func(
            "kernel32.dll",
            "Sleep",
            as_win_api!(crate::kernel32::thread::Sleep),
        );
        self.register_func(
            "kernel32.dll",
            "VirtualProtect",
            as_win_api!(crate::kernel32::memory::VirtualProtect),
        );
        self.register_func(
            "kernel32.dll",
            "VirtualQuery",
            as_win_api!(crate::kernel32::memory::VirtualQuery),
        );
    }
}

impl Default for DllRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Normalize a DLL name: lowercase, ensure `.dll` extension.
fn normalize_dll_name(name: &str) -> String {
    let lower = name.to_ascii_lowercase();
    if lower.ends_with(".dll") {
        lower
    } else {
        format!("{lower}.dll")
    }
}

/// Default stub for unimplemented Windows API functions.
/// Logs the call and aborts — this is intentionally noisy so missing
/// implementations are immediately visible during development.
unsafe extern "win64" fn stub_function() {
    // In a real call this will be hit when the PE tries to call an
    // unimplemented import. We can't know which function was called from
    // here (the caller burned through the IAT pointer), but the resolver
    // logs which imports were stubbed at load time.
    eprintln!("rine: called unimplemented Windows API stub — aborting");
    std::process::abort();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_adds_dll_extension() {
        assert_eq!(normalize_dll_name("kernel32"), "kernel32.dll");
        assert_eq!(normalize_dll_name("KERNEL32.DLL"), "kernel32.dll");
        assert_eq!(normalize_dll_name("msvcrt.dll"), "msvcrt.dll");
    }

    #[test]
    fn registry_knows_core_dlls() {
        let reg = DllRegistry::new();
        assert!(reg.has_dll("kernel32.dll"));
        assert!(reg.has_dll("NTDLL.DLL"));
        assert!(reg.has_dll("msvcrt"));
        assert!(!reg.has_dll("imaginary.dll"));
    }

    #[test]
    fn unimplemented_function_returns_stub() {
        let reg = DllRegistry::new();
        let result = reg.resolve_by_name("kernel32.dll", "CreateFileA");
        assert!(matches!(result, LookupResult::Stub(_)));
    }

    #[test]
    fn resolve_by_ordinal_returns_stub_for_unknown() {
        let reg = DllRegistry::new();
        let result = reg.resolve_by_ordinal("kernel32.dll", 999);
        assert!(matches!(result, LookupResult::Stub(_)));
    }

    #[test]
    fn manual_registration_works() {
        let mut reg = DllRegistry::new();

        unsafe extern "win64" fn fake_func() {}

        reg.register_func("test.dll", "TestFunc", fake_func);
        reg.register_func_ordinal("test.dll", 42, fake_func);

        assert!(matches!(
            reg.resolve_by_name("test.dll", "TestFunc"),
            LookupResult::Found(_)
        ));
        assert!(matches!(
            reg.resolve_by_ordinal("test.dll", 42),
            LookupResult::Found(_)
        ));
        assert!(matches!(
            reg.resolve_by_name("test.dll", "Missing"),
            LookupResult::Stub(_)
        ));
    }

    #[test]
    fn phase1_step4_functions_resolve_as_found() {
        let reg = DllRegistry::new();

        // ntdll
        assert!(matches!(
            reg.resolve_by_name("ntdll.dll", "NtWriteFile"),
            LookupResult::Found(_)
        ));
        assert!(matches!(
            reg.resolve_by_name("ntdll.dll", "NtTerminateProcess"),
            LookupResult::Found(_)
        ));
        assert!(matches!(
            reg.resolve_by_name("ntdll.dll", "RtlInitUnicodeString"),
            LookupResult::Found(_)
        ));

        // kernel32
        assert!(matches!(
            reg.resolve_by_name("kernel32.dll", "GetStdHandle"),
            LookupResult::Found(_)
        ));
        assert!(matches!(
            reg.resolve_by_name("kernel32.dll", "WriteConsoleA"),
            LookupResult::Found(_)
        ));
        assert!(matches!(
            reg.resolve_by_name("kernel32.dll", "WriteConsoleW"),
            LookupResult::Found(_)
        ));
        assert!(matches!(
            reg.resolve_by_name("kernel32.dll", "WriteFile"),
            LookupResult::Found(_)
        ));
        assert!(matches!(
            reg.resolve_by_name("kernel32.dll", "ExitProcess"),
            LookupResult::Found(_)
        ));
        assert!(matches!(
            reg.resolve_by_name("kernel32.dll", "GetCommandLineA"),
            LookupResult::Found(_)
        ));
        assert!(matches!(
            reg.resolve_by_name("kernel32.dll", "GetCommandLineW"),
            LookupResult::Found(_)
        ));
        assert!(matches!(
            reg.resolve_by_name("kernel32.dll", "GetModuleHandleA"),
            LookupResult::Found(_)
        ));
        assert!(matches!(
            reg.resolve_by_name("kernel32.dll", "GetModuleHandleW"),
            LookupResult::Found(_)
        ));

        // Unimplemented functions still return Stub
        assert!(matches!(
            reg.resolve_by_name("kernel32.dll", "CreateFileA"),
            LookupResult::Stub(_)
        ));
    }

    #[test]
    fn phase1_step5_msvcrt_functions_resolve_as_found() {
        let reg = DllRegistry::new();

        // msvcrt.dll
        assert!(matches!(
            reg.resolve_by_name("msvcrt.dll", "printf"),
            LookupResult::Found(_)
        ));
        assert!(matches!(
            reg.resolve_by_name("msvcrt.dll", "puts"),
            LookupResult::Found(_)
        ));
        assert!(matches!(
            reg.resolve_by_name("msvcrt.dll", "exit"),
            LookupResult::Found(_)
        ));
        assert!(matches!(
            reg.resolve_by_name("msvcrt.dll", "_cexit"),
            LookupResult::Found(_)
        ));
        assert!(matches!(
            reg.resolve_by_name("msvcrt.dll", "__getmainargs"),
            LookupResult::Found(_)
        ));
        assert!(matches!(
            reg.resolve_by_name("msvcrt.dll", "_initterm"),
            LookupResult::Found(_)
        ));
        assert!(matches!(
            reg.resolve_by_name("msvcrt.dll", "_initterm_e"),
            LookupResult::Found(_)
        ));

        // CRT forwarder DLLs should resolve the same functions
        assert!(matches!(
            reg.resolve_by_name("api-ms-win-crt-stdio-l1-1-0.dll", "printf"),
            LookupResult::Found(_)
        ));
        assert!(matches!(
            reg.resolve_by_name("api-ms-win-crt-runtime-l1-1-0.dll", "_initterm"),
            LookupResult::Found(_)
        ));
    }
}
