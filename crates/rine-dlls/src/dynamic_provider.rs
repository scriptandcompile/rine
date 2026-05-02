use std::collections::HashMap;
use std::ffi::{CStr, CString, c_char};
use std::path::{Path, PathBuf};
use std::ptr;
use std::str::Utf8Error;

use libloading::{Library, Symbol};
use thiserror::Error;

use crate::{DllPlugin, Export};

pub const DYNAMIC_PROVIDER_ABI_VERSION: u32 = 1;
pub const DYNAMIC_PROVIDER_ENTRYPOINT: &str = "rine_dynamic_provider_v1";

pub type DynamicProviderEntrypoint = unsafe extern "C" fn() -> *const DynamicProviderDescriptor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DynamicProviderArch {
    X86 = 1,
    X64 = 2,
}

impl DynamicProviderArch {
    pub const fn current() -> Self {
        #[cfg(target_pointer_width = "32")]
        {
            Self::X86
        }

        #[cfg(not(target_pointer_width = "32"))]
        {
            Self::X64
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DynamicProviderExportKind {
    NamedFunction = 1,
    NamedData = 2,
    OrdinalFunction = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DynamicProviderExportStatus {
    Implemented = 1,
    Partial = 2,
    Stub = 3,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct DynamicProviderExport {
    pub dll_name: *const c_char,
    pub export_name: *const c_char,
    pub ordinal: u16,
    pub kind: DynamicProviderExportKind,
    pub status: DynamicProviderExportStatus,
    pub target: *const (),
}

#[derive(Debug)]
#[repr(C)]
pub struct DynamicProviderDescriptor {
    pub abi_version: u32,
    pub arch: DynamicProviderArch,
    pub provider_name: *const c_char,
    pub dll_count: usize,
    pub dll_names: *const *const c_char,
    pub export_count: usize,
    pub exports: *const DynamicProviderExport,
}

pub struct OwnedDynamicProviderDescriptor {
    descriptor: DynamicProviderDescriptor,
    _string_pool: Box<[CString]>,
    _dll_names: Box<[*const c_char]>,
    _exports: Box<[DynamicProviderExport]>,
}

#[derive(Debug, Clone)]
pub struct ResolvedDynamicProvider {
    pub provider_name: String,
    pub dll_names: Vec<String>,
    pub exports: Vec<ResolvedDynamicExport>,
}

#[derive(Debug, Clone)]
pub struct ResolvedDynamicExport {
    pub dll_name: String,
    pub export_name: Option<String>,
    pub ordinal: u16,
    pub kind: DynamicProviderExportKind,
    pub status: DynamicProviderExportStatus,
    pub target: *const (),
}

pub struct LoadedDynamicProviderLibrary {
    pub resolved: ResolvedDynamicProvider,
    _library: Library,
}

#[derive(Debug, Error)]
pub enum DynamicProviderLoadError {
    #[error("failed to open dynamic provider library {path}: {source}")]
    Open {
        path: PathBuf,
        #[source]
        source: libloading::Error,
    },

    #[error(
        "failed to resolve entrypoint {entrypoint} in dynamic provider library {path}: {source}"
    )]
    MissingEntrypoint {
        path: PathBuf,
        entrypoint: &'static str,
        #[source]
        source: libloading::Error,
    },

    #[error("dynamic provider library {path} returned a null descriptor")]
    NullDescriptor { path: PathBuf },

    #[error(
        "dynamic provider library {path} uses unsupported ABI version {found}; expected {expected}"
    )]
    AbiVersionMismatch {
        path: PathBuf,
        expected: u32,
        found: u32,
    },

    #[error(
        "dynamic provider library {path} targets {found:?}, but this runtime expects {expected:?}"
    )]
    ArchMismatch {
        path: PathBuf,
        expected: DynamicProviderArch,
        found: DynamicProviderArch,
    },

    #[error("dynamic provider library {path} contains invalid UTF-8 in {field}: {source}")]
    InvalidUtf8 {
        path: PathBuf,
        field: &'static str,
        #[source]
        source: Utf8Error,
    },

    #[error("dynamic provider library {path} contains a null pointer for {field}")]
    NullField { path: PathBuf, field: &'static str },
}

