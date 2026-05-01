//! DLL plugin trait and registry for rine.
//!
//! Each reimplemented Windows DLL lives in its own crate (e.g. `rine-kernel32`,
//! `rine-msvcrt`) and implements the [`DllPlugin`] trait. The [`DllRegistry`]
//! collects plugins and provides import resolution at load time.

mod registry;

pub use registry::{DllRegistry, LookupResult, WinApiFunc};
pub use rine_dll_attrs::{data_export, dll, implemented, ordinal, partial, stubbed};

/// A function pointer stored in the registry, castable to the appropriate
/// signature. Uses `extern "win64"` because PE code calls through the IAT
/// using the Windows x64 calling convention.
///
/// Re-exported from [`registry`] for convenience.
///
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

/// A stub function declaration with its name.
pub struct StubExport {
    pub name: &'static str,
    pub func: WinApiFunc,
}

/// A partially implemented function declaration with its name.
pub struct PartialExport {
    pub name: &'static str,
    pub func: WinApiFunc,
}

/// Trait implemented by each DLL crate to declare its exports.
///
/// Depending on registry mode, [`exports()`](DllPlugin::exports) may be
/// called eagerly at startup or lazily on first lookup for a DLL.
pub trait DllPlugin {
    /// The canonical DLL name(s) this plugin provides, including the `.dll`
    /// suffix. e.g. `&["kernel32.dll"]` or `&["msvcrt.dll", "api-ms-win-crt-runtime-l1-1-0.dll"]`.
    fn dll_names(&self) -> &[&str];

    /// Return all exports this plugin provides.
    fn exports(&self) -> Vec<Export>;

    /// Return all stub functions. Stubs return default values (usually 0) to let
    /// programs continue. Optional; default is empty.
    fn stubs(&self) -> Vec<StubExport> {
        vec![]
    }

    /// Return all partially implemented functions. Partials have some missing features
    /// but can still be used for basic scenarios. Optional; default is empty.
    fn partials(&self) -> Vec<PartialExport> {
        vec![]
    }
}

/// Define a win32 DLL stub function with centralized ABI selection.
///
/// On 32-bit targets we use the native C ABI to keep win32 plugin crates
/// buildable for i686 Linux. On non-32-bit targets we use `win64` so host
/// builds remain compatible with existing function pointer types.
#[macro_export]
macro_rules! win32_stub {
    ($name:ident, $target:literal) => {
        #[cfg(target_pointer_width = "32")]
        #[allow(non_snake_case)]
        #[allow(clippy::missing_safety_doc)]
        pub unsafe extern "C" fn $name() -> u32 {
            tracing::warn!(api = stringify!($name), dll = $target, "win32 stub called");
            0
        }

        #[cfg(not(target_pointer_width = "32"))]
        #[allow(non_snake_case)]
        #[allow(clippy::missing_safety_doc)]
        pub unsafe extern "win64" fn $name() -> u32 {
            tracing::warn!(api = stringify!($name), dll = $target, "win32 stub called");
            0
        }
    };
}

/// Define a win32 DLL partial function with centralized ABI selection.
///
/// A partial function is one where some features are not implemented but the function
/// can still be used for basic scenarios. On 32-bit targets we use the native C ABI
/// to keep win32 plugin crates buildable for i686 Linux. On non-32-bit targets we use
/// `win64` so host builds remain compatible with existing function pointer types.
#[macro_export]
macro_rules! win32_partial {
    ($name:ident, $target:literal) => {
        #[cfg(target_pointer_width = "32")]
        #[allow(non_snake_case)]
        #[allow(clippy::missing_safety_doc)]
        pub unsafe extern "C" fn $name() -> u32 {
            tracing::warn!(
                api = stringify!($name),
                dll = $target,
                "win32 partial function called"
            );
            0
        }

        #[cfg(not(target_pointer_width = "32"))]
        #[allow(non_snake_case)]
        #[allow(clippy::missing_safety_doc)]
        pub unsafe extern "win64" fn $name() -> u32 {
            tracing::warn!(
                api = stringify!($name),
                dll = $target,
                "win32 partial function called"
            );
            0
        }
    };
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
