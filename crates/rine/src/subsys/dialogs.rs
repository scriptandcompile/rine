use std::sync::OnceLock;

use rine_types::config::{
    DialogConfig, DialogMode, EmulatedDialogTheme, NativeDialogBackend, WindowsVersion,
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
    pub mode: DialogMode,
    pub native_backend: NativeDialogBackend,
    pub emulated_theme: EmulatedDialogTheme,
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

fn resolve_mode(mode: DialogMode) -> DialogMode {
    match mode {
        DialogMode::Auto => {
            // Default to native dialogs for out-of-the-box DE integration.
            DialogMode::Native
        }
        explicit => explicit,
    }
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

fn resolve_emulated_theme(
    theme: EmulatedDialogTheme,
    windows_version: WindowsVersion,
) -> EmulatedDialogTheme {
    match theme {
        EmulatedDialogTheme::WindowsVersion | EmulatedDialogTheme::Auto => match windows_version {
            WindowsVersion::WinXP => EmulatedDialogTheme::Xp,
            WindowsVersion::Win7 => EmulatedDialogTheme::Win7,
            WindowsVersion::Win10 => EmulatedDialogTheme::Win10,
            WindowsVersion::Win11 => EmulatedDialogTheme::Win11,
        },
        explicit => explicit,
    }
}

/// Initialize dialog policy from app config.
pub fn init_policy(cfg: DialogConfig, windows_version: WindowsVersion) {
    let desktop = detect_desktop();
    let resolved = ResolvedDialogPolicy {
        mode: resolve_mode(cfg.default_mode),
        native_backend: resolve_native_backend(cfg.native_backend, desktop),
        emulated_theme: resolve_emulated_theme(cfg.emulated_theme, windows_version),
        desktop,
    };
    let _ = DIALOG_POLICY.set(resolved);
}

/// Get the resolved dialog policy, if initialized.
pub fn policy() -> Option<&'static ResolvedDialogPolicy> {
    DIALOG_POLICY.get()
}

pub fn mode_env(mode: DialogMode) -> &'static str {
    match mode {
        DialogMode::Auto => "auto",
        DialogMode::Native => "native",
        DialogMode::Emulated => "emulated",
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

pub fn theme_env(theme: EmulatedDialogTheme) -> &'static str {
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
            resolve_emulated_theme(EmulatedDialogTheme::WindowsVersion, WindowsVersion::Win11),
            EmulatedDialogTheme::Win11
        );
        assert_eq!(
            resolve_emulated_theme(EmulatedDialogTheme::WindowsVersion, WindowsVersion::WinXP),
            EmulatedDialogTheme::Xp
        );
    }

    #[test]
    fn auto_mode_prefers_native() {
        assert_eq!(resolve_mode(DialogMode::Auto), DialogMode::Native);
        assert_eq!(resolve_mode(DialogMode::Emulated), DialogMode::Emulated);
    }
}
