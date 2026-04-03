//! Optional dev-mode hooks for runtime telemetry.
//!
//! When rine runs with `--dev`, a [`DevHook`] implementation is installed
//! that forwards handle/thread/TLS lifecycle events to the rine-dev
//! dashboard over the dev channel.  Without `--dev`, the global hook is
//! `None` and the inline checks compile to a single branch-not-taken.

use std::sync::OnceLock;

/// Telemetry payload for opening a common-dialog API call.
#[derive(Debug, Clone, Copy)]
pub struct DialogOpenTelemetry<'a> {
    pub api: &'a str,
    pub theme: &'a str,
    pub native_backend: &'a str,
    pub windows_theme: &'a str,
}

/// Telemetry payload for the result of a common-dialog API call.
#[derive(Debug, Clone, Copy)]
pub struct DialogResultTelemetry<'a> {
    pub api: &'a str,
    pub theme: &'a str,
    pub native_backend: &'a str,
    pub windows_theme: &'a str,
    pub success: bool,
    pub error_code: u32,
    pub selected_path: Option<&'a str>,
}

/// Trait implemented by the dev-channel bridge in `rine`.
///
/// All methods take `&self` — implementations must use interior
/// mutability (e.g. `Mutex<DevSender>`) for the underlying I/O.
pub trait DevHook: Send + Sync {
    /// A new handle was inserted into the handle table.
    fn on_handle_created(&self, handle: i64, kind: &str, detail: &str);
    /// A handle was removed from the handle table.
    fn on_handle_closed(&self, handle: i64);
    /// A new thread was created via `CreateThread`.
    fn on_thread_created(&self, handle: i64, thread_id: u32, entry_point: u64);
    /// A thread exited.
    fn on_thread_exited(&self, thread_id: u32, exit_code: u32);
    /// A TLS slot was allocated.
    fn on_tls_allocated(&self, index: u32);
    /// A TLS slot was freed.
    fn on_tls_freed(&self, index: u32);
    /// A memory region was allocated.
    fn on_memory_allocated(&self, address: u64, size: u64, source: &str);
    /// A memory region was freed.
    fn on_memory_freed(&self, address: u64, size: u64, source: &str);
    /// A common-dialog API call was opened/requested.
    fn on_dialog_opened(&self, opened: DialogOpenTelemetry<'_>);
    /// A common-dialog API call completed.
    fn on_dialog_result(&self, result: DialogResultTelemetry<'_>);
    /// The process is about to exit.  Implementations should flush any
    /// buffered events and shut down the channel.
    fn on_process_exiting(&self, exit_code: i32);
}

static DEV_HOOK: OnceLock<Box<dyn DevHook>> = OnceLock::new();

/// Install the global dev hook.  Must be called at most once (before PE
/// entry).  Returns `Err` if a hook was already installed.
pub fn set_dev_hook(hook: Box<dyn DevHook>) -> Result<(), Box<dyn DevHook>> {
    DEV_HOOK.set(hook)
}

/// Get a reference to the installed dev hook, if any.
#[inline]
pub fn dev_hook() -> Option<&'static dyn DevHook> {
    DEV_HOOK.get().map(|h| h.as_ref())
}

/// Convenience macro: call a [`DevHook`] method if the hook is installed.
///
/// ```ignore
/// dev_notify!(on_handle_created(handle, "File", &path));
/// ```
#[macro_export]
macro_rules! dev_notify {
    ($method:ident ( $($arg:expr),* $(,)? )) => {
        if let Some(hook) = $crate::dev_hooks::dev_hook() {
            hook.$method($($arg),*);
        }
    };
}
