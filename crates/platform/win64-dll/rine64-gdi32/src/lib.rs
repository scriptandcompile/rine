#![allow(unsafe_op_in_unsafe_fn)]

use rine_dlls::{DllPlugin, as_win_api};
mod ops;

pub struct Gdi32Plugin;

impl DllPlugin for Gdi32Plugin {
    fn dll_names(&self) -> &[&str] {
        &["gdi32.dll"]
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

rine_dlls::export_dynamic_provider!(|| Gdi32Plugin);
