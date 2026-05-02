#![allow(unsafe_op_in_unsafe_fn)]

pub mod error;
pub mod open;
pub mod save;

use rine_dlls::{DllPlugin, as_win_api};

#[cfg(not(target_pointer_width = "32"))]
compile_error!(
    "crate `rine32-comdlg32` must be built for a 32-bit target (for example: --target i686-unknown-linux-gnu)"
);

pub struct Comdlg32Plugin32;

impl DllPlugin for Comdlg32Plugin32 {
    fn dll_names(&self) -> &[&str] {
        &["comdlg32.dll"]
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

rine_dlls::export_dynamic_provider!(|| Comdlg32Plugin32);
