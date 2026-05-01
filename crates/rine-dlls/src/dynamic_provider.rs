use std::collections::HashMap;
use std::ffi::{CString, c_char};
use std::ptr;

use crate::{DllPlugin, Export};

pub const DYNAMIC_PROVIDER_ABI_VERSION: u32 = 1;
pub const DYNAMIC_PROVIDER_ENTRYPOINT: &str = "rine_dynamic_provider_v1";

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

// SAFETY: exported metadata is immutable after construction and points either
// to process-lifetime C strings or function/data addresses owned by the plugin.
unsafe impl Send for DynamicProviderExport {}
unsafe impl Sync for DynamicProviderExport {}

// SAFETY: the descriptor only contains immutable pointers into the owned string
// and export storage retained by OwnedDynamicProviderDescriptor.
unsafe impl Send for DynamicProviderDescriptor {}
unsafe impl Sync for DynamicProviderDescriptor {}

// SAFETY: owned descriptor state is built once, never mutated afterward, and
// keeps all pointees alive for the process lifetime through the static OnceLock.
unsafe impl Send for OwnedDynamicProviderDescriptor {}
unsafe impl Sync for OwnedDynamicProviderDescriptor {}

impl OwnedDynamicProviderDescriptor {
    pub fn from_plugin(plugin: &dyn DllPlugin) -> Self {
        let mut string_pool = Vec::new();
        let provider_name = push_c_string(&mut string_pool, plugin.provider_name());

        let dll_name_ptrs = plugin
            .dll_names()
            .iter()
            .map(|dll_name| (dll_name.to_string(), push_c_string(&mut string_pool, dll_name)))
            .collect::<HashMap<_, _>>();

        let dll_names = plugin
            .dll_names()
            .iter()
            .map(|dll_name| *dll_name_ptrs.get(*dll_name).expect("missing DLL name pointer"))
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
                    dll_name: *dll_name_ptrs.get(*dll_name).expect("missing DLL name pointer"),
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
                    dll_name: *dll_name_ptrs.get(*dll_name).expect("missing DLL name pointer"),
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
                        dll_name: *dll_name_ptrs.get(*dll_name).expect("missing DLL name pointer"),
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
                        dll_name: *dll_name_ptrs.get(*dll_name).expect("missing DLL name pointer"),
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
                        dll_name: *dll_name_ptrs.get(*dll_name).expect("missing DLL name pointer"),
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

        let dll_names = unsafe { std::slice::from_raw_parts(descriptor.dll_names, descriptor.dll_count) };
        let dll_names = dll_names
            .iter()
            .map(|ptr| unsafe { CStr::from_ptr(*ptr) }.to_str().expect("dll name should be utf-8"))
            .collect::<Vec<_>>();
        assert_eq!(dll_names, vec!["alpha.dll", "beta.dll"]);

        let exports = unsafe { std::slice::from_raw_parts(descriptor.exports, descriptor.export_count) };
        assert_eq!(
            exports
                .iter()
                .filter(|export| export.kind == DynamicProviderExportKind::NamedFunction
                    && export.status == DynamicProviderExportStatus::Implemented)
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