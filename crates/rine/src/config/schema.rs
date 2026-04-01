//! Per-app configuration schema.
//!
//! Settings are loaded from TOML files at `~/.rine/apps/<app-hash>/config.toml`.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Top-level config
// ---------------------------------------------------------------------------

/// Top-level per-application config.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    /// Filesystem / path-translation settings.
    pub filesystem: FilesystemConfig,

    /// Spoofed Windows version reported to the PE.
    pub windows_version: WindowsVersion,

    /// DLL resolution behaviour.
    pub dll: DllConfig,

    /// Extra environment variables injected before PE entry.
    pub environment: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// Windows version
// ---------------------------------------------------------------------------

/// Windows version to report via `GetVersionEx` / `RtlGetVersion`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum WindowsVersion {
    #[serde(alias = "xp")]
    WinXP,
    #[serde(alias = "7", alias = "win7")]
    Win7,
    #[serde(alias = "10", alias = "win10")]
    Win10,
    #[default]
    #[serde(alias = "11", alias = "win11")]
    Win11,
}

impl WindowsVersion {
    /// `(major, minor, build)` tuple matching `OSVERSIONINFOW` semantics.
    pub fn version_triple(self) -> (u32, u32, u32) {
        match self {
            Self::WinXP => (5, 1, 2600),
            Self::Win7 => (6, 1, 7601),
            Self::Win10 => (10, 0, 19045),
            Self::Win11 => (10, 0, 22631),
        }
    }
}

impl std::fmt::Display for WindowsVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WinXP => write!(f, "Windows XP (5.1)"),
            Self::Win7 => write!(f, "Windows 7 (6.1)"),
            Self::Win10 => write!(f, "Windows 10 (10.0)"),
            Self::Win11 => write!(f, "Windows 11 (10.0)"),
        }
    }
}

// ---------------------------------------------------------------------------
// DLL config
// ---------------------------------------------------------------------------

/// DLL resolution settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct DllConfig {
    /// Ordered list of DLL names to try before the built-in registry.
    /// Each entry is a DLL filename (e.g. `"msvcrt.dll"`).
    pub search_order: Vec<String>,

    /// DLLs that should always be stubbed (never loaded from disk).
    pub force_stub: Vec<String>,
}

// ---------------------------------------------------------------------------
// Filesystem config
// ---------------------------------------------------------------------------

/// Drive mappings and path-translation options.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct FilesystemConfig {
    /// Root directory under which unmapped drive letters are created
    /// as single-letter subdirectories (`<root>/c`, `<root>/d`, …).
    ///
    /// Defaults to `~/.rine/drives`.
    pub default_root: Option<PathBuf>,

    /// Explicit drive-letter → Linux directory overrides.
    /// Keys are single uppercase ASCII letters (e.g. `"C"`, `"D"`).
    pub drives: HashMap<String, PathBuf>,

    /// When `true`, filename lookups walk the real directory tree to
    /// match names case-insensitively (like Windows NTFS).
    /// Expensive — off by default.
    pub case_insensitive: bool,
}

impl FilesystemConfig {
    /// Build a [`PathTranslator`](crate::subsys::filesystem::PathTranslator)
    /// from this configuration.
    #[allow(dead_code)]
    pub fn to_translator(&self) -> crate::subsys::filesystem::PathTranslator {
        let mut translator = match &self.default_root {
            Some(root) => crate::subsys::filesystem::PathTranslator::with_root(root.clone()),
            None => crate::subsys::filesystem::PathTranslator::new(),
        };

        for (letter, target) in &self.drives {
            if let Some(ch) = letter.chars().next() {
                translator.map_drive(ch, target.clone());
            }
        }

        translator.case_insensitive = self.case_insensitive;
        translator
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_roundtrips() {
        let cfg = AppConfig::default();
        let toml_str = toml::to_string_pretty(&cfg).unwrap();
        let parsed: AppConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.filesystem.case_insensitive, false);
        assert!(parsed.filesystem.drives.is_empty());
        assert_eq!(parsed.windows_version, WindowsVersion::Win11);
        assert!(parsed.dll.search_order.is_empty());
        assert!(parsed.environment.is_empty());
    }

    #[test]
    fn custom_drives_parse() {
        let toml_str = r#"
[filesystem]
case_insensitive = true

[filesystem.drives]
C = "/mnt/windows"
D = "/media/data"
"#;
        let cfg: AppConfig = toml::from_str(toml_str).unwrap();
        assert!(cfg.filesystem.case_insensitive);
        assert_eq!(
            cfg.filesystem.drives.get("C").unwrap(),
            &PathBuf::from("/mnt/windows")
        );
        assert_eq!(
            cfg.filesystem.drives.get("D").unwrap(),
            &PathBuf::from("/media/data")
        );
    }

    #[test]
    fn to_translator_applies_mappings() {
        let toml_str = r#"
[filesystem]
default_root = "/tmp/rine_test"

[filesystem.drives]
D = "/media/games"
"#;
        let cfg: AppConfig = toml::from_str(toml_str).unwrap();
        let t = cfg.filesystem.to_translator();
        let p = t.translate(r"D:\app.exe", None).unwrap();
        assert_eq!(p, PathBuf::from("/media/games/app.exe"));
    }

    #[test]
    fn windows_version_aliases() {
        for (input, expected) in [
            (r#"windows_version = "xp""#, WindowsVersion::WinXP),
            (r#"windows_version = "winxp""#, WindowsVersion::WinXP),
            (r#"windows_version = "7""#, WindowsVersion::Win7),
            (r#"windows_version = "win7""#, WindowsVersion::Win7),
            (r#"windows_version = "10""#, WindowsVersion::Win10),
            (r#"windows_version = "win10""#, WindowsVersion::Win10),
            (r#"windows_version = "11""#, WindowsVersion::Win11),
            (r#"windows_version = "win11""#, WindowsVersion::Win11),
        ] {
            let cfg: AppConfig = toml::from_str(input).unwrap();
            assert_eq!(cfg.windows_version, expected, "failed for: {input}");
        }
    }

    #[test]
    fn version_triple() {
        assert_eq!(WindowsVersion::WinXP.version_triple(), (5, 1, 2600));
        assert_eq!(WindowsVersion::Win7.version_triple(), (6, 1, 7601));
        assert_eq!(WindowsVersion::Win10.version_triple(), (10, 0, 19045));
        assert_eq!(WindowsVersion::Win11.version_triple(), (10, 0, 22631));
    }

    #[test]
    fn full_config_parse() {
        let toml_str = r#"
windows_version = "win7"

[filesystem]
case_insensitive = true
default_root = "/home/user/.rine/drives"

[filesystem.drives]
C = "/mnt/windows"

[dll]
search_order = ["msvcrt.dll", "kernel32.dll"]
force_stub = ["d3d11.dll"]

[environment]
WINDIR = "C:\\Windows"
LANG = "en_US"
"#;
        let cfg: AppConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.windows_version, WindowsVersion::Win7);
        assert_eq!(cfg.dll.search_order, vec!["msvcrt.dll", "kernel32.dll"]);
        assert_eq!(cfg.dll.force_stub, vec!["d3d11.dll"]);
        assert_eq!(cfg.environment.get("WINDIR").unwrap(), "C:\\Windows");
        assert_eq!(cfg.environment.get("LANG").unwrap(), "en_US");
    }
}
