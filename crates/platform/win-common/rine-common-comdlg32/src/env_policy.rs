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

/// Resolves the dialog policy based on environment variables and returns a `DialogPolicy` struct.
///
/// # Returns
/// A `DialogPolicy` struct containing the resolved dialog theme, native backend, and Windows theme.
/// The resolution is based on the following environment variables:
/// - `RINE_DIALOG_THEME`: Can be set to "windows" or "emulated" to use the Windows dialog theme,
///   or any other value (or unset) to use the native dialog theme.
/// - `RINE_DIALOG_MODE`: (Backward compatibility) Can be set to "emulated" or "windows" to use the
///   Windows dialog theme, or any other value (or unset) to use the native dialog theme.
/// - `RINE_DIALOG_NATIVE_BACKEND`: Can be set to "gtk", "kde", or "portal" (or "auto") to specify
///   the native backend to use. Defaults to "portal" if unset or unrecognized.
/// - `RINE_DIALOG_EMULATED_THEME`: Can be set to "xp", "win7", "win10", "win11", or "windows_version"
///   (or "auto") to specify the Windows theme to use when the dialog theme is set to Windows.
///   Defaults to "windows_version" (auto) if unset or unrecognized.
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

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn set_env(k: &str, v: &str) {
        unsafe { std::env::set_var(k, v) };
    }

    fn remove_env(k: &str) {
        unsafe { std::env::remove_var(k) };
    }

    // ── resolve_dialog_policy ──────────────────────────────────────

    #[test]
    #[serial]
    fn default_policy_is_native_portal_auto() {
        remove_env("RINE_DIALOG_THEME");
        remove_env("RINE_DIALOG_MODE");
        remove_env("RINE_DIALOG_NATIVE_BACKEND");
        remove_env("RINE_DIALOG_EMULATED_THEME");
        let policy = resolve_dialog_policy();
        assert_eq!(policy.theme, DialogTheme::Native);
        assert_eq!(policy.native_backend, NativeBackend::Portal);
        assert_eq!(policy.windows_theme, WindowsTheme::Auto);
    }

    // ── resolve_theme ──────────────────────────────────────────────

    #[test]
    #[serial]
    fn theme_windows_via_primary_env() {
        set_env("RINE_DIALOG_THEME", "windows");
        remove_env("RINE_DIALOG_MODE");
        assert_eq!(resolve_theme(), DialogTheme::Windows);
    }

    #[test]
    #[serial]
    fn theme_windows_via_primary_env_emulated_alias() {
        set_env("RINE_DIALOG_THEME", "emulated");
        remove_env("RINE_DIALOG_MODE");
        assert_eq!(resolve_theme(), DialogTheme::Windows);
    }

    #[test]
    #[serial]
    fn theme_windows_is_case_insensitive() {
        set_env("RINE_DIALOG_THEME", "WINDOWS");
        remove_env("RINE_DIALOG_MODE");
        assert_eq!(resolve_theme(), DialogTheme::Windows);
    }

    #[test]
    #[serial]
    fn theme_native_for_unrecognized_primary_value() {
        set_env("RINE_DIALOG_THEME", "other");
        remove_env("RINE_DIALOG_MODE");
        assert_eq!(resolve_theme(), DialogTheme::Native);
    }

    #[test]
    #[serial]
    fn theme_windows_via_backward_compat_env() {
        remove_env("RINE_DIALOG_THEME");
        set_env("RINE_DIALOG_MODE", "windows");
        assert_eq!(resolve_theme(), DialogTheme::Windows);
    }

    #[test]
    #[serial]
    fn theme_windows_via_backward_compat_emulated() {
        remove_env("RINE_DIALOG_THEME");
        set_env("RINE_DIALOG_MODE", "emulated");
        assert_eq!(resolve_theme(), DialogTheme::Windows);
    }

    #[test]
    #[serial]
    fn theme_native_when_only_backward_compat_is_unrecognized() {
        remove_env("RINE_DIALOG_THEME");
        set_env("RINE_DIALOG_MODE", "other");
        assert_eq!(resolve_theme(), DialogTheme::Native);
    }

    #[test]
    #[serial]
    fn theme_primary_takes_precedence_over_backward_compat() {
        set_env("RINE_DIALOG_THEME", "native");
        set_env("RINE_DIALOG_MODE", "windows");
        assert_eq!(resolve_theme(), DialogTheme::Native);
    }

    #[test]
    #[serial]
    fn theme_native_when_no_env_set() {
        remove_env("RINE_DIALOG_THEME");
        remove_env("RINE_DIALOG_MODE");
        assert_eq!(resolve_theme(), DialogTheme::Native);
    }

    // ── resolve_native_backend ─────────────────────────────────────

    #[test]
    #[serial]
    fn native_backend_gtk() {
        set_env("RINE_DIALOG_NATIVE_BACKEND", "gtk");
        assert_eq!(resolve_native_backend(), NativeBackend::Gtk);
    }

    #[test]
    #[serial]
    fn native_backend_kde() {
        set_env("RINE_DIALOG_NATIVE_BACKEND", "kde");
        assert_eq!(resolve_native_backend(), NativeBackend::Kde);
    }

    #[test]
    #[serial]
    fn native_backend_portal() {
        set_env("RINE_DIALOG_NATIVE_BACKEND", "portal");
        assert_eq!(resolve_native_backend(), NativeBackend::Portal);
    }

    #[test]
    #[serial]
    fn native_backend_auto_defaults_to_portal() {
        set_env("RINE_DIALOG_NATIVE_BACKEND", "auto");
        assert_eq!(resolve_native_backend(), NativeBackend::Portal);
    }

    #[test]
    #[serial]
    fn native_backend_unrecognized_defaults_to_portal() {
        set_env("RINE_DIALOG_NATIVE_BACKEND", "unknown");
        assert_eq!(resolve_native_backend(), NativeBackend::Portal);
    }

    #[test]
    #[serial]
    fn native_backend_case_insensitive() {
        set_env("RINE_DIALOG_NATIVE_BACKEND", "GTK");
        assert_eq!(resolve_native_backend(), NativeBackend::Gtk);
    }

    #[test]
    #[serial]
    fn native_backend_default_when_unset() {
        remove_env("RINE_DIALOG_NATIVE_BACKEND");
        assert_eq!(resolve_native_backend(), NativeBackend::Portal);
    }

    // ── resolve_windows_theme ──────────────────────────────────────

    #[test]
    #[serial]
    fn windows_theme_xp() {
        set_env("RINE_DIALOG_EMULATED_THEME", "xp");
        assert_eq!(resolve_windows_theme(), WindowsTheme::Xp);
    }

    #[test]
    #[serial]
    fn windows_theme_win7() {
        set_env("RINE_DIALOG_EMULATED_THEME", "win7");
        assert_eq!(resolve_windows_theme(), WindowsTheme::Win7);
    }

    #[test]
    #[serial]
    fn windows_theme_win10() {
        set_env("RINE_DIALOG_EMULATED_THEME", "win10");
        assert_eq!(resolve_windows_theme(), WindowsTheme::Win10);
    }

    #[test]
    #[serial]
    fn windows_theme_win11() {
        set_env("RINE_DIALOG_EMULATED_THEME", "win11");
        assert_eq!(resolve_windows_theme(), WindowsTheme::Win11);
    }

    #[test]
    #[serial]
    fn windows_theme_auto_explicit() {
        set_env("RINE_DIALOG_EMULATED_THEME", "auto");
        assert_eq!(resolve_windows_theme(), WindowsTheme::Auto);
    }

    #[test]
    #[serial]
    fn windows_theme_windows_version() {
        set_env("RINE_DIALOG_EMULATED_THEME", "windows_version");
        assert_eq!(resolve_windows_theme(), WindowsTheme::Auto);
    }

    #[test]
    #[serial]
    fn windows_theme_unrecognized_defaults_to_auto() {
        set_env("RINE_DIALOG_EMULATED_THEME", "unknown");
        assert_eq!(resolve_windows_theme(), WindowsTheme::Auto);
    }

    #[test]
    #[serial]
    fn windows_theme_case_insensitive() {
        set_env("RINE_DIALOG_EMULATED_THEME", "XP");
        assert_eq!(resolve_windows_theme(), WindowsTheme::Xp);
    }

    #[test]
    #[serial]
    fn windows_theme_default_when_unset() {
        remove_env("RINE_DIALOG_EMULATED_THEME");
        assert_eq!(resolve_windows_theme(), WindowsTheme::Auto);
    }
}
