//! Install/uninstall Freedesktop `.desktop` file and MIME type association so
//! that file managers (GNOME Files, Dolphin, …) can open `.exe` files with rine.
//!
//! Installed files:
//! - `~/.local/share/applications/rine.desktop`
//! - `~/.local/share/mime/packages/rine-exe.xml`
//!
//! After writing, `update-mime-database` and `update-desktop-database` are
//! invoked (best-effort) so changes take effect without logout.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::errors::DesktopError;

/// Name used for the .desktop entry (without extension).
const APP_ID: &str = "rine";

/// Return `$XDG_DATA_HOME` or fall back to `$HOME/.local/share`.
fn xdg_data_home() -> Result<PathBuf, DesktopError> {
    if let Ok(dir) = std::env::var("XDG_DATA_HOME") {
        return Ok(PathBuf::from(dir));
    }
    if let Ok(home) = std::env::var("HOME") {
        return Ok(PathBuf::from(home).join(".local/share"));
    }
    Err(DesktopError::NoDataDir)
}

/// Paths where rine installs Freedesktop integration files.
struct Paths {
    desktop_file: PathBuf,
    mime_xml: PathBuf,
    applications_dir: PathBuf,
    mime_packages_dir: PathBuf,
    mime_dir: PathBuf,
}

impl Paths {
    fn new() -> Result<Self, DesktopError> {
        let data = xdg_data_home()?;
        let applications_dir = data.join("applications");
        let mime_packages_dir = data.join("mime/packages");
        let mime_dir = data.join("mime");
        Ok(Self {
            desktop_file: applications_dir.join(format!("{APP_ID}.desktop")),
            mime_xml: mime_packages_dir.join(format!("{APP_ID}-exe.xml")),
            applications_dir,
            mime_packages_dir,
            mime_dir,
        })
    }
}

// ---------------------------------------------------------------------------
// .desktop file content
// ---------------------------------------------------------------------------

fn desktop_entry(interpreter: &Path) -> String {
    format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=rine\n\
         Comment=Run Windows executables on Linux\n\
         Exec={interpreter} %f\n\
         Terminal=true\n\
         NoDisplay=true\n\
         MimeType=application/x-dosexec;application/x-ms-dos-executable;\n\
         Categories=System;Emulator;\n",
        interpreter = interpreter.display(),
    )
}

// ---------------------------------------------------------------------------
// MIME type XML
// ---------------------------------------------------------------------------

/// Shared MIME-info XML that associates `.exe` with rine's MIME types.
const MIME_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info">
  <mime-type type="application/x-dosexec">
    <comment>Windows executable</comment>
    <glob pattern="*.exe"/>
  </mime-type>
  <mime-type type="application/x-ms-dos-executable">
    <comment>Windows executable</comment>
    <glob pattern="*.exe"/>
  </mime-type>
</mime-info>
"#;

// ---------------------------------------------------------------------------
// Database refresh helpers
// ---------------------------------------------------------------------------

fn update_mime_database(mime_dir: &Path) {
    let _ = Command::new("update-mime-database").arg(mime_dir).status();
}

fn update_desktop_database(applications_dir: &Path) {
    let _ = Command::new("update-desktop-database")
        .arg(applications_dir)
        .status();
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Status of the Freedesktop desktop integration.
#[derive(Debug)]
pub enum DesktopStatus {
    /// Neither .desktop file nor MIME XML is installed.
    NotInstalled,
    /// Both files are installed.
    Installed {
        desktop_file: PathBuf,
        mime_xml: PathBuf,
    },
    /// Only one of the two files exists (partial install).
    Partial {
        desktop_file: Option<PathBuf>,
        mime_xml: Option<PathBuf>,
    },
}

impl fmt::Display for DesktopStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInstalled => write!(f, "not installed"),
            Self::Installed {
                desktop_file,
                mime_xml,
            } => {
                write!(
                    f,
                    "installed\n  .desktop: {}\n  MIME XML: {}",
                    desktop_file.display(),
                    mime_xml.display()
                )
            }
            Self::Partial {
                desktop_file,
                mime_xml,
            } => {
                write!(f, "partial install")?;
                if let Some(p) = desktop_file {
                    write!(f, "\n  .desktop: {}", p.display())?;
                }
                if let Some(p) = mime_xml {
                    write!(f, "\n  MIME XML: {}", p.display())?;
                }
                Ok(())
            }
        }
    }
}

/// Query the current desktop-integration status.
pub fn status() -> Result<DesktopStatus, DesktopError> {
    let paths = Paths::new()?;
    let has_desktop = paths.desktop_file.exists();
    let has_mime = paths.mime_xml.exists();

    Ok(match (has_desktop, has_mime) {
        (false, false) => DesktopStatus::NotInstalled,
        (true, true) => DesktopStatus::Installed {
            desktop_file: paths.desktop_file,
            mime_xml: paths.mime_xml,
        },
        _ => DesktopStatus::Partial {
            desktop_file: has_desktop.then_some(paths.desktop_file),
            mime_xml: has_mime.then_some(paths.mime_xml),
        },
    })
}

/// Install the .desktop file and MIME type XML.
///
/// Uses the currently-running rine binary as the interpreter unless
/// `interpreter_override` is provided.
pub fn install(interpreter_override: Option<&Path>) -> Result<PathBuf, DesktopError> {
    let paths = Paths::new()?;

    let interpreter = match interpreter_override {
        Some(p) => p.to_path_buf(),
        None => std::env::current_exe().map_err(DesktopError::NoSelfPath)?,
    };

    // Create directories if they don't exist.
    fs::create_dir_all(&paths.applications_dir)?;
    fs::create_dir_all(&paths.mime_packages_dir)?;

    // Write the .desktop file.
    fs::write(&paths.desktop_file, desktop_entry(&interpreter))?;

    // Write the MIME type XML.
    fs::write(&paths.mime_xml, MIME_XML)?;

    // Refresh databases so changes take effect immediately.
    update_mime_database(&paths.mime_dir);
    update_desktop_database(&paths.applications_dir);

    Ok(interpreter)
}

/// Remove the .desktop file and MIME type XML.
pub fn uninstall() -> Result<(), DesktopError> {
    let paths = Paths::new()?;
    let has_desktop = paths.desktop_file.exists();
    let has_mime = paths.mime_xml.exists();

    if !has_desktop && !has_mime {
        return Err(DesktopError::NotInstalled);
    }

    if has_desktop {
        fs::remove_file(&paths.desktop_file)?;
    }
    if has_mime {
        fs::remove_file(&paths.mime_xml)?;
    }

    // Refresh databases.
    update_mime_database(&paths.mime_dir);
    update_desktop_database(&paths.applications_dir);

    Ok(())
}
