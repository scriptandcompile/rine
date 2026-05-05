#![allow(unsafe_op_in_unsafe_fn)]

mod dialogs;

use rine_dlls::{DllPlugin, as_win_api};

pub struct Shell32Plugin;

impl DllPlugin for Shell32Plugin {
    fn dll_names(&self) -> &[&str] {
        &["shell32.dll"]
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

rine_dlls::export_dynamic_provider!(|| Shell32Plugin);