// SAFETY: exported metadata is immutable after construction and points either
// to process-lifetime C strings or function/data addresses owned by the plugin.
unsafe impl Send for DynamicProviderExport {}
unsafe impl Sync for DynamicProviderExport {}

// SAFETY: resolved dynamic exports are immutable values copied from provider metadata.
unsafe impl Send for ResolvedDynamicExport {}
unsafe impl Sync for ResolvedDynamicExport {}

// SAFETY: the descriptor only contains immutable pointers into the owned string
// and export storage retained by OwnedDynamicProviderDescriptor.
unsafe impl Send for DynamicProviderDescriptor {}
unsafe impl Sync for DynamicProviderDescriptor {}

// SAFETY: owned descriptor state is built once, never mutated afterward, and
// keeps all pointees alive for the process lifetime through the static OnceLock.
unsafe impl Send for OwnedDynamicProviderDescriptor {}
unsafe impl Sync for OwnedDynamicProviderDescriptor {}

// SAFETY: loaded dynamic provider state is immutable after load and the library
// handle remains owned for the life of the registry that stores it.
unsafe impl Send for LoadedDynamicProviderLibrary {}
unsafe impl Sync for LoadedDynamicProviderLibrary {}

impl OwnedDynamicProviderDescriptor {
    pub fn from_plugin(plugin: &dyn DllPlugin) -> Self {
        let mut string_pool = Vec::new();
        let provider_name = push_c_string(&mut string_pool, plugin.provider_name());

        let dll_name_ptrs = plugin
            .dll_names()
            .iter()
            .map(|dll_name| {
                (
                    dll_name.to_string(),
                    push_c_string(&mut string_pool, dll_name),
                )
            })
            .collect::<HashMap<_, _>>();

        let dll_names = plugin
            .dll_names()
            .iter()
            .map(|dll_name| {
                *dll_name_ptrs
                    .get(*dll_name)
                    .expect("missing DLL name pointer")
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();

        let mut exports = Vec::new();

        append_exports(
            &mut exports,
            plugin.dll_names(),
            &dll_name_ptrs,
            &mut string_pool,
            plugin.exports(),
        );

        for stub in plugin.stubs() {
            for dll_name in plugin.dll_names() {
                exports.push(DynamicProviderExport {
                    dll_name: *dll_name_ptrs
                        .get(*dll_name)
                        .expect("missing DLL name pointer"),
                    export_name: push_c_string(&mut string_pool, stub.name),
                    ordinal: 0,
                    kind: DynamicProviderExportKind::NamedFunction,
                    status: DynamicProviderExportStatus::Stub,
                    target: stub.func as *const (),
                });
            }
        }

        for partial in plugin.partials() {
            for dll_name in plugin.dll_names() {
                exports.push(DynamicProviderExport {
                    dll_name: *dll_name_ptrs
                        .get(*dll_name)
                        .expect("missing DLL name pointer"),
                    export_name: push_c_string(&mut string_pool, partial.name),
                    ordinal: 0,
                    kind: DynamicProviderExportKind::NamedFunction,
                    status: DynamicProviderExportStatus::Partial,
                    target: partial.func as *const (),
                });
            }
        }

        let string_pool = string_pool.into_boxed_slice();
        let exports = exports.into_boxed_slice();
        let descriptor = DynamicProviderDescriptor {
            abi_version: DYNAMIC_PROVIDER_ABI_VERSION,
            arch: DynamicProviderArch::current(),
            provider_name,
            dll_count: dll_names.len(),
            dll_names: dll_names.as_ptr(),
            export_count: exports.len(),
            exports: exports.as_ptr(),
        };

        Self {
            descriptor,
            _string_pool: string_pool,
            _dll_names: dll_names,
            _exports: exports,
        }
    }

    pub fn as_ptr(&self) -> *const DynamicProviderDescriptor {
        &self.descriptor
    }

    pub fn descriptor(&self) -> &DynamicProviderDescriptor {
        &self.descriptor
    }
}

impl LoadedDynamicProviderLibrary {
    /// # Safety
    /// The loaded library must export a descriptor compatible with the current
    /// ABI and keep all pointed-to metadata alive while the library remains loaded.
    pub unsafe fn open(path: &Path) -> Result<Self, DynamicProviderLoadError> {
        let path = path.to_path_buf();
        let library =
            unsafe { Library::new(&path) }.map_err(|source| DynamicProviderLoadError::Open {
                path: path.clone(),
                source,
            })?;

        let entrypoint: Symbol<'_, DynamicProviderEntrypoint> = unsafe {
            library.get(DYNAMIC_PROVIDER_ENTRYPOINT.as_bytes())
        }
        .map_err(|source| DynamicProviderLoadError::MissingEntrypoint {
            path: path.clone(),
            entrypoint: DYNAMIC_PROVIDER_ENTRYPOINT,
            source,
        })?;

        let descriptor = unsafe { entrypoint() };
        if descriptor.is_null() {
            return Err(DynamicProviderLoadError::NullDescriptor { path });
        }

        let resolved = unsafe { resolve_descriptor(&path, &*descriptor) }?;
        Ok(Self {
            resolved,
            _library: library,
        })
    }
}

unsafe fn resolve_descriptor(
    path: &Path,
    descriptor: &DynamicProviderDescriptor,
) -> Result<ResolvedDynamicProvider, DynamicProviderLoadError> {
    if descriptor.abi_version != DYNAMIC_PROVIDER_ABI_VERSION {
        return Err(DynamicProviderLoadError::AbiVersionMismatch {
            path: path.to_path_buf(),
            expected: DYNAMIC_PROVIDER_ABI_VERSION,
            found: descriptor.abi_version,
        });
    }

    let expected_arch = DynamicProviderArch::current();
    if descriptor.arch != expected_arch {
        return Err(DynamicProviderLoadError::ArchMismatch {
            path: path.to_path_buf(),
            expected: expected_arch,
            found: descriptor.arch,
        });
    }

    let provider_name = read_c_string(path, descriptor.provider_name, "provider_name")?;
    let dll_ptrs = read_ptr_slice(
        path,
        descriptor.dll_names,
        descriptor.dll_count,
        "dll_names",
    )?;
    let dll_names = dll_ptrs
        .iter()
        .map(|dll_name| read_c_string(path, *dll_name, "dll_name"))
        .collect::<Result<Vec<_>, _>>()?;

    let exports = read_ptr_slice(path, descriptor.exports, descriptor.export_count, "exports")?
        .iter()
        .map(|export| resolve_export(path, export))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ResolvedDynamicProvider {
        provider_name,
        dll_names,
        exports,
    })
}

