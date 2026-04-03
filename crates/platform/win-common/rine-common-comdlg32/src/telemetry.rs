use crate::env_policy::{DialogPolicy, DialogTheme, NativeBackend, WindowsTheme};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogOpenFields {
    pub api: &'static str,
    pub theme: &'static str,
    pub native_backend: &'static str,
    pub windows_theme: &'static str,
}

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

pub fn build_open_fields(api: &'static str, policy: DialogPolicy) -> DialogOpenFields {
    DialogOpenFields {
        api,
        theme: theme_label(policy.theme),
        native_backend: backend_label(policy.native_backend),
        windows_theme: windows_theme_label(policy.windows_theme),
    }
}

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
