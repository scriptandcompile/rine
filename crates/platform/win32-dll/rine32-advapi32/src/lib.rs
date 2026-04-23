#![allow(unsafe_op_in_unsafe_fn)]

#[cfg(not(target_pointer_width = "32"))]
compile_error!(
    "crate `rine32-advapi32` must be built for a 32-bit target (for example: --target i686-unknown-linux-gnu)"
);

mod registry;

use rine_dlls::{DllPlugin, as_win_api};

pub struct Advapi32Plugin32;

impl DllPlugin for Advapi32Plugin32 {
    fn dll_names(&self) -> &[&str] {
        &["advapi32.dll"]
    }

    fn exports(&self) -> Vec<rine_dlls::Export> {
        include!(concat!(env!("OUT_DIR"), "/dll_plugin_generated.rs"))
    }

    fn partials(&self) -> Vec<rine_dlls::PartialExport> {
        include!(concat!(
            env!("OUT_DIR"),
            "/dll_plugin_generated_partials.rs"
        ))
    }

    fn stubs(&self) -> Vec<rine_dlls::StubExport> {
        include!(concat!(env!("OUT_DIR"), "/dll_plugin_generated_stubs.rs"))
    }
}