fn resolve_export(
    path: &Path,
    export: &DynamicProviderExport,
) -> Result<ResolvedDynamicExport, DynamicProviderLoadError> {
    let dll_name = read_c_string(path, export.dll_name, "export.dll_name")?;
    let export_name = match export.kind {
        DynamicProviderExportKind::NamedFunction | DynamicProviderExportKind::NamedData => Some(
            read_c_string(path, export.export_name, "export.export_name")?,
        ),
        DynamicProviderExportKind::OrdinalFunction => None,
    };

    Ok(ResolvedDynamicExport {
        dll_name,
        export_name,
        ordinal: export.ordinal,
        kind: export.kind,
        status: export.status,
        target: export.target,
    })
}

fn read_c_string(
    path: &Path,
    ptr: *const c_char,
    field: &'static str,
) -> Result<String, DynamicProviderLoadError> {
    if ptr.is_null() {
        return Err(DynamicProviderLoadError::NullField {
            path: path.to_path_buf(),
            field,
        });
    }

    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .map(|value| value.to_owned())
        .map_err(|source| DynamicProviderLoadError::InvalidUtf8 {
            path: path.to_path_buf(),
            field,
            source,
        })
}

fn read_ptr_slice<'a, T>(
    path: &Path,
    ptr: *const T,
    len: usize,
    field: &'static str,
) -> Result<&'a [T], DynamicProviderLoadError> {
    if len == 0 {
        return Ok(&[]);
    }

    if ptr.is_null() {
        return Err(DynamicProviderLoadError::NullField {
            path: path.to_path_buf(),
            field,
        });
    }

    Ok(unsafe { std::slice::from_raw_parts(ptr, len) })
}

