#![allow(unsafe_op_in_unsafe_fn)]

#[cfg(not(target_pointer_width = "32"))]
compile_error!(
    "crate `rine32-shell32` must be built for a 32-bit target (for example: --target i686-unknown-linux-gnu)"
);

mod dialogs;

use rine_dlls::{DllPlugin, as_win_api};

pub struct Shell32Plugin32;

impl DllPlugin for Shell32Plugin32 {
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

rine_dlls::export_dynamic_provider!(|| Shell32Plugin32);
