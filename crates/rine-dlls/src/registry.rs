//! DLL function registry — maps Windows DLL names and function names/ordinals
//! to Rust function pointers that implement the corresponding Windows API.

use std::collections::HashMap;

use crate::{DllPlugin, Export};

/// A function pointer stored in the registry, castable to the appropriate signature.
///
/// Uses `extern "win64"` on x86_64 hosts because PE code calls through the
/// IAT using the Windows x64 calling convention. On non-x86_64 builds we use
/// a portable ABI to keep crates type-checkable while 32-bit runtime support
/// is still in progress.
#[cfg(target_arch = "x86_64")]
pub type WinApiFunc = unsafe extern "win64" fn();

#[cfg(not(target_arch = "x86_64"))]
pub type WinApiFunc = unsafe extern "C" fn();

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
    /// Build the registry from a set of DLL plugins.
    ///
    /// Each plugin declares which DLL name(s) it provides and returns its
    /// list of exports. The registry collects everything into lookup tables.
    pub fn from_plugins(plugins: &[&dyn DllPlugin]) -> Self {
        let mut reg = Self {
            dlls: HashMap::new(),
        };

        for plugin in plugins {
            let dll_names = plugin.dll_names();
            let exports = plugin.exports();

            for dll_name in dll_names {
                let module = reg.get_or_create_module(dll_name);
                for export in &exports {
                    match export {
                        Export::Func(name, func) => {
                            module.by_name.insert(name, *func);
                        }
                        Export::Ordinal(ord, func) => {
                            module.by_ordinal.insert(*ord, *func);
                        }
                        Export::Data(name, addr) => {
                            // SAFETY: data pointers are stored as WinApiFunc
                            // for uniform IAT writing. The PE reads the raw
                            // address, not calling it as a function.
                            let func =
                                unsafe { core::mem::transmute::<*const (), WinApiFunc>(*addr) };
                            module.by_name.insert(name, func);
                        }
                    }
                }
            }

            tracing::debug!(
                dlls = ?dll_names,
                exports = exports.len(),
                "registered DLL plugin"
            );
        }

        reg
    }

    /// Look up a function by DLL name and function name.
    pub fn resolve_by_name(&self, dll: &str, name: &str) -> LookupResult {
        let key = normalize_dll_name(dll);
        if let Some(module) = self.dlls.get(key.as_str())
            && let Some(&func) = module.by_name.get(name)
        {
            return LookupResult::Found(func);
        }
        LookupResult::Stub(stub_function)
    }

    /// Look up a function by DLL name and ordinal number.
    pub fn resolve_by_ordinal(&self, dll: &str, ordinal: u16) -> LookupResult {
        let key = normalize_dll_name(dll);
        if let Some(module) = self.dlls.get(key.as_str())
            && let Some(&func) = module.by_ordinal.get(&ordinal)
        {
            return LookupResult::Found(func);
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

    fn get_or_create_module(&mut self, dll_name: &str) -> &mut DllModule {
        let key = normalize_dll_name(dll_name);
        self.dlls.entry(key).or_insert_with(DllModule::new)
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
#[cfg(target_arch = "x86_64")]
unsafe extern "win64" fn stub_function() {
    eprintln!("rine: called unimplemented Windows API stub — aborting");
    std::process::abort();
}

#[cfg(not(target_arch = "x86_64"))]
unsafe extern "C" fn stub_function() {
    eprintln!("rine: called unimplemented Windows API stub — aborting");
    std::process::abort();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Export;

    struct TestPlugin;

    impl DllPlugin for TestPlugin {
        fn dll_names(&self) -> &[&str] {
            &["test.dll"]
        }

        fn exports(&self) -> Vec<Export> {
            #[cfg(target_arch = "x86_64")]
            unsafe extern "win64" fn fake_func() {}

            #[cfg(not(target_arch = "x86_64"))]
            unsafe extern "C" fn fake_func() {}

            vec![
                Export::Func("TestFunc", fake_func),
                Export::Ordinal(42, fake_func),
            ]
        }
    }

    #[test]
    fn normalize_adds_dll_extension() {
        assert_eq!(normalize_dll_name("kernel32"), "kernel32.dll");
        assert_eq!(normalize_dll_name("KERNEL32.DLL"), "kernel32.dll");
        assert_eq!(normalize_dll_name("msvcrt.dll"), "msvcrt.dll");
    }

    #[test]
    fn plugin_registration_works() {
        let plugin = TestPlugin;
        let reg = DllRegistry::from_plugins(&[&plugin]);

        assert!(reg.has_dll("test.dll"));
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
    fn unknown_dll_returns_stub() {
        let reg = DllRegistry::from_plugins(&[]);
        assert!(!reg.has_dll("imaginary.dll"));
        assert!(matches!(
            reg.resolve_by_name("imaginary.dll", "Foo"),
            LookupResult::Stub(_)
        ));
    }
}