fn append_exports(
    dynamic_exports: &mut Vec<DynamicProviderExport>,
    dll_names: &[&str],
    dll_name_ptrs: &HashMap<String, *const c_char>,
    string_pool: &mut Vec<CString>,
    exports: Vec<Export>,
) {
    for export in exports {
        match export {
            Export::Func(name, func) => {
                for dll_name in dll_names {
                    dynamic_exports.push(DynamicProviderExport {
                        dll_name: *dll_name_ptrs
                            .get(*dll_name)
                            .expect("missing DLL name pointer"),
                        export_name: push_c_string(string_pool, name),
                        ordinal: 0,
                        kind: DynamicProviderExportKind::NamedFunction,
                        status: DynamicProviderExportStatus::Implemented,
                        target: func as *const (),
                    });
                }
            }
            Export::Ordinal(ordinal, func) => {
                for dll_name in dll_names {
                    dynamic_exports.push(DynamicProviderExport {
                        dll_name: *dll_name_ptrs
                            .get(*dll_name)
                            .expect("missing DLL name pointer"),
                        export_name: ptr::null(),
                        ordinal,
                        kind: DynamicProviderExportKind::OrdinalFunction,
                        status: DynamicProviderExportStatus::Implemented,
                        target: func as *const (),
                    });
                }
            }
            Export::Data(name, addr) => {
                for dll_name in dll_names {
                    dynamic_exports.push(DynamicProviderExport {
                        dll_name: *dll_name_ptrs
                            .get(*dll_name)
                            .expect("missing DLL name pointer"),
                        export_name: push_c_string(string_pool, name),
                        ordinal: 0,
                        kind: DynamicProviderExportKind::NamedData,
                        status: DynamicProviderExportStatus::Implemented,
                        target: addr,
                    });
                }
            }
        }
    }
}

fn push_c_string(pool: &mut Vec<CString>, value: &str) -> *const c_char {
    let c_string = CString::new(value).expect("dynamic provider metadata cannot contain NUL bytes");
    let ptr = c_string.as_ptr();
    pool.push(c_string);
    ptr
}

#[cfg(test)]
mod tests {
    use std::ffi::CStr;

    use super::*;
    use crate::{PartialExport, StubExport};

    struct DynamicTestPlugin;

    #[cfg(target_arch = "x86_64")]
    unsafe extern "win64" fn fake_func() {}

    #[cfg(not(target_arch = "x86_64"))]
    unsafe extern "C" fn fake_func() {}

    static TEST_DATA: u32 = 7;

    impl DllPlugin for DynamicTestPlugin {
        fn dll_names(&self) -> &[&str] {
            &["alpha.dll", "beta.dll"]
        }

        fn exports(&self) -> Vec<Export> {
            vec![
                Export::Func("NamedFunc", fake_func),
                Export::Ordinal(77, fake_func),
                Export::Data("SharedData", (&TEST_DATA as *const u32).cast()),
            ]
        }

        fn stubs(&self) -> Vec<StubExport> {
            vec![StubExport {
                name: "StubbedFunc",
                func: fake_func,
            }]
        }

        fn partials(&self) -> Vec<PartialExport> {
            vec![PartialExport {
                name: "PartialFunc",
                func: fake_func,
            }]
        }
    }

    fn make_minimal_descriptor(
        abi_version: u32,
        arch: DynamicProviderArch,
        provider_name: *const c_char,
    ) -> DynamicProviderDescriptor {
        DynamicProviderDescriptor {
            abi_version,
            arch,
            provider_name,
            dll_count: 0,
            dll_names: std::ptr::null(),
            export_count: 0,
            exports: std::ptr::null(),
        }
    }

