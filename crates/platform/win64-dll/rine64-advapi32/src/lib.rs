pub mod registry;

use rine_dlls::{DllPlugin, Export, as_win_api};

pub struct Advapi32Plugin;

impl DllPlugin for Advapi32Plugin {
    fn dll_names(&self) -> &[&str] {
        &["advapi32.dll"]
    }

    fn exports(&self) -> Vec<Export> {
        vec![
            // Registry
            Export::Func("RegOpenKeyExA", as_win_api!(registry::RegOpenKeyExA)),
            Export::Func("RegOpenKeyExW", as_win_api!(registry::RegOpenKeyExW)),
            Export::Func("RegCreateKeyExA", as_win_api!(registry::RegCreateKeyExA)),
            Export::Func("RegCreateKeyExW", as_win_api!(registry::RegCreateKeyExW)),
            Export::Func("RegQueryValueExA", as_win_api!(registry::RegQueryValueExA)),
            Export::Func("RegQueryValueExW", as_win_api!(registry::RegQueryValueExW)),
            Export::Func("RegSetValueExA", as_win_api!(registry::RegSetValueExA)),
            Export::Func("RegSetValueExW", as_win_api!(registry::RegSetValueExW)),
            Export::Func("RegCloseKey", as_win_api!(registry::RegCloseKey)),
        ]
    }
}
