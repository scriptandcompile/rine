pub mod console;
pub mod environment;
pub mod file;
pub mod memory;
pub mod process;
pub mod strings;
pub mod sync;
pub mod thread;
pub mod version;

use rine_dlls::{DllPlugin, as_win_api};

pub struct Kernel32Plugin;

impl DllPlugin for Kernel32Plugin {
    fn dll_names(&self) -> &[&str] {
        &["kernel32.dll"]
    }
    fn exports(&self) -> Vec<rine_dlls::Export> {
        include!(concat!(env!("OUT_DIR"), "/dll_plugin_generated.rs"))
    }

    fn stubs(&self) -> Vec<rine_dlls::StubExport> {
        include!(concat!(env!("OUT_DIR"), "/dll_plugin_generated_stubs.rs"))
    }

    fn partials(&self) -> Vec<rine_dlls::PartialExport> {
        include!(concat!(
            env!("OUT_DIR"),
            "/dll_plugin_generated_partials.rs"
        ))
    }
}

rine_dlls::export_dynamic_provider!(|| Kernel32Plugin);