    #[test]
    fn resolve_descriptor_rejects_wrong_abi_version() {
        let name = c"test-provider";
        let bad_version = DYNAMIC_PROVIDER_ABI_VERSION + 1;
        let descriptor =
            make_minimal_descriptor(bad_version, DynamicProviderArch::current(), name.as_ptr());

        let result = unsafe { resolve_descriptor(Path::new("fake.so"), &descriptor) };
        assert!(
            matches!(
                result,
                Err(DynamicProviderLoadError::AbiVersionMismatch {
                    expected,
                    found,
                    ..
                }) if expected == DYNAMIC_PROVIDER_ABI_VERSION && found == bad_version
            ),
            "expected AbiVersionMismatch, got: {result:?}"
        );
    }

    #[test]
    fn resolve_descriptor_rejects_wrong_arch() {
        let name = c"test-provider";
        let wrong_arch = match DynamicProviderArch::current() {
            DynamicProviderArch::X64 => DynamicProviderArch::X86,
            DynamicProviderArch::X86 => DynamicProviderArch::X64,
        };
        let descriptor =
            make_minimal_descriptor(DYNAMIC_PROVIDER_ABI_VERSION, wrong_arch, name.as_ptr());

        let result = unsafe { resolve_descriptor(Path::new("fake.so"), &descriptor) };
        assert!(
            matches!(
                result,
                Err(DynamicProviderLoadError::ArchMismatch { found, .. }) if found == wrong_arch
            ),
            "expected ArchMismatch, got: {result:?}"
        );
    }

    #[test]
    fn resolve_descriptor_rejects_null_provider_name() {
        let descriptor = make_minimal_descriptor(
            DYNAMIC_PROVIDER_ABI_VERSION,
            DynamicProviderArch::current(),
            std::ptr::null(),
        );

        let result = unsafe { resolve_descriptor(Path::new("fake.so"), &descriptor) };
        assert!(
            matches!(
                result,
                Err(DynamicProviderLoadError::NullField { field, .. }) if field == "provider_name"
            ),
            "expected NullField(provider_name), got: {result:?}"
        );
    }

    #[test]
    fn owned_descriptor_captures_plugin_metadata() {
        let plugin = DynamicTestPlugin;
        let descriptor = OwnedDynamicProviderDescriptor::from_plugin(&plugin);
        let descriptor = descriptor.descriptor();

        assert_eq!(descriptor.abi_version, DYNAMIC_PROVIDER_ABI_VERSION);
        assert_eq!(descriptor.arch, DynamicProviderArch::current());
        assert_eq!(descriptor.dll_count, 2);
        assert_eq!(descriptor.export_count, 10);

        let provider_name = unsafe { CStr::from_ptr(descriptor.provider_name) }
            .to_str()
            .expect("provider name should be utf-8");
        assert!(provider_name.contains("DynamicTestPlugin"));

        let dll_names =
            unsafe { std::slice::from_raw_parts(descriptor.dll_names, descriptor.dll_count) };
        let dll_names = dll_names
            .iter()
            .map(|ptr| {
                unsafe { CStr::from_ptr(*ptr) }
                    .to_str()
                    .expect("dll name should be utf-8")
            })
            .collect::<Vec<_>>();
        assert_eq!(dll_names, vec!["alpha.dll", "beta.dll"]);

        let exports =
            unsafe { std::slice::from_raw_parts(descriptor.exports, descriptor.export_count) };
        assert_eq!(
            exports
                .iter()
                .filter(
                    |export| export.kind == DynamicProviderExportKind::NamedFunction
                        && export.status == DynamicProviderExportStatus::Implemented
                )
                .count(),
            2
        );
        assert_eq!(
            exports
                .iter()
                .filter(|export| export.kind == DynamicProviderExportKind::OrdinalFunction)
                .count(),
            2
        );
        assert_eq!(
            exports
                .iter()
                .filter(|export| export.kind == DynamicProviderExportKind::NamedData)
                .count(),
            2
        );
        assert_eq!(
            exports
                .iter()
                .filter(|export| export.status == DynamicProviderExportStatus::Stub)
                .count(),
            2
        );
        assert_eq!(
            exports
                .iter()
                .filter(|export| export.status == DynamicProviderExportStatus::Partial)
                .count(),
            2
        );
    }
}
