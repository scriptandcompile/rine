use rine_types::dev_hooks::{DialogOpenTelemetry, DialogResultTelemetry};
use rine_types::dev_notify;

use crate::env_policy::{
    DialogPolicy, DialogTheme, NativeBackend, WindowsTheme, resolve_dialog_policy,
};
use crate::error::last_error;

/// Fields for dialog open telemetry.
/// Contains information about the dialog that was opened, such as:
/// - `api`: The name of the API that is being called (e.g., "GetOpenFileNameA").
/// - `theme`: The dialog theme that was used (e.g., "native" or "windows").
/// - `native_backend`: The native backend that was used (e.g., "gtk", "kde", or "portal").
/// - `windows_theme`: The Windows theme that was used (e.g., "xp", "win7", "win10", "win11", or "windows_version").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogOpenFields {
    pub api: &'static str,
    pub theme: &'static str,
    pub native_backend: &'static str,
    pub windows_theme: &'static str,
}

/// Fields for dialog result telemetry.
/// Includes the same fields as `DialogOpenFields`, plus:
/// - `success`: Whether the dialog operation was successful (true for success, false for failure).
/// - `error_code`: The error code returned by `GetLastError()` after the dialog operation completed.
/// - `selected_path`: An optional string containing the path selected by the user, if applicable (e.g., for file dialogs).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogResultFields {
    pub api: &'static str,
    pub theme: &'static str,
    pub native_backend: &'static str,
    pub windows_theme: &'static str,
    pub success: bool,
    pub error_code: u32,
    pub selected_path: Option<String>,
}

/// Emits telemetry for a dialog open event and returns the dialog policy that should be used for this dialog.
///
/// # Arguments
/// - `api`: The name of the API that is being called (e.g., "GetOpenFileNameA").
///
/// # Returns
/// The `DialogPolicy` that should be used for this dialog, which may be needed by the caller when emitting
/// the result telemetry after the dialog completes.
pub fn emit_opened(api: &'static str) -> DialogPolicy {
    let policy = resolve_dialog_policy();
    let fields = build_open_fields(api, policy);
    dev_notify!(on_dialog_opened(DialogOpenTelemetry {
        api: fields.api,
        theme: fields.theme,
        native_backend: fields.native_backend,
        windows_theme: fields.windows_theme,
    }));
    policy
}

/// Emits telemetry for a dialog result and returns the error code.
///
/// # Arguments
/// * `api`: The name of the API that was called (e.g., "GetOpenFileNameA").
/// * `policy`: The dialog policy that was in effect when the dialog was opened.
/// * `result`: The result of the dialog operation (nonzero for success, zero for failure).
pub fn emit_result(api: &'static str, policy: DialogPolicy, result: i32) {
    let error_code = last_error();
    let fields = build_result_fields(api, policy, result != 0, error_code, None);
    dev_notify!(on_dialog_result(DialogResultTelemetry {
        api: fields.api,
        theme: fields.theme,
        native_backend: fields.native_backend,
        windows_theme: fields.windows_theme,
        success: fields.success,
        error_code: fields.error_code,
        selected_path: fields.selected_path.as_deref(),
    }));
}

/// Builds the fields for a dialog open telemetry event based on the API name and dialog policy.
///
/// # Arguments
/// * `api`: The name of the API that is being called (e.g., "GetOpenFileNameA").
/// * `policy`: The dialog policy that was resolved when the dialog was opened.
///
/// # Returns
/// A `DialogOpenFields` struct containing the relevant information for telemetry.
pub fn build_open_fields(api: &'static str, policy: DialogPolicy) -> DialogOpenFields {
    DialogOpenFields {
        api,
        theme: theme_label(policy.theme),
        native_backend: backend_label(policy.native_backend),
        windows_theme: windows_theme_label(policy.windows_theme),
    }
}

/// Builds the fields for a dialog result telemetry event based on the API name, dialog policy, and result.
///
/// # Arguments
/// * `api`: The name of the API that was called (e.g., "GetOpenFileNameA").
/// * `policy`: The dialog policy that was in effect when the dialog was opened.
/// * `success`: Whether the dialog operation was successful (true for success, false for failure).
/// * `error_code`: The error code returned by `GetLastError()` after the dialog operation completed.
/// * `selected_path`: An optional string containing the path selected by the user, if applicable (e.g., for file dialogs).
///
/// # Returns
/// A `DialogResultFields` struct containing the relevant information for telemetry.
pub fn build_result_fields(
    api: &'static str,
    policy: DialogPolicy,
    success: bool,
    error_code: u32,
    selected_path: Option<String>,
) -> DialogResultFields {
    DialogResultFields {
        api,
        theme: theme_label(policy.theme),
        native_backend: backend_label(policy.native_backend),
        windows_theme: windows_theme_label(policy.windows_theme),
        success,
        error_code,
        selected_path,
    }
}

fn theme_label(theme: DialogTheme) -> &'static str {
    match theme {
        DialogTheme::Native => "native",
        DialogTheme::Windows => "windows",
    }
}

fn backend_label(backend: NativeBackend) -> &'static str {
    match backend {
        NativeBackend::Gtk => "gtk",
        NativeBackend::Kde => "kde",
        NativeBackend::Portal => "portal",
    }
}

fn windows_theme_label(theme: WindowsTheme) -> &'static str {
    match theme {
        WindowsTheme::Xp => "xp",
        WindowsTheme::Win7 => "win7",
        WindowsTheme::Win10 => "win10",
        WindowsTheme::Win11 => "win11",
        WindowsTheme::Auto => "windows_version",
    }
}
