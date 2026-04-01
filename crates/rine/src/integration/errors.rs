use std::io;

use super::binfmt::BinfmtStatus;

/// binfmt_misc filesystem mount point.
const BINFMT_MISC_DIR: &str = "/proc/sys/fs/binfmt_misc";

#[derive(Debug, thiserror::Error)]
pub enum BinfmtError {
    #[error("binfmt_misc is not mounted at {BINFMT_MISC_DIR}")]
    NotMounted,

    #[error("permission denied — binfmt_misc registration requires root")]
    PermissionDenied,

    #[error("already registered: {0}")]
    AlreadyRegistered(BinfmtStatus),

    #[error("not registered — nothing to uninstall")]
    NotRegistered,

    #[error("could not determine rine binary path: {0}")]
    NoSelfPath(io::Error),

    #[error("{0}")]
    Io(#[from] io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum DesktopError {
    #[error("could not determine rine binary path: {0}")]
    NoSelfPath(io::Error),

    #[error("XDG data directory not found (no HOME or XDG_DATA_HOME set)")]
    NoDataDir,

    #[error("desktop entry not installed — nothing to uninstall")]
    NotInstalled,

    #[error("{0}")]
    Io(#[from] io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum ContextMenuError {
    #[error("could not determine rine binary path: {0}")]
    NoSelfPath(io::Error),

    #[error("XDG data directory not found (no HOME or XDG_DATA_HOME set)")]
    NoDataDir,

    #[error("context menu integration not installed — nothing to uninstall")]
    NotInstalled,

    #[error("{0}")]
    Io(#[from] io::Error),
}
