#![allow(unsafe_op_in_unsafe_fn)]

#[cfg(not(target_pointer_width = "32"))]
compile_error!(
    "crate `rine32-advapi32` must be built for a 32-bit target (for example: --target i686-unknown-linux-gnu)"
);

mod registry;

use rine_dlls::{DllPlugin, Export, PartialExport, as_win_api};

pub struct Advapi32Plugin32;

impl DllPlugin for Advapi32Plugin32 {
    fn dll_names(&self) -> &[&str] {
        &["advapi32.dll"]
    }

    fn exports(&self) -> Vec<Export> {
        vec![
            Export::Func("RegCreateKeyExA", as_win_api!(registry::RegCreateKeyExA)),
            Export::Func("RegCreateKeyExW", as_win_api!(registry::RegCreateKeyExW)),
            Export::Func("RegQueryValueExA", as_win_api!(registry::RegQueryValueExA)),
            Export::Func("RegQueryValueExW", as_win_api!(registry::RegQueryValueExW)),
            Export::Func("RegSetValueExA", as_win_api!(registry::RegSetValueExA)),
            Export::Func("RegSetValueExW", as_win_api!(registry::RegSetValueExW)),
            Export::Func("RegCloseKey", as_win_api!(registry::RegCloseKey)),
        ]
    }

    fn partials(&self) -> Vec<rine_dlls::PartialExport> {
        vec![
            PartialExport {
                name: "RegOpenKeyExA",
                func: as_win_api!(registry::RegOpenKeyExA),
            },
            PartialExport {
                name: "RegOpenKeyExW",
                func: as_win_api!(registry::RegOpenKeyExW),
            },
        ]
    }
}
