pub mod console;
pub mod date_time;
pub mod environment;
pub mod file;
pub mod locale;
pub mod memory;
pub mod process;
pub mod strings;
pub mod sync;
pub mod thread;
pub mod version;

use rine_dlls::{DllPlugin, as_win_api};

#[cfg(not(target_pointer_width = "32"))]
compile_error!(
    "crate `rine32-kernel32` must be built for a 32-bit target (for example: --target i686-unknown-linux-gnu)"
);

pub struct Kernel32Plugin32;

impl DllPlugin for Kernel32Plugin32 {
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

rine_dlls::export_dynamic_provider!(|| Kernel32Plugin32);
