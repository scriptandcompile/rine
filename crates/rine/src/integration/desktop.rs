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
const DESKTOP_FILE_NAME: &str = "rine.desktop";
const MIME_TYPES: [&str; 3] = [
    "application/x-dosexec",
    "application/x-ms-dos-executable",
    "application/vnd.microsoft.portable-executable",
];

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

/// Return `$XDG_CONFIG_HOME` or fall back to `$HOME/.config`.
fn xdg_config_home() -> Result<PathBuf, DesktopError> {
    if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(dir));
    }
    if let Ok(home) = std::env::var("HOME") {
        return Ok(PathBuf::from(home).join(".config"));
    }
    Err(DesktopError::NoDataDir)
}

/// Paths where rine installs Freedesktop integration files.
struct Paths {
    desktop_file: PathBuf,
    mime_xml: PathBuf,
    mimeapps_list: PathBuf,
    applications_dir: PathBuf,
    mime_packages_dir: PathBuf,
    mime_dir: PathBuf,
    config_dir: PathBuf,
}

impl Paths {
    fn new() -> Result<Self, DesktopError> {
        let data = xdg_data_home()?;
        let config_dir = xdg_config_home()?;
        let applications_dir = data.join("applications");
        let mime_packages_dir = data.join("mime/packages");
        let mime_dir = data.join("mime");
        Ok(Self {
            desktop_file: applications_dir.join(DESKTOP_FILE_NAME),
            mime_xml: mime_packages_dir.join(format!("{APP_ID}-exe.xml")),
            mimeapps_list: config_dir.join("mimeapps.list"),
            applications_dir,
            mime_packages_dir,
            mime_dir,
            config_dir,
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
         MimeType=application/x-dosexec;application/x-ms-dos-executable;application/vnd.microsoft.portable-executable;\n\
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

fn set_default_handler_with_xdg_mime() -> bool {
    let status = Command::new("xdg-mime")
        .arg("default")
        .arg(DESKTOP_FILE_NAME)
        .args(MIME_TYPES)
        .status();

    matches!(status, Ok(status) if status.success())
}

fn update_default_applications(
    current: &str,
    desktop_file_name: &str,
    mime_types: &[&str],
) -> String {
    let mut lines: Vec<String> = if current.is_empty() {
        Vec::new()
    } else {
        current.lines().map(str::to_string).collect()
    };

    let default_section_idx = lines
        .iter()
        .position(|line| line.trim() == "[Default Applications]");

    let section_start = match default_section_idx {
        Some(index) => index,
        None => {
            if !lines.is_empty() && !lines.last().is_some_and(|line| line.is_empty()) {
                lines.push(String::new());
            }
            lines.push("[Default Applications]".to_string());
            lines.len() - 1
        }
    };

    let section_end = lines
        .iter()
        .enumerate()
        .skip(section_start + 1)
        .find(|(_, line)| line.starts_with('[') && line.ends_with(']'))
        .map(|(idx, _)| idx)
        .unwrap_or(lines.len());

    for mime_type in mime_types {
        let entry = format!("{mime_type}={desktop_file_name};");
        if let Some(relative_idx) = lines[section_start + 1..section_end]
            .iter()
            .position(|line| line.trim_start().starts_with(&format!("{mime_type}=")))
        {
            lines[section_start + 1 + relative_idx] = entry;
        } else {
            lines.insert(section_end, entry);
        }
    }

    let mut output = lines.join("\n");
    if !output.is_empty() {
        output.push('\n');
    }
    output
}

fn prune_default_applications(
    current: &str,
    desktop_file_name: &str,
    mime_types: &[&str],
) -> String {
    let mut lines: Vec<String> = if current.is_empty() {
        Vec::new()
    } else {
        current.lines().map(str::to_string).collect()
    };

    let Some(section_start) = lines
        .iter()
        .position(|line| line.trim() == "[Default Applications]")
    else {
        return current.to_string();
    };

    for mime_type in mime_types {
        let section_end = lines
            .iter()
            .enumerate()
            .skip(section_start + 1)
            .find(|(_, line)| line.starts_with('[') && line.ends_with(']'))
            .map(|(idx, _)| idx)
            .unwrap_or(lines.len());

        if let Some(relative_idx) = lines[section_start + 1..section_end]
            .iter()
            .position(|line| line.trim_start().starts_with(&format!("{mime_type}=")))
        {
            let idx = section_start + 1 + relative_idx;
            let Some((key, value)) = lines[idx].split_once('=') else {
                continue;
            };
            let mut apps = value
                .split(';')
                .filter(|entry| !entry.is_empty())
                .filter(|entry| *entry != desktop_file_name)
                .collect::<Vec<_>>();

            if apps.is_empty() {
                lines.remove(idx);
            } else {
                apps.push("");
                lines[idx] = format!("{key}={}", apps.join(";"));
            }
        }
    }

    let mut output = lines.join("\n");
    if !output.is_empty() {
        output.push('\n');
    }
    output
}

fn set_default_handler(paths: &Paths) -> Result<(), DesktopError> {
    if set_default_handler_with_xdg_mime() {
        return Ok(());
    }

    fs::create_dir_all(&paths.config_dir)?;
    let current = fs::read_to_string(&paths.mimeapps_list).unwrap_or_default();
    let updated = update_default_applications(&current, DESKTOP_FILE_NAME, &MIME_TYPES);
    fs::write(&paths.mimeapps_list, updated)?;
    Ok(())
}

fn remove_default_handler(paths: &Paths) -> Result<(), DesktopError> {
    let current = match fs::read_to_string(&paths.mimeapps_list) {
        Ok(content) => content,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(DesktopError::Io(err)),
    };

    let updated = prune_default_applications(&current, DESKTOP_FILE_NAME, &MIME_TYPES);
    if updated == current {
        return Ok(());
    }

    if updated.trim().is_empty() {
        fs::remove_file(&paths.mimeapps_list)?;
    } else {
        fs::write(&paths.mimeapps_list, updated)?;
    }

    Ok(())
}

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

    // Set rine as the default opener for supported Windows executable MIME types.
    set_default_handler(&paths)?;

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

    remove_default_handler(&paths)?;

    // Refresh databases.
    update_mime_database(&paths.mime_dir);
    update_desktop_database(&paths.applications_dir);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        DESKTOP_FILE_NAME, MIME_TYPES, prune_default_applications, update_default_applications,
    };

    #[test]
    fn add_default_applications_section_when_missing() {
        let updated = update_default_applications(
            "[Added Associations]\ntext/plain=gedit.desktop;\n",
            DESKTOP_FILE_NAME,
            &MIME_TYPES,
        );

        assert!(updated.contains("[Default Applications]"));
        for mime_type in MIME_TYPES {
            assert!(updated.contains(&format!("{mime_type}={DESKTOP_FILE_NAME};")));
        }
    }

    #[test]
    fn replace_existing_default_applications() {
        let current = "[Default Applications]\napplication/x-dosexec=org.example.Other.desktop;\napplication/x-ms-dos-executable=org.example.Other.desktop;\n";
        let updated = update_default_applications(current, DESKTOP_FILE_NAME, &MIME_TYPES[..2]);

        assert!(updated.contains("application/x-dosexec=rine.desktop;"));
        assert!(updated.contains("application/x-ms-dos-executable=rine.desktop;"));
    }

    #[test]
    fn prune_only_rine_from_default_applications() {
        let current = "[Default Applications]\napplication/x-dosexec=rine.desktop;org.example.Other.desktop;\napplication/x-ms-dos-executable=rine.desktop;\napplication/vnd.microsoft.portable-executable=org.example.Other.desktop;rine.desktop;\n";
        let updated = prune_default_applications(current, DESKTOP_FILE_NAME, &MIME_TYPES);

        assert!(updated.contains("application/x-dosexec=org.example.Other.desktop;"));
        assert!(!updated.contains("application/x-ms-dos-executable="));
        assert!(
            updated.contains(
                "application/vnd.microsoft.portable-executable=org.example.Other.desktop;"
            )
        );
    }
}
