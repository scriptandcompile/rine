//! Configuration error types.

use std::io;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("config I/O error at {0}: {1}")]
    Io(PathBuf, #[source] io::Error),

    #[error("failed to parse config at {0}: {1}")]
    Parse(PathBuf, #[source] toml::de::Error),

    #[error("failed to serialise config: {0}")]
    Serialize(#[source] toml::ser::Error),
}
