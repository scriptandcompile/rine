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
//! Context-menu actions vary by build:
//! - **Configure with rine** — always installed (`rine --config <exe>`).
//! - **Dev dashboard** — only installed in builds with the `dev` feature.

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
    configure_desktop: PathBuf,
    /// Freedesktop .desktop file for "Dev dashboard" action.
    dev_desktop: PathBuf,
    /// Nautilus "configure" script.
    nautilus_configure: PathBuf,
    /// Nautilus "dev" script.
    nautilus_dev: PathBuf,
    /// Dolphin/KDE service menu (single file with both actions).
    dolphin_service: PathBuf,
    /// Nemo "configure" action file.
    nemo_configure: PathBuf,
    /// Nemo "dev" action file.
    nemo_dev: PathBuf,
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
            configure_desktop: applications_dir.join("rine-configure.desktop"),
            dev_desktop: applications_dir.join("rine-dev.desktop"),
            nautilus_configure: nautilus_scripts_dir.join("rine Settings"),
            nautilus_dev: nautilus_scripts_dir.join("rine Dev Dashboard"),
            dolphin_service: dolphin_services_dir.join("rine.desktop"),
            nemo_configure: nemo_actions_dir.join("rine-configure.nemo_action"),
            nemo_dev: nemo_actions_dir.join("rine-dev.nemo_action"),
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

/// Freedesktop `.desktop` file for the dev dashboard action.
fn dev_desktop_entry(interpreter: &Path) -> String {
    format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=Dev dashboard (rine)\n\
         Comment=Launch the rine developer dashboard for this Windows executable\n\
         Exec={interpreter} --dev %f\n\
         Terminal=false\n\
         NoDisplay=true\n\
         MimeType=application/x-dosexec;application/x-ms-dos-executable;\n\
         Categories=Development;\n\
         Icon=utilities-terminal\n",
        interpreter = interpreter.display(),
    )
}

/// Shell script for the Nautilus Scripts menu (configure).
///
/// `$NAUTILUS_SCRIPT_SELECTED_FILE_PATHS` contains newline-separated paths of
/// selected files. We take the first one.
fn nautilus_configure_content(interpreter: &Path) -> String {
    format!(
        "#!/bin/sh\n\
         # rine — right-click \"rine Settings\" for .exe files\n\
         exe=\"$(echo \"$NAUTILUS_SCRIPT_SELECTED_FILE_PATHS\" | head -n1)\"\n\
         [ -z \"$exe\" ] && exit 1\n\
         exec {interpreter} --config \"$exe\"\n",
        interpreter = interpreter.display(),
    )
}

/// Shell script for the Nautilus Scripts menu (dev dashboard).
fn nautilus_dev_content(interpreter: &Path) -> String {
    format!(
        "#!/bin/sh\n\
         # rine — right-click \"rine Dev Dashboard\" for .exe files\n\
         exe=\"$(echo \"$NAUTILUS_SCRIPT_SELECTED_FILE_PATHS\" | head -n1)\"\n\
         [ -z \"$exe\" ] && exit 1\n\
         exec {interpreter} --dev \"$exe\"\n",
        interpreter = interpreter.display(),
    )
}

/// KDE/Dolphin ServiceMenu `.desktop` file.
///
/// Appears as a top-level right-click action on `.exe` files.
/// Contains both "configure" and "dev dashboard" actions under a rine submenu.
fn dolphin_service_content(interpreter: &Path) -> String {
    if cfg!(feature = "dev") {
        return format!(
            "[Desktop Entry]\n\
             Type=Service\n\
             MimeType=application/x-dosexec;application/x-ms-dos-executable;\n\
             Actions=configure;dev\n\
             X-KDE-Submenu=rine\n\
             \n\
             [Desktop Action configure]\n\
             Name=Configure\n\
             Icon=preferences-system\n\
             Exec={interpreter} --config %f\n\
             \n\
             [Desktop Action dev]\n\
             Name=Dev dashboard\n\
             Icon=utilities-terminal\n\
             Exec={interpreter} --dev %f\n",
            interpreter = interpreter.display(),
        );
    }

    format!(
        "[Desktop Entry]\n\
         Type=Service\n\
         MimeType=application/x-dosexec;application/x-ms-dos-executable;\n\
         Actions=configure\n\
         X-KDE-Submenu=rine\n\
         \n\
         [Desktop Action configure]\n\
         Name=Configure\n\
         Icon=preferences-system\n\
         Exec={interpreter} --config %f\n",
        interpreter = interpreter.display(),
    )
}

