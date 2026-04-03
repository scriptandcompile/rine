//! Per-app configuration types, directory helpers, and TOML I/O.
//!
//! Available when the `config` feature is enabled.

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("config I/O error at {0}: {1}")]
    Io(PathBuf, #[source] io::Error),

    #[error("failed to parse config at {0}: {1}")]
    Parse(PathBuf, #[source] toml::de::Error),

    #[error("failed to serialise config: {0}")]
    Serialize(#[source] toml::ser::Error),
}

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

    /// Dialog/backend behaviour for common dialogs.
    pub dialogs: DialogConfig,

    /// Extra environment variables injected before PE entry.
    pub environment: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// Dialog config
// ---------------------------------------------------------------------------

/// Dialog behaviour settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DialogConfig {
    /// Primary dialog mode selection.
    pub default_mode: DialogMode,

    /// Native backend preference when native dialogs are used.
    pub native_backend: NativeDialogBackend,

    /// Emulated dialog visual style preference.
    pub emulated_theme: EmulatedDialogTheme,
}

impl Default for DialogConfig {
    fn default() -> Self {
        Self {
            default_mode: DialogMode::Auto,
            native_backend: NativeDialogBackend::Auto,
            emulated_theme: EmulatedDialogTheme::WindowsVersion,
        }
    }
}

/// Dialog mode to use by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DialogMode {
    /// Pick best available mode from runtime environment.
    #[default]
    Auto,
    Native,
    Emulated,
}

/// Native dialog backend preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum NativeDialogBackend {
    #[default]
    Auto,
    Portal,
    Gtk,
    Kde,
}

/// Visual style for emulated dialogs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EmulatedDialogTheme {
    Auto,
    Xp,
    Win7,
    Win10,
    #[default]
    WindowsVersion,
    Win11,
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

    /// Human-readable label (e.g. `"Windows 11 (10.0)"`).
    pub fn label(&self) -> &'static str {
        match self {
            Self::WinXP => "Windows XP (5.1.2600)",
            Self::Win7 => "Windows 7 (6.1.7601)",
            Self::Win10 => "Windows 10 (10.0.19045)",
            Self::Win11 => "Windows 11 (10.0.22631)",
        }
    }

    /// All known variants, useful for populating UI dropdowns.
    pub fn all() -> &'static [WindowsVersion] {
        &[Self::WinXP, Self::Win7, Self::Win10, Self::Win11]
    }
}

impl std::fmt::Display for WindowsVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
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
    pub search_order: Vec<String>,

    /// DLLs that should always be stubbed (never loaded from disk).
    pub force_stub: Vec<String>,
}

// ---------------------------------------------------------------------------
// Filesystem config
// ---------------------------------------------------------------------------

/// Drive mappings and path-translation options.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub case_insensitive: bool,
}

impl Default for FilesystemConfig {
    fn default() -> Self {
        Self {
            default_root: None,
            drives: HashMap::new(),
            case_insensitive: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Directory helpers
// ---------------------------------------------------------------------------

/// `~/.rine`
pub fn rine_root() -> PathBuf {
    home().join(".rine")
}

fn home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
}

/// Produce a short, filesystem-safe identifier from an exe path.
///
/// Format: `<stem>-<hex hash>` (e.g. `hello-a1b2c3d4e5f6g7h8`).
pub fn app_hash(exe_path: &Path) -> String {
    let canonical = exe_path
        .canonicalize()
        .unwrap_or_else(|_| exe_path.to_path_buf());
    let stem = exe_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    // Sanitise the stem to be filesystem-safe.
    let stem: String = stem
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    let mut hasher = DefaultHasher::new();
    canonical.hash(&mut hasher);
    let hash = hasher.finish();
    format!("{}-{:016x}", stem, hash)
}

/// Return the on-disk path where the config for `exe_path` is stored.
pub fn config_path(exe_path: &Path) -> PathBuf {
    rine_root()
        .join("apps")
        .join(app_hash(exe_path))
        .join("config.toml")
}

// ---------------------------------------------------------------------------
// Config I/O
// ---------------------------------------------------------------------------

/// Load the config for `exe_path`, returning [`AppConfig::default()`] if
/// no config file exists yet.
pub fn load_config(exe_path: &Path) -> Result<AppConfig, ConfigError> {
    let path = config_path(exe_path);
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let contents = std::fs::read_to_string(&path).map_err(|e| ConfigError::Io(path.clone(), e))?;
    let cfg: AppConfig =
        toml::from_str(&contents).map_err(|e| ConfigError::Parse(path.clone(), e))?;
    Ok(cfg)
}

/// Write `cfg` to disk, creating the app directory if needed.
pub fn save_config(exe_path: &Path, cfg: &AppConfig) -> Result<PathBuf, ConfigError> {
    let path = config_path(exe_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| ConfigError::Io(parent.to_path_buf(), e))?;
    }
    let contents = toml::to_string_pretty(cfg).map_err(ConfigError::Serialize)?;
    std::fs::write(&path, contents).map_err(|e| ConfigError::Io(path.clone(), e))?;
    Ok(path)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_hash_deterministic() {
        let h1 = app_hash(Path::new("/some/fake/path.exe"));
        let h2 = app_hash(Path::new("/some/fake/path.exe"));
        assert_eq!(h1, h2);
    }

    #[test]
    fn app_hash_includes_stem() {
        let h = app_hash(Path::new("/foo/bar/game.exe"));
        assert!(h.starts_with("game-"), "got: {h}");
    }

    #[test]
    fn default_config_roundtrips() {
        let cfg = AppConfig::default();
        let toml_str = toml::to_string_pretty(&cfg).unwrap();
        let parsed: AppConfig = toml::from_str(&toml_str).unwrap();
        assert!(parsed.filesystem.case_insensitive);
        assert!(parsed.filesystem.drives.is_empty());
        assert_eq!(parsed.windows_version, WindowsVersion::Win11);
        assert_eq!(parsed.dialogs.default_mode, DialogMode::Auto);
        assert_eq!(
            parsed.dialogs.emulated_theme,
            EmulatedDialogTheme::WindowsVersion
        );
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
