pub use rine_types::config::{
    AppConfig, ConfigError, DllConfig, FilesystemConfig, WindowsVersion, config_path, load_config,
    save_config,
};
use serde::Serialize;

#[derive(Serialize)]
pub struct VersionOption {
    pub value: serde_json::Value,
    pub label: String,
}
