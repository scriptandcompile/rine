//! DLL function registry — maps Windows DLL names and function names/ordinals
//! to Rust function pointers that implement the corresponding Windows API.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

use crate::dynamic_provider::{
    DynamicProviderExportKind, DynamicProviderExportStatus, LoadedDynamicProviderLibrary,
    ResolvedDynamicExport, ResolvedDynamicProvider,
};
use crate::{DllPlugin, Export, PartialExport, StubExport};

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
    dlls: RwLock<HashMap<String, DllModule>>,
    /// Map from lowercase DLL name -> lazy provider factory.
    factories: HashMap<String, ProviderFactory>,
    dynamic_libraries: RwLock<Vec<LoadedDynamicProviderLibrary>>,
    metrics: RegistryCounters,
}

type DllFactory = Arc<dyn Fn() -> Box<dyn DllPlugin> + Send + Sync + 'static>;

#[derive(Clone)]
enum ProviderFactory {
    Static(DllFactory),
    Dynamic(Arc<PathBuf>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DllRegistryMetrics {
    pub registered_dlls: usize,
    pub loaded_dlls: usize,
    pub name_lookups: usize,
    pub ordinal_lookups: usize,
    pub lazy_loads: usize,
    pub cache_hits: usize,
}

#[derive(Default)]
struct RegistryCounters {
    name_lookups: AtomicUsize,
    ordinal_lookups: AtomicUsize,
    lazy_loads: AtomicUsize,
    cache_hits: AtomicUsize,
}

impl RegistryCounters {
    fn snapshot(&self, registered_dlls: usize, loaded_dlls: usize) -> DllRegistryMetrics {
        DllRegistryMetrics {
            registered_dlls,
            loaded_dlls,
            name_lookups: self.name_lookups.load(Ordering::Relaxed),
            ordinal_lookups: self.ordinal_lookups.load(Ordering::Relaxed),
            lazy_loads: self.lazy_loads.load(Ordering::Relaxed),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
        }
    }
}

/// A single reimplemented DLL module with its exported functions.
struct DllModule {
    /// Map from function name → function pointer.
    by_name: HashMap<&'static str, WinApiFunc>,
    /// Map from ordinal → function pointer.
    by_ordinal: HashMap<u16, WinApiFunc>,
    /// Set of function names that are stubs (low-priority implementations).
    stub_names: std::collections::HashSet<&'static str>,
    /// Set of function names that are partial implementations.
    partial_names: std::collections::HashSet<&'static str>,
}

impl DllModule {
    fn new() -> Self {
        Self {
            by_name: HashMap::new(),
            by_ordinal: HashMap::new(),
            stub_names: std::collections::HashSet::new(),
            partial_names: std::collections::HashSet::new(),
        }
    }
}

/// Result of looking up a single import.
#[derive(Debug, Clone, Copy)]
pub enum LookupResult {
    /// Found a fully-implemented Rust function for this import.
    Found(WinApiFunc),
    /// Found a partial implementation with some features missing.
    Partial(WinApiFunc),
    /// Found a stub implementation that provides default behavior.
    Stub(WinApiFunc),
    /// No implementation exists; no matching export, stub, or partial found.
    Unimplemented(WinApiFunc),
}

impl LookupResult {
    /// Get the function pointer regardless of implementation level.
    pub fn as_ptr(self) -> WinApiFunc {
        match self {
            LookupResult::Found(f)
            | LookupResult::Partial(f)
            | LookupResult::Stub(f)
            | LookupResult::Unimplemented(f) => f,
        }
    }
}

impl DllRegistry {
    /// Create an empty registry that can be populated lazily via plugin factories.
    pub fn new_lazy() -> Self {
        Self {
            dlls: RwLock::new(HashMap::new()),
            factories: HashMap::new(),
            dynamic_libraries: RwLock::new(Vec::new()),
            metrics: RegistryCounters::default(),
        }
    }

    /// Register a plugin factory for all DLL names declared by the plugin.
    ///
    /// The factory is called on first lookup of a DLL name, and the resulting
    /// module is cached for subsequent lookups.
    pub fn register_plugin_factory<F>(&mut self, factory: F)
    where
        F: Fn() -> Box<dyn DllPlugin> + Send + Sync + 'static,
    {
        let factory: DllFactory = Arc::new(factory);
        let names = {
            let plugin = factory();
            plugin
                .dll_names()
                .iter()
                .map(|name| normalize_dll_name(name))
                .collect::<Vec<_>>()
        };

        for name in names {
            self.factories
                .insert(name, ProviderFactory::Static(Arc::clone(&factory)));
        }
    }

    /// Register a lazily loaded dynamic provider library for specific DLL names.
    pub fn register_dynamic_provider_library<P>(&mut self, dll_names: &[&str], library_path: P)
    where
        P: AsRef<Path>,
    {
        let library_path = Arc::new(library_path.as_ref().to_path_buf());
        for dll_name in dll_names {
            self.factories.insert(
                normalize_dll_name(dll_name),
                ProviderFactory::Dynamic(Arc::clone(&library_path)),
            );
        }
    }

    /// Build the registry from a set of DLL plugins.
    ///
    /// Each plugin declares which DLL name(s) it provides and returns its
    /// list of exports, stubs, and partial implementations. The registry
    /// collects everything into lookup tables and tracks them separately
    /// to distinguish fully-implemented, partial, and stubbed functions.
    pub fn from_plugins(plugins: &[&dyn DllPlugin]) -> Self {
        let reg = Self::new_lazy();

        for plugin in plugins {
            let dll_names = plugin.dll_names();
            let exports = plugin.exports();
            let stubs = plugin.stubs();
            let partials = plugin.partials();

            for dll_name in dll_names {
                reg.insert_module_for_dll(dll_name, &exports, &stubs, &partials);
            }

            tracing::debug!(
                dlls = ?dll_names,
                exports = exports.len(),
                stubs = stubs.len(),
                partials = partials.len(),
                "registered DLL plugin"
            );
        }

        reg
    }

    /// Look up a function by DLL name and function name.
    pub fn resolve_by_name(&self, dll: &str, name: &str) -> LookupResult {
        self.metrics.name_lookups.fetch_add(1, Ordering::Relaxed);
        let key = normalize_dll_name(dll);
        self.ensure_dll_loaded(&key);

        let dlls = self
            .dlls
            .read()
            .expect("dll registry read lock poisoned in resolve_by_name");
        if let Some(module) = dlls.get(key.as_str())
            && let Some(&func) = module.by_name.get(name)
        {
            if module.stub_names.contains(name) {
                return LookupResult::Stub(func);
            } else if module.partial_names.contains(name) {
                return LookupResult::Partial(func);
            } else {
                return LookupResult::Found(func);
            }
        }
        LookupResult::Unimplemented(stub_function)
    }

    /// Look up a function by DLL name and ordinal number.
    pub fn resolve_by_ordinal(&self, dll: &str, ordinal: u16) -> LookupResult {
        self.metrics.ordinal_lookups.fetch_add(1, Ordering::Relaxed);
        let key = normalize_dll_name(dll);
        self.ensure_dll_loaded(&key);

        let dlls = self
            .dlls
            .read()
            .expect("dll registry read lock poisoned in resolve_by_ordinal");
        if let Some(module) = dlls.get(key.as_str())
            && let Some(&func) = module.by_ordinal.get(&ordinal)
        {
            return LookupResult::Found(func);
        }
        LookupResult::Unimplemented(stub_function)
    }

    /// Returns the list of DLL names this registry knows about.
    pub fn known_dlls(&self) -> Vec<String> {
        let mut names = self.factories.keys().cloned().collect::<Vec<_>>();

        let dlls = self
            .dlls
            .read()
            .expect("dll registry read lock poisoned in known_dlls");
        names.extend(dlls.keys().cloned());
        names.sort_unstable();
        names.dedup();
        names
    }

    /// Return a lightweight snapshot of registry counters.
    pub fn metrics(&self) -> DllRegistryMetrics {
        let registered_dlls = self.known_dlls().len();
        let loaded_dlls = self
            .dlls
            .read()
            .expect("dll registry read lock poisoned in metrics")
            .len();
        self.metrics.snapshot(registered_dlls, loaded_dlls)
    }

    /// Returns true if the registry has any implementation for the given DLL.
    pub fn has_dll(&self, dll: &str) -> bool {
        let key = normalize_dll_name(dll);
        if self.factories.contains_key(key.as_str()) {
            return true;
        }

        let dlls = self
            .dlls
            .read()
            .expect("dll registry read lock poisoned in has_dll");
        dlls.contains_key(key.as_str())
    }

    fn insert_module_for_dll(
        &self,
        dll_name: &str,
        exports: &[Export],
        stubs: &[StubExport],
        partials: &[PartialExport],
    ) {
        let key = normalize_dll_name(dll_name);
        let mut module = DllModule::new();
        populate_module(&mut module, exports, stubs, partials);

        let mut dlls = self
            .dlls
            .write()
            .expect("dll registry write lock poisoned while inserting module");
        dlls.insert(key, module);
    }

    fn insert_dynamic_provider(&self, provider: &ResolvedDynamicProvider) {
        let mut modules = provider
            .dll_names
            .iter()
            .map(|dll_name| (normalize_dll_name(dll_name), DllModule::new()))
            .collect::<HashMap<_, _>>();

        for export in &provider.exports {
            let normalized_dll_name = normalize_dll_name(&export.dll_name);
            let module = modules
                .entry(normalized_dll_name)
                .or_insert_with(DllModule::new);
            let target = dynamic_export_target(export);

            match export.kind {
                DynamicProviderExportKind::NamedFunction | DynamicProviderExportKind::NamedData => {
                    let Some(name) = export.export_name.as_deref() else {
                        continue;
                    };
                    let name = Box::leak(name.to_owned().into_boxed_str());
                    module.by_name.insert(name, target);
                    match export.status {
                        DynamicProviderExportStatus::Implemented => {}
                        DynamicProviderExportStatus::Partial => {
                            module.partial_names.insert(name);
                        }
                        DynamicProviderExportStatus::Stub => {
                            module.stub_names.insert(name);
                        }
                    }
                }
                DynamicProviderExportKind::OrdinalFunction => {
                    module.by_ordinal.insert(export.ordinal, target);
                }
            }
        }

        let mut dlls = self
            .dlls
            .write()
            .expect("dll registry write lock poisoned while inserting dynamic provider modules");
        for (dll_name, module) in modules {
            dlls.insert(dll_name, module);
        }
    }

    fn ensure_dll_loaded(&self, normalized_dll_name: &str) {
        {
            let dlls = self
                .dlls
                .read()
                .expect("dll registry read lock poisoned while checking module cache");
            if dlls.contains_key(normalized_dll_name) {
                self.metrics.cache_hits.fetch_add(1, Ordering::Relaxed);
                return;
            }
        }

        let Some(factory) = self.factories.get(normalized_dll_name).cloned() else {
            return;
        };

        match factory {
            ProviderFactory::Static(factory) => {
                let plugin = factory();
                let exports = plugin.exports();
                let stubs = plugin.stubs();
                let partials = plugin.partials();

                self.insert_module_for_dll(normalized_dll_name, &exports, &stubs, &partials);
                self.metrics.lazy_loads.fetch_add(1, Ordering::Relaxed);

                tracing::debug!(
                    dll = normalized_dll_name,
                    exports = exports.len(),
                    stubs = stubs.len(),
                    partials = partials.len(),
                    "loaded DLL plugin on demand"
                );
            }
            ProviderFactory::Dynamic(path) => {
                let result = unsafe { LoadedDynamicProviderLibrary::open(path.as_ref()) };
                match result {
                    Ok(provider) => {
                        let provider_name = provider.resolved.provider_name.clone();
                        let export_count = provider.resolved.exports.len();
                        self.insert_dynamic_provider(&provider.resolved);
                        self.dynamic_libraries
                            .write()
                            .expect(
                                "dll registry write lock poisoned while storing dynamic library",
                            )
                            .push(provider);
                        self.metrics.lazy_loads.fetch_add(1, Ordering::Relaxed);

                        tracing::debug!(
                            dll = normalized_dll_name,
                            provider = provider_name,
                            path = %path.display(),
                            exports = export_count,
                            "loaded dynamic DLL provider on demand"
                        );
                    }
                    Err(error) => {
                        tracing::error!(
                            dll = normalized_dll_name,
                            path = %path.display(),
                            error = %error,
                            "failed to load dynamic DLL provider"
                        );
                    }
                }
            }
        }
    }
}

fn dynamic_export_target(export: &ResolvedDynamicExport) -> WinApiFunc {
    match export.kind {
        DynamicProviderExportKind::NamedData => unsafe {
            core::mem::transmute::<*const (), WinApiFunc>(export.target)
        },
        DynamicProviderExportKind::NamedFunction | DynamicProviderExportKind::OrdinalFunction => unsafe {
            core::mem::transmute::<*const (), WinApiFunc>(export.target)
        },
    }
}

fn populate_module(
    module: &mut DllModule,
    exports: &[Export],
    stubs: &[StubExport],
    partials: &[PartialExport],
) {
    // Register fully-implemented exports.
    for export in exports {
        match export {
            Export::Func(name, func) => {
                module.by_name.insert(name, *func);
            }
            Export::Ordinal(ord, func) => {
                module.by_ordinal.insert(*ord, *func);
            }
            Export::Data(name, addr) => {
                // SAFETY: data pointers are stored as WinApiFunc for
                // uniform IAT writing. The PE reads the raw address and
                // does not call it as a function.
                let func = unsafe { core::mem::transmute::<*const (), WinApiFunc>(*addr) };
                module.by_name.insert(name, func);
            }
        }
    }

    // Register stubs.
    for stub_export in stubs {
        module.by_name.insert(stub_export.name, stub_export.func);
        module.stub_names.insert(stub_export.name);
    }

    // Register partials.
    for partial_export in partials {
        module
            .by_name
            .insert(partial_export.name, partial_export.func);
        module.partial_names.insert(partial_export.name);
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
    use std::path::Path;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;
    use crate::Export;

    struct TestPlugin;

    #[cfg(target_arch = "x86_64")]
    unsafe extern "win64" fn lazy_fake_func() {}

    #[cfg(not(target_arch = "x86_64"))]
    unsafe extern "C" fn lazy_fake_func() {}

    static LAZY_FACTORY_CALLS: AtomicUsize = AtomicUsize::new(0);
    static LAZY_EXPORT_CALLS: AtomicUsize = AtomicUsize::new(0);

    struct LazyTestPlugin;

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

    impl DllPlugin for LazyTestPlugin {
        fn dll_names(&self) -> &[&str] {
            &["lazy.dll"]
        }

        fn exports(&self) -> Vec<Export> {
            LAZY_EXPORT_CALLS.fetch_add(1, Ordering::SeqCst);
            vec![Export::Func("LazyFunc", lazy_fake_func)]
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
            LookupResult::Unimplemented(_)
        ));
    }

    #[test]
    fn unknown_dll_returns_unimplemented() {
        let reg = DllRegistry::from_plugins(&[]);
        assert!(!reg.has_dll("imaginary.dll"));
        assert!(matches!(
            reg.resolve_by_name("imaginary.dll", "Foo"),
            LookupResult::Unimplemented(_)
        ));
    }

    #[test]
    fn lazy_factory_loads_on_first_hit_and_caches() {
        LAZY_FACTORY_CALLS.store(0, Ordering::SeqCst);
        LAZY_EXPORT_CALLS.store(0, Ordering::SeqCst);

        let mut reg = DllRegistry::new_lazy();
        reg.register_plugin_factory(|| {
            LAZY_FACTORY_CALLS.fetch_add(1, Ordering::SeqCst);
            Box::new(LazyTestPlugin)
        });

        assert!(reg.has_dll("lazy.dll"));
        assert_eq!(LAZY_FACTORY_CALLS.load(Ordering::SeqCst), 1);
        assert_eq!(LAZY_EXPORT_CALLS.load(Ordering::SeqCst), 0);
        assert_eq!(reg.metrics().registered_dlls, 1);
        assert_eq!(reg.metrics().loaded_dlls, 0);

        assert!(matches!(
            reg.resolve_by_name("lazy.dll", "LazyFunc"),
            LookupResult::Found(_)
        ));
        assert_eq!(LAZY_EXPORT_CALLS.load(Ordering::SeqCst), 1);
        let metrics = reg.metrics();
        assert_eq!(metrics.name_lookups, 1);
        assert_eq!(metrics.lazy_loads, 1);
        assert_eq!(metrics.loaded_dlls, 1);
        assert_eq!(metrics.cache_hits, 0);

        assert!(matches!(
            reg.resolve_by_name("lazy.dll", "LazyFunc"),
            LookupResult::Found(_)
        ));
        assert_eq!(LAZY_EXPORT_CALLS.load(Ordering::SeqCst), 1);
        let metrics = reg.metrics();
        assert_eq!(metrics.name_lookups, 2);
        assert_eq!(metrics.lazy_loads, 1);
        assert_eq!(metrics.cache_hits, 1);
    }

    #[test]
    fn lazy_unknown_dll_still_falls_back_to_unimplemented() {
        let mut reg = DllRegistry::new_lazy();
        reg.register_plugin_factory(|| Box::new(LazyTestPlugin));

        assert!(matches!(
            reg.resolve_by_name("missing.dll", "Anything"),
            LookupResult::Unimplemented(_)
        ));
    }

    #[test]
    fn dynamic_provider_missing_library_keeps_stub_fallback() {
        let mut reg = DllRegistry::new_lazy();
        reg.register_dynamic_provider_library(
            &["dynamic-missing.dll"],
            Path::new("/definitely/missing/librine_missing_provider.so"),
        );

        assert!(matches!(
            reg.resolve_by_name("dynamic-missing.dll", "Anything"),
            LookupResult::Unimplemented(_)
        ));
    }
}
