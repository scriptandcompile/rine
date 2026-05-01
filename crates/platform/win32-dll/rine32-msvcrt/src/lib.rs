//! MSVCRT (Microsoft Visual C Runtime) support for 32-bit Windows DLLs.
//!
//! This crate provides a 32-bit implementation of the MSVCRT DLL by forwarding
//! calls to shared logic in `rine-common-msvcrt`. Functions are organized into modules:
//! - `crt_init`: CRT initialization (__getmainargs, _initterm)
//! - `crt_support`: Exception handling, signal, locks, file descriptor tables
//! - `memory`: malloc, calloc, realloc, free
//! - `stdlib`: exit, _cexit
//! - `string`: string and memory operations
//! - `stdio`: formatted I/O

use rine_dlls::{DllPlugin, as_win_api};

#[cfg(not(target_pointer_width = "32"))]
compile_error!(
    "crate `rine32-msvcrt` must be built for a 32-bit target (for example: --target i686-unknown-linux-gnu)"
);

pub mod crt_init;
pub mod crt_support;
pub mod math;
pub mod memory;
pub mod stdio;
pub mod stdlib;
pub mod string;

pub struct MsvcrtPlugin32;
pub struct CrtForwarderPlugin32;

impl DllPlugin for MsvcrtPlugin32 {
    fn dll_names(&self) -> &[&str] {
        &["msvcrt.dll"]
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

impl DllPlugin for CrtForwarderPlugin32 {
    fn dll_names(&self) -> &[&str] {
        &[
            "api-ms-win-crt-runtime-l1-1-0.dll",
            "api-ms-win-crt-stdio-l1-1-0.dll",
            "api-ms-win-crt-math-l1-1-0.dll",
            "api-ms-win-crt-locale-l1-1-0.dll",
            "api-ms-win-crt-heap-l1-1-0.dll",
            "api-ms-win-crt-string-l1-1-0.dll",
            "api-ms-win-crt-convert-l1-1-0.dll",
            "api-ms-win-crt-environment-l1-1-0.dll",
            "api-ms-win-crt-time-l1-1-0.dll",
            "api-ms-win-crt-filesystem-l1-1-0.dll",
            "api-ms-win-crt-utility-l1-1-0.dll",
            "vcruntime140.dll",
        ]
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
