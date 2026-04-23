#![allow(unsafe_op_in_unsafe_fn)]

#[cfg(not(target_pointer_width = "32"))]
compile_error!(
    "crate `rine32-user32` must be built for a 32-bit target (for example: --target i686-unknown-linux-gnu)"
);

use rine_dlls::{DllPlugin, as_win_api};

mod class_registration;
mod message_queue;
mod window_lifecycle;
mod window_text;

pub struct User32Plugin32;

impl DllPlugin for User32Plugin32 {
    fn dll_names(&self) -> &[&str] {
        &["user32.dll"]
    }

    fn stubs(&self) -> Vec<rine_dlls::StubExport> {
        include!(concat!(env!("OUT_DIR"), "/dll_plugin_generated_stubs.rs"))
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
}
