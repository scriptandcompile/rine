//! Per-app configuration schema.
//!
//! Settings are loaded from TOML files at `~/.rine/apps/<app-hash>/config.toml`.
//! Phase 3 will add a full config manager; for now we define the schema
//! structs so that the filesystem and other subsystems can consume them.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Top-level per-application config.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    /// Filesystem / path-translation settings.
    pub filesystem: FilesystemConfig,
}

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
    /// Expensive — off by default.
    pub case_insensitive: bool,
}

impl Default for FilesystemConfig {
    fn default() -> Self {
        Self {
            default_root: None,
            drives: HashMap::new(),
            case_insensitive: false,
        }
    }
}

impl FilesystemConfig {
    /// Build a [`PathTranslator`](crate::subsys::filesystem::PathTranslator)
    /// from this configuration.
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
}
