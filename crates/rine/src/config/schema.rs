//! Per-app configuration schema.
//!
//! Types are defined in [`rine_types::config`] and re-exported here.
//! This module adds rine-specific extensions (e.g. `to_translator()`).

#[allow(unused_imports)]
pub use rine_types::config::{AppConfig, DllConfig, FilesystemConfig, WindowsVersion};

pub trait FilesystemConfigExt {
    fn to_translator(&self) -> crate::subsys::filesystem::PathTranslator;
}

impl FilesystemConfigExt for FilesystemConfig {
    /// Build a [`PathTranslator`](crate::subsys::filesystem::PathTranslator)
    /// from this configuration.
    #[allow(dead_code)]
    fn to_translator(&self) -> crate::subsys::filesystem::PathTranslator {
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
    use std::path::PathBuf;

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
