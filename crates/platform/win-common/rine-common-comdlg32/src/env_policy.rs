#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogTheme {
    Native,
    Windows,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeBackend {
    Gtk,
    Kde,
    Portal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowsTheme {
    Xp,
    Win7,
    Win10,
    Win11,
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DialogPolicy {
    pub theme: DialogTheme,
    pub native_backend: NativeBackend,
    pub windows_theme: WindowsTheme,
}

pub fn resolve_dialog_policy() -> DialogPolicy {
    DialogPolicy {
        theme: resolve_theme(),
        native_backend: resolve_native_backend(),
        windows_theme: resolve_windows_theme(),
    }
}

fn resolve_theme() -> DialogTheme {
    if let Ok(v) = std::env::var("RINE_DIALOG_THEME") {
        if v.eq_ignore_ascii_case("windows") || v.eq_ignore_ascii_case("emulated") {
            return DialogTheme::Windows;
        }
        return DialogTheme::Native;
    }

    // Backward compatibility for older env key.
    match std::env::var("RINE_DIALOG_MODE") {
        Ok(v) if v.eq_ignore_ascii_case("emulated") || v.eq_ignore_ascii_case("windows") => {
            DialogTheme::Windows
        }
        _ => DialogTheme::Native,
    }
}

fn resolve_native_backend() -> NativeBackend {
    match std::env::var("RINE_DIALOG_NATIVE_BACKEND") {
        Ok(v) if v.eq_ignore_ascii_case("gtk") => NativeBackend::Gtk,
        Ok(v) if v.eq_ignore_ascii_case("kde") => NativeBackend::Kde,
        Ok(v) if v.eq_ignore_ascii_case("portal") || v.eq_ignore_ascii_case("auto") => {
            NativeBackend::Portal
        }
        _ => NativeBackend::Portal,
    }
}

fn resolve_windows_theme() -> WindowsTheme {
    match std::env::var("RINE_DIALOG_EMULATED_THEME") {
        Ok(v) if v.eq_ignore_ascii_case("xp") => WindowsTheme::Xp,
        Ok(v) if v.eq_ignore_ascii_case("win7") => WindowsTheme::Win7,
        Ok(v) if v.eq_ignore_ascii_case("win10") => WindowsTheme::Win10,
        Ok(v) if v.eq_ignore_ascii_case("win11") => WindowsTheme::Win11,
        Ok(v) if v.eq_ignore_ascii_case("windows_version") || v.eq_ignore_ascii_case("auto") => {
            WindowsTheme::Auto
        }
        _ => WindowsTheme::Auto,
    }
}
