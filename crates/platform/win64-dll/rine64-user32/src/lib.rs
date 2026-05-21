#![allow(unsafe_op_in_unsafe_fn)]

use rine_dlls::{DllPlugin, as_win_api};

mod accelerator;
mod char_navigation;
mod class_registration;
mod clipboard;
mod cursor;
mod device_context;
mod dialogs;
mod focus;
mod format;
mod icon;
mod load_string;
mod menu;
mod message_queue;
mod message_registration;
mod sound;
mod system_metrics;
mod window_lifecycle;
mod window_long;
mod window_text;

pub struct User32Plugin;

impl DllPlugin for User32Plugin {
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

rine_dlls::export_dynamic_provider!(|| User32Plugin);