/// Nemo (Cinnamon) "configure" action file.
fn nemo_configure_content(interpreter: &Path) -> String {
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

/// Nemo (Cinnamon) "dev dashboard" action file.
fn nemo_dev_content(interpreter: &Path) -> String {
    format!(
        "[Nemo Action]\n\
         Name=Dev dashboard (rine)\n\
         Comment=Launch the rine developer dashboard for this Windows executable\n\
         Exec={interpreter} --dev %F\n\
         Icon-Name=utilities-terminal\n\
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
    pub configure_desktop: Option<PathBuf>,
    pub dev_desktop: Option<PathBuf>,
    pub nautilus_configure: Option<PathBuf>,
    pub nautilus_dev: Option<PathBuf>,
    pub dolphin_service: Option<PathBuf>,
    pub nemo_configure: Option<PathBuf>,
    pub nemo_dev: Option<PathBuf>,
}

impl ContextMenuStatus {
    pub fn is_installed(&self) -> bool {
        self.configure_desktop.is_some()
            || self.dev_desktop.is_some()
            || self.nautilus_configure.is_some()
            || self.nautilus_dev.is_some()
            || self.dolphin_service.is_some()
            || self.nemo_configure.is_some()
            || self.nemo_dev.is_some()
    }
}

impl fmt::Display for ContextMenuStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.is_installed() {
            return write!(f, "not installed");
        }
        write!(f, "installed")?;
        if let Some(ref p) = self.configure_desktop {
            write!(f, "\n  desktop (configure): {}", p.display())?;
        }
        if let Some(ref p) = self.dev_desktop {
            write!(f, "\n  desktop (dev):       {}", p.display())?;
        }
        if let Some(ref p) = self.nautilus_configure {
            write!(f, "\n  nautilus (configure): {}", p.display())?;
        }
        if let Some(ref p) = self.nautilus_dev {
            write!(f, "\n  nautilus (dev):       {}", p.display())?;
        }
        if let Some(ref p) = self.dolphin_service {
            write!(f, "\n  dolphin service:      {}", p.display())?;
        }
        if let Some(ref p) = self.nemo_configure {
            write!(f, "\n  nemo (configure): {}", p.display())?;
        }
        if let Some(ref p) = self.nemo_dev {
            write!(f, "\n  nemo (dev):       {}", p.display())?;
        }
        Ok(())
    }
}

/// Query which context-menu files are currently installed.
pub fn status() -> Result<ContextMenuStatus, ContextMenuError> {
    let paths = Paths::new()?;

    Ok(ContextMenuStatus {
        configure_desktop: paths
            .configure_desktop
            .exists()
            .then_some(paths.configure_desktop),
        dev_desktop: paths.dev_desktop.exists().then_some(paths.dev_desktop),
        nautilus_configure: paths
            .nautilus_configure
            .exists()
            .then_some(paths.nautilus_configure),
        nautilus_dev: paths.nautilus_dev.exists().then_some(paths.nautilus_dev),
        dolphin_service: paths
            .dolphin_service
            .exists()
            .then_some(paths.dolphin_service),
        nemo_configure: paths
            .nemo_configure
            .exists()
            .then_some(paths.nemo_configure),
        nemo_dev: paths.nemo_dev.exists().then_some(paths.nemo_dev),
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

    // -- Freedesktop .desktop actions (always) --------------------------------
    fs::create_dir_all(&paths.applications_dir)?;
    fs::write(
        &paths.configure_desktop,
        configure_desktop_entry(&interpreter),
    )?;
    if cfg!(feature = "dev") {
        fs::write(&paths.dev_desktop, dev_desktop_entry(&interpreter))?;
    } else if paths.dev_desktop.exists() {
        let _ = fs::remove_file(&paths.dev_desktop);
    }

    // -- Nautilus scripts -----------------------------------------------------
    fs::create_dir_all(&paths.nautilus_scripts_dir)?;
    fs::write(
        &paths.nautilus_configure,
        nautilus_configure_content(&interpreter),
    )?;
    fs::set_permissions(&paths.nautilus_configure, fs::Permissions::from_mode(0o755))?;
    if cfg!(feature = "dev") {
        fs::write(&paths.nautilus_dev, nautilus_dev_content(&interpreter))?;
        fs::set_permissions(&paths.nautilus_dev, fs::Permissions::from_mode(0o755))?;
    } else if paths.nautilus_dev.exists() {
        let _ = fs::remove_file(&paths.nautilus_dev);
    }

    // -- Dolphin service menu -------------------------------------------------
    fs::create_dir_all(&paths.dolphin_services_dir)?;
    fs::write(
        &paths.dolphin_service,
        dolphin_service_content(&interpreter),
    )?;

    // -- Nemo actions ---------------------------------------------------------
    fs::create_dir_all(&paths.nemo_actions_dir)?;
    fs::write(&paths.nemo_configure, nemo_configure_content(&interpreter))?;
    if cfg!(feature = "dev") {
        fs::write(&paths.nemo_dev, nemo_dev_content(&interpreter))?;
    } else if paths.nemo_dev.exists() {
        let _ = fs::remove_file(&paths.nemo_dev);
    }

    // -- Refresh --------------------------------------------------------------
    update_desktop_database(&paths.applications_dir);

    status()
}

/// Remove all installed context-menu entries.
pub fn uninstall() -> Result<(), ContextMenuError> {
    let paths = Paths::new()?;

    let all_files: &[&Path] = &[
        &paths.configure_desktop,
        &paths.dev_desktop,
        &paths.nautilus_configure,
        &paths.nautilus_dev,
        &paths.dolphin_service,
        &paths.nemo_configure,
        &paths.nemo_dev,
    ];

    if !all_files.iter().any(|p| p.exists()) {
        return Err(ContextMenuError::NotInstalled);
    }

    for path in all_files {
        if path.exists() {
            fs::remove_file(path)?;
        }
    }

    // Also clean up the old file names from before the dev action was added.
    let legacy: &[&str] = &["rine-configure.desktop"];
    for name in legacy {
        let old = paths.dolphin_services_dir.join(name);
        if old.exists() {
            let _ = fs::remove_file(old);
        }
    }

    update_desktop_database(&paths.applications_dir);

    Ok(())
}
