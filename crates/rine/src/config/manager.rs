//! Per-app configuration manager.
//!
//! Configs live at `~/.rine/apps/<app-hash>/config.toml` where `<app-hash>`
//! is a short, filesystem-safe identifier derived from the canonical path of
//! the executable.

use std::path::{Path, PathBuf};

use rine_types::config::{self, AppConfig, ConfigError};

/// Manages loading and saving of per-app configuration files.
pub struct ConfigManager {
    /// Root directory for all per-app configs (default `~/.rine/apps`).
    root: PathBuf,
}

impl ConfigManager {
    /// Create a manager that stores configs under `~/.rine/apps`.
    pub fn new() -> Self {
        let root = config::rine_root().join("apps");
        Self { root }
    }

    /// Load the config for `exe_path`, returning [`AppConfig::default()`] if
    /// no config file exists yet.
    pub fn load(&self, exe_path: &Path) -> Result<AppConfig, ConfigError> {
        let path = self.config_path(exe_path);
        if !path.exists() {
            return Ok(AppConfig::default());
        }
        let contents =
            std::fs::read_to_string(&path).map_err(|e| ConfigError::Io(path.clone(), e))?;
        let cfg: AppConfig =
            toml::from_str(&contents).map_err(|e| ConfigError::Parse(path.clone(), e))?;
        Ok(cfg)
    }

    /// Write `cfg` to disk, creating the app directory if needed.
    pub fn save(&self, exe_path: &Path, cfg: &AppConfig) -> Result<PathBuf, ConfigError> {
        let path = self.config_path(exe_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ConfigError::Io(parent.to_path_buf(), e))?;
        }
        let contents = toml::to_string_pretty(cfg).map_err(ConfigError::Serialize)?;
        std::fs::write(&path, contents).map_err(|e| ConfigError::Io(path.clone(), e))?;
        Ok(path)
    }

    /// Return the on-disk path where the config for `exe_path` is stored.
    pub fn config_path(&self, exe_path: &Path) -> PathBuf {
        let hash = config::app_hash(exe_path);
        self.root.join(&hash).join("config.toml")
    }

    /// Return the app-hash directory name for `exe_path`.
    pub fn app_dir(&self, exe_path: &Path) -> PathBuf {
        let hash = config::app_hash(exe_path);
        self.root.join(&hash)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rine_types::config::WindowsVersion;
    use std::path::Path;

    #[test]
    fn load_missing_returns_default() {
        let mgr = ConfigManager {
            root: PathBuf::from("/nonexistent/path"),
        };
        let cfg = mgr.load(Path::new("/fake.exe")).unwrap();
        assert_eq!(cfg.windows_version, WindowsVersion::Win11);
    }

    #[test]
    fn save_and_reload() {
        let dir = std::env::temp_dir().join("rine_test_config");
        let _ = std::fs::remove_dir_all(&dir);
        let mgr = ConfigManager { root: dir.clone() };

        let mut cfg = AppConfig::default();
        cfg.windows_version = WindowsVersion::Win7;
        cfg.environment.insert("FOO".into(), "bar".into());

        let exe = Path::new("/tmp/test.exe");
        mgr.save(exe, &cfg).unwrap();

        let loaded = mgr.load(exe).unwrap();
        assert_eq!(loaded.windows_version, WindowsVersion::Win7);
        assert_eq!(loaded.environment.get("FOO").unwrap(), "bar");

        // Cleanup.
        let _ = std::fs::remove_dir_all(&dir);
    }
}
