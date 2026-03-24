//! DLL plugin trait and registry for rine.
//!
//! Each reimplemented Windows DLL lives in its own crate (e.g. `rine-kernel32`,
//! `rine-msvcrt`) and implements the [`DllPlugin`] trait. The [`DllRegistry`]
//! collects plugins and provides import resolution at load time.

mod registry;

pub use registry::{DllRegistry, LookupResult, WinApiFunc};

/// A function pointer stored in the registry, castable to the appropriate
/// signature. Uses `extern "win64"` because PE code calls through the IAT
/// using the Windows x64 calling convention.
///
/// Re-exported from [`registry`] for convenience.

/// A single exported symbol from a DLL plugin.
pub enum Export {
    /// A function export (IAT slot = address of function).
    Func(&'static str, WinApiFunc),
    /// An ordinal function export.
    Ordinal(u16, WinApiFunc),
    /// A data export (IAT slot = raw address of variable, not a function).
    Data(&'static str, *const ()),
}

// SAFETY: data pointers in Export::Data are heap-allocated and live for the
// process lifetime. They are not mutated after creation through this path.
unsafe impl Send for Export {}
unsafe impl Sync for Export {}

/// Trait implemented by each DLL crate to declare its exports.
///
/// The registry calls [`exports()`](DllPlugin::exports) once at startup to
/// collect all function/data pointers into the lookup tables.
pub trait DllPlugin {
    /// The canonical DLL name(s) this plugin provides, including the `.dll`
    /// suffix. e.g. `&["kernel32.dll"]` or `&["msvcrt.dll", "api-ms-win-crt-runtime-l1-1-0.dll"]`.
    fn dll_names(&self) -> &[&str];

    /// Return all exports this plugin provides.
    fn exports(&self) -> Vec<Export>;
}

/// Type-erase a function pointer to [`WinApiFunc`] for plugin registration.
///
/// The PE code calls through the IAT with the correct Windows x64 calling
/// convention, so the true signature is recovered at call-site.
#[macro_export]
macro_rules! as_win_api {
    ($f:expr) => {
        // SAFETY: all function pointers are pointer-sized. The PE caller
        // will invoke through the IAT with the matching argument layout.
        unsafe { core::mem::transmute::<*const (), $crate::WinApiFunc>($f as *const ()) }
    };
}
