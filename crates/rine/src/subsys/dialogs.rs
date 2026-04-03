use std::sync::OnceLock;

use rine_types::config::{
    DialogConfig, DialogTheme, EmulatedDialogTheme, NativeDialogBackend, WindowsVersion,
};

static DIALOG_POLICY: OnceLock<ResolvedDialogPolicy> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopEnvironment {
    Gnome,
    Kde,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedDialogPolicy {
    pub theme: DialogTheme,
    pub native_backend: NativeDialogBackend,
    pub windows_theme: EmulatedDialogTheme,
    pub desktop: DesktopEnvironment,
}

fn detect_desktop() -> DesktopEnvironment {
    let desktop = std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .unwrap_or_default()
        .to_ascii_lowercase();

    if desktop.contains("gnome") {
        return DesktopEnvironment::Gnome;
    }
    if desktop.contains("kde") || std::env::var_os("KDE_FULL_SESSION").is_some() {
        return DesktopEnvironment::Kde;
    }
    DesktopEnvironment::Other
}

fn resolve_native_backend(
    backend: NativeDialogBackend,
    desktop: DesktopEnvironment,
) -> NativeDialogBackend {
    match backend {
        NativeDialogBackend::Auto => match desktop {
            // First implementation uses rfd/portal path, so keep portal-preferred.
            DesktopEnvironment::Gnome | DesktopEnvironment::Kde | DesktopEnvironment::Other => {
                NativeDialogBackend::Portal
            }
        },
        explicit => explicit,
    }
}

fn resolve_emulated_theme(windows_version: WindowsVersion) -> EmulatedDialogTheme {
    match windows_version {
        WindowsVersion::WinXP => EmulatedDialogTheme::Xp,
        WindowsVersion::Win7 => EmulatedDialogTheme::Win7,
        WindowsVersion::Win10 => EmulatedDialogTheme::Win10,
        WindowsVersion::Win11 => EmulatedDialogTheme::Win11,
    }
}

/// Initialize dialog policy from app config.
pub fn init_policy(cfg: DialogConfig, windows_version: WindowsVersion) {
    let desktop = detect_desktop();
    let resolved = ResolvedDialogPolicy {
        theme: cfg.theme,
        native_backend: resolve_native_backend(cfg.native_backend, desktop),
        windows_theme: resolve_emulated_theme(windows_version),
        desktop,
    };
    let _ = DIALOG_POLICY.set(resolved);
}

/// Get the resolved dialog policy, if initialized.
pub fn policy() -> Option<&'static ResolvedDialogPolicy> {
    DIALOG_POLICY.get()
}

pub fn dialog_theme_env(theme: DialogTheme) -> &'static str {
    match theme {
        DialogTheme::Native => "native",
        DialogTheme::Windows => "windows",
    }
}

pub fn native_backend_env(backend: NativeDialogBackend) -> &'static str {
    match backend {
        NativeDialogBackend::Auto => "auto",
        NativeDialogBackend::Portal => "portal",
        NativeDialogBackend::Gtk => "gtk",
        NativeDialogBackend::Kde => "kde",
    }
}

pub fn windows_theme_env(theme: EmulatedDialogTheme) -> &'static str {
    match theme {
        EmulatedDialogTheme::Auto => "auto",
        EmulatedDialogTheme::Xp => "xp",
        EmulatedDialogTheme::Win7 => "win7",
        EmulatedDialogTheme::Win10 => "win10",
        EmulatedDialogTheme::WindowsVersion => "windows_version",
        EmulatedDialogTheme::Win11 => "win11",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_windows_version_theme() {
        assert_eq!(
            resolve_emulated_theme(WindowsVersion::Win11),
            EmulatedDialogTheme::Win11
        );
        assert_eq!(
            resolve_emulated_theme(WindowsVersion::WinXP),
            EmulatedDialogTheme::Xp
        );
    }
}
