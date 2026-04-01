//! Install/uninstall right-click context menu entries for `.exe` files in Linux
//! file managers.
//!
//! Supported environments:
//! - **Freedesktop**: A secondary `.desktop` file (`rine-configure.desktop`)
//!   that adds "Configure with rine" to the "Open With" menu for `.exe` files.
//! - **Nautilus (GNOME Files)**: A script in `~/.local/share/nautilus/scripts/`
//!   that appears in the right-click → Scripts submenu.
//! - **Dolphin (KDE)**: A ServiceMenu `.desktop` file in
//!   `~/.local/share/kio/servicemenus/` that adds a top-level right-click action.
//! - **Nemo (Cinnamon)**: A `.nemo_action` file in
//!   `~/.local/share/nemo/actions/` that adds a MIME-filtered right-click action.
//!
//! All entries launch `rine --config <exe>` (or `rine-config <exe>` when the
//! Tauri config editor is available).

use std::fmt;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::errors::ContextMenuError;

// ---------------------------------------------------------------------------
// XDG / home helpers
// ---------------------------------------------------------------------------

fn xdg_data_home() -> Result<PathBuf, ContextMenuError> {
    if let Ok(dir) = std::env::var("XDG_DATA_HOME") {
        return Ok(PathBuf::from(dir));
    }
    if let Ok(home) = std::env::var("HOME") {
        return Ok(PathBuf::from(home).join(".local/share"));
    }
    Err(ContextMenuError::NoDataDir)
}

// ---------------------------------------------------------------------------
// Installed file paths
// ---------------------------------------------------------------------------

struct Paths {
    /// Freedesktop .desktop file for "Configure with rine" action.
    desktop_file: PathBuf,
    /// Nautilus script.
    nautilus_script: PathBuf,
    /// Dolphin/KDE service menu.
    dolphin_service: PathBuf,
    /// Nemo action file.
    nemo_action: PathBuf,
    /// Parent directories (for create_dir_all).
    applications_dir: PathBuf,
    nautilus_scripts_dir: PathBuf,
    dolphin_services_dir: PathBuf,
    nemo_actions_dir: PathBuf,
}

impl Paths {
    fn new() -> Result<Self, ContextMenuError> {
        let data = xdg_data_home()?;
        let applications_dir = data.join("applications");
        let nautilus_scripts_dir = data.join("nautilus/scripts");
        let dolphin_services_dir = data.join("kio/servicemenus");
        let nemo_actions_dir = data.join("nemo/actions");

        Ok(Self {
            desktop_file: applications_dir.join("rine-configure.desktop"),
            nautilus_script: nautilus_scripts_dir.join("rine Settings"),
            dolphin_service: dolphin_services_dir.join("rine-configure.desktop"),
            nemo_action: nemo_actions_dir.join("rine-configure.nemo_action"),
            applications_dir,
            nautilus_scripts_dir,
            dolphin_services_dir,
            nemo_actions_dir,
        })
    }
}

// ---------------------------------------------------------------------------
// File content generators
// ---------------------------------------------------------------------------

/// Freedesktop `.desktop` file that appears in the "Open With" list for .exe
/// files. Uses `rine --config %f`.
fn configure_desktop_entry(interpreter: &Path) -> String {
    format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=Configure with rine\n\
         Comment=Open rine settings for this Windows executable\n\
         Exec={interpreter} --config %f\n\
         Terminal=false\n\
         NoDisplay=true\n\
         MimeType=application/x-dosexec;application/x-ms-dos-executable;\n\
         Categories=System;Settings;\n\
         Icon=preferences-system\n",
        interpreter = interpreter.display(),
    )
}

/// Shell script for the Nautilus Scripts menu.
///
/// `$NAUTILUS_SCRIPT_SELECTED_FILE_PATHS` contains newline-separated paths of
/// selected files. We take the first one.
fn nautilus_script_content(interpreter: &Path) -> String {
    format!(
        "#!/bin/sh\n\
         # rine — right-click \"rine Settings\" for .exe files\n\
         exe=\"$(echo \"$NAUTILUS_SCRIPT_SELECTED_FILE_PATHS\" | head -n1)\"\n\
         [ -z \"$exe\" ] && exit 1\n\
         exec {interpreter} --config \"$exe\"\n",
        interpreter = interpreter.display(),
    )
}

/// KDE/Dolphin ServiceMenu `.desktop` file.
///
/// Appears as a top-level right-click action on `.exe` files.
fn dolphin_service_content(interpreter: &Path) -> String {
    format!(
        "[Desktop Entry]\n\
         Type=Service\n\
         MimeType=application/x-dosexec;application/x-ms-dos-executable;\n\
         Actions=configure\n\
         X-KDE-Submenu=rine\n\
         \n\
         [Desktop Action configure]\n\
         Name=Configure with rine\n\
         Icon=preferences-system\n\
         Exec={interpreter} --config %f\n",
        interpreter = interpreter.display(),
    )
}

