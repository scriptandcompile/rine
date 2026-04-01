//! Register/unregister rine with the Linux `binfmt_misc` kernel module so that
//! PE executables (MZ magic) can be launched directly (e.g. `./program.exe`).
//!
//! Registration writes to `/proc/sys/fs/binfmt_misc/register` and requires
//! root privileges. Once registered, the kernel will invoke rine automatically
//! for any file whose first two bytes are `MZ`.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::errors::BinfmtError;

/// Name used for the binfmt_misc entry.
const BINFMT_NAME: &str = "DOSWin";

/// The binfmt_misc filesystem paths.
const BINFMT_MISC_DIR: &str = "/proc/sys/fs/binfmt_misc";
const BINFMT_REGISTER: &str = "/proc/sys/fs/binfmt_misc/register";

/// Returns the path to the per-entry control file.
fn entry_path() -> PathBuf {
    Path::new(BINFMT_MISC_DIR).join(BINFMT_NAME)
}

/// Status of the binfmt_misc registration for rine.
#[derive(Debug)]
pub enum BinfmtStatus {
    /// Not registered — the entry file does not exist.
    NotRegistered,
    /// Registered and enabled.
    Enabled { interpreter: PathBuf },
    /// Registered but disabled.
    Disabled { interpreter: PathBuf },
}

impl fmt::Display for BinfmtStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotRegistered => write!(f, "not registered"),
            Self::Enabled { interpreter } => {
                write!(f, "enabled (interpreter: {})", interpreter.display())
            }
            Self::Disabled { interpreter } => {
                write!(f, "disabled (interpreter: {})", interpreter.display())
            }
        }
    }
}

/// Check whether binfmt_misc is mounted.
fn ensure_mounted() -> Result<(), BinfmtError> {
    if Path::new(BINFMT_MISC_DIR).join("status").exists() {
        Ok(())
    } else {
        Err(BinfmtError::NotMounted)
    }
}

/// Query the current registration status.
pub fn status() -> Result<BinfmtStatus, BinfmtError> {
    ensure_mounted()?;

    let path = entry_path();
    if !path.exists() {
        return Ok(BinfmtStatus::NotRegistered);
    }

    let contents = fs::read_to_string(&path)?;
    let interpreter = contents
        .lines()
        .find_map(|line| line.strip_prefix("interpreter "))
        .map(PathBuf::from)
        .unwrap_or_default();

    if contents.lines().any(|l| l == "enabled") {
        Ok(BinfmtStatus::Enabled { interpreter })
    } else {
        Ok(BinfmtStatus::Disabled { interpreter })
    }
}

/// Build the binfmt_misc registration string.
///
/// Format: `:name:type:offset:magic:mask:interpreter:flags:`
/// - `M` = magic-number matching
/// - `MZ` = the DOS/PE magic bytes
/// - `F` = fix-binary: keep using this interpreter even if it is later
///   updated/moved (avoids issues with mount namespaces)
fn registration_string(interpreter: &Path) -> String {
    format!(":{BINFMT_NAME}:M::MZ::{}:F", interpreter.display())
}

/// Register rine as the handler for MZ executables via binfmt_misc.
///
/// Requires root. Uses the currently-running rine binary as the interpreter
/// unless `interpreter_override` is provided.
pub fn install(interpreter_override: Option<&Path>) -> Result<PathBuf, BinfmtError> {
    ensure_mounted()?;

    // Check if already registered.
    let current = status()?;
    if !matches!(current, BinfmtStatus::NotRegistered) {
        return Err(BinfmtError::AlreadyRegistered(current));
    }

    let interpreter = match interpreter_override {
        Some(p) => p.to_path_buf(),
        None => std::env::current_exe().map_err(BinfmtError::NoSelfPath)?,
    };

    let reg = registration_string(&interpreter);
    fs::write(BINFMT_REGISTER, reg.as_bytes()).map_err(|e| {
        if e.kind() == io::ErrorKind::PermissionDenied {
            BinfmtError::PermissionDenied
        } else {
            BinfmtError::Io(e)
        }
    })?;

    Ok(interpreter)
}

/// Remove the binfmt_misc registration for rine.
///
/// Requires root. Writes `-1` to the entry control file.
pub fn uninstall() -> Result<(), BinfmtError> {
    ensure_mounted()?;

    let path = entry_path();
    if !path.exists() {
        return Err(BinfmtError::NotRegistered);
    }

    fs::write(&path, b"-1").map_err(|e| {
        if e.kind() == io::ErrorKind::PermissionDenied {
            BinfmtError::PermissionDenied
        } else {
            BinfmtError::Io(e)
        }
    })?;

    Ok(())
}