/// Nemo (Cinnamon) action file.
///
/// `.nemo_action` files support MIME-type filtering, so the action only
/// appears on `.exe` files without needing a Scripts submenu.
fn nemo_action_content(interpreter: &Path) -> String {
    format!(
        "[Nemo Action]\n\
         Name=Configure with rine\n\
         Comment=Open rine settings for this Windows executable\n\
         Exec={interpreter} --config %F\n\
         Icon-Name=preferences-system\n\
         Selection=s\n\
         Mimetypes=application/x-dosexec;application/x-ms-dos-executable;\n",
        interpreter = interpreter.display(),
    )
}

// ---------------------------------------------------------------------------
// Database refresh helper
// ---------------------------------------------------------------------------

fn update_desktop_database(applications_dir: &Path) {
    let _ = Command::new("update-desktop-database")
        .arg(applications_dir)
        .status();
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Summary of which context-menu integrations are installed.
#[derive(Debug)]
pub struct ContextMenuStatus {
    pub desktop_file: Option<PathBuf>,
    pub nautilus_script: Option<PathBuf>,
    pub dolphin_service: Option<PathBuf>,
    pub nemo_action: Option<PathBuf>,
}

impl ContextMenuStatus {
    pub fn is_installed(&self) -> bool {
        self.desktop_file.is_some()
            || self.nautilus_script.is_some()
            || self.dolphin_service.is_some()
            || self.nemo_action.is_some()
    }
}

impl fmt::Display for ContextMenuStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.is_installed() {
            return write!(f, "not installed");
        }
        write!(f, "installed")?;
        if let Some(ref p) = self.desktop_file {
            write!(f, "\n  desktop action: {}", p.display())?;
        }
        if let Some(ref p) = self.nautilus_script {
            write!(f, "\n  nautilus script: {}", p.display())?;
        }
        if let Some(ref p) = self.dolphin_service {
            write!(f, "\n  dolphin service: {}", p.display())?;
        }
        if let Some(ref p) = self.nemo_action {
            write!(f, "\n  nemo action: {}", p.display())?;
        }
        Ok(())
    }
}

/// Query which context-menu files are currently installed.
pub fn status() -> Result<ContextMenuStatus, ContextMenuError> {
    let paths = Paths::new()?;

    Ok(ContextMenuStatus {
        desktop_file: paths.desktop_file.exists().then_some(paths.desktop_file),
        nautilus_script: paths
            .nautilus_script
            .exists()
            .then_some(paths.nautilus_script),
        dolphin_service: paths
            .dolphin_service
            .exists()
            .then_some(paths.dolphin_service),
        nemo_action: paths.nemo_action.exists().then_some(paths.nemo_action),
    })
}

/// Install context-menu entries for all detected desktop environments.
///
/// Always installs the Freedesktop `.desktop` action. Additionally installs
/// Nautilus and Dolphin integrations.
pub fn install(interpreter_override: Option<&Path>) -> Result<ContextMenuStatus, ContextMenuError> {
    let paths = Paths::new()?;

    let interpreter = match interpreter_override {
        Some(p) => p.to_path_buf(),
        None => std::env::current_exe().map_err(ContextMenuError::NoSelfPath)?,
    };

    // -- Freedesktop .desktop action (always) --------------------------------
    fs::create_dir_all(&paths.applications_dir)?;
    fs::write(&paths.desktop_file, configure_desktop_entry(&interpreter))?;

    // -- Nautilus script ------------------------------------------------------
    fs::create_dir_all(&paths.nautilus_scripts_dir)?;
    fs::write(
        &paths.nautilus_script,
        nautilus_script_content(&interpreter),
    )?;
    fs::set_permissions(&paths.nautilus_script, fs::Permissions::from_mode(0o755))?;

    // -- Dolphin service menu -------------------------------------------------
    fs::create_dir_all(&paths.dolphin_services_dir)?;
    fs::write(
        &paths.dolphin_service,
        dolphin_service_content(&interpreter),
    )?;

    // -- Nemo action ----------------------------------------------------------
    fs::create_dir_all(&paths.nemo_actions_dir)?;
    fs::write(&paths.nemo_action, nemo_action_content(&interpreter))?;

    // -- Refresh --------------------------------------------------------------
    update_desktop_database(&paths.applications_dir);

    status()
}

/// Remove all installed context-menu entries.
pub fn uninstall() -> Result<(), ContextMenuError> {
    let paths = Paths::new()?;

    let has_any = paths.desktop_file.exists()
        || paths.nautilus_script.exists()
        || paths.dolphin_service.exists()
        || paths.nemo_action.exists();

    if !has_any {
        return Err(ContextMenuError::NotInstalled);
    }

    if paths.desktop_file.exists() {
        fs::remove_file(&paths.desktop_file)?;
    }
    if paths.nautilus_script.exists() {
        fs::remove_file(&paths.nautilus_script)?;
    }
    if paths.dolphin_service.exists() {
        fs::remove_file(&paths.dolphin_service)?;
    }
    if paths.nemo_action.exists() {
        fs::remove_file(&paths.nemo_action)?;
    }

    update_desktop_database(&paths.applications_dir);

    Ok(())
}
