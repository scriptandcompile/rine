use crate::file_kind::{is_config_toml_path, is_exe_path};
use crate::registry_ui;
use rine_config_lib::{self as lib, AppConfig, VersionOption, WindowsVersion};
use rine_types::registry::RegistryValue;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

/// Holds the exe path passed as a CLI argument.
pub struct OpenPath(pub Mutex<Option<String>>);

/// Tracks whether the next close request should be allowed.
pub struct CloseApproval(pub Mutex<bool>);

#[tauri::command]
pub fn get_exe_path(state: State<'_, OpenPath>) -> Option<String> {
    state.0.lock().unwrap().clone()
}

#[derive(Serialize)]
pub struct OpenedConfig {
    exe_path: Option<String>,
    config_path: String,
    config: AppConfig,
}

#[tauri::command]
pub fn open_config_target(path: String) -> Result<OpenedConfig, String> {
    let input = PathBuf::from(path);
    if is_exe_path(&input) {
        let cfg = lib::load_config(&input).map_err(|e| e.to_string())?;
        let cfg_path = lib::config_path(&input);
        let exe_path = input
            .canonicalize()
            .unwrap_or_else(|_| input.clone())
            .to_string_lossy()
            .into_owned();

        // Initialize the registry for this exe and version (creates defaults if needed).
        rine_types::registry::init_registry_for_app(&input, cfg.windows_version);

        return Ok(OpenedConfig {
            exe_path: Some(exe_path),
            config_path: cfg_path.to_string_lossy().into_owned(),
            config: cfg,
        });
    }

    if is_config_toml_path(&input) {
        let cfg_path = input.canonicalize().unwrap_or(input.clone());
        let content = std::fs::read_to_string(&cfg_path)
            .map_err(|e| format!("failed to read config file {}: {e}", cfg_path.display()))?;
        let cfg = toml::from_str::<AppConfig>(&content)
            .map_err(|e| format!("failed to parse config file {}: {e}", cfg_path.display()))?;
        return Ok(OpenedConfig {
            exe_path: None,
            config_path: cfg_path.to_string_lossy().into_owned(),
            config: cfg,
        });
    }

    Err("unsupported file type: select a .exe or a .toml config file".to_string())
}

#[tauri::command]
pub fn set_menu_enabled(app: AppHandle, id: String, enabled: bool) -> Result<(), String> {
    let menu = app.menu().ok_or("no menu")?;
    if let Some(item) = menu.get(&id) {
        item.as_menuitem()
            .ok_or("not a menu item")?
            .set_enabled(enabled)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn request_app_exit(
    app: AppHandle,
    close_approval: State<'_, CloseApproval>,
) -> Result<(), String> {
    if let Ok(mut approved) = close_approval.0.lock() {
        *approved = true;
    }

    if let Some(window) = app.get_webview_window("main") {
        window.close().map_err(|e| e.to_string())?;
    } else {
        app.exit(0);
    }

    Ok(())
}

#[tauri::command]
pub fn get_config(exe_path: String) -> Result<AppConfig, String> {
    lib::load_config(Path::new(&exe_path)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_config_cmd(exe_path: String, config: AppConfig) -> Result<(), String> {
    lib::save_config(Path::new(&exe_path), &config)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_config_file(config_path: String, config: AppConfig) -> Result<(), String> {
    let path = PathBuf::from(&config_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create {}: {e}", parent.display()))?;
    }
    let content = toml::to_string_pretty(&config).map_err(|e| format!("serialize error: {e}"))?;
    std::fs::write(&path, content).map_err(|e| format!("failed to write {}: {e}", path.display()))
}

#[tauri::command]
pub fn get_config_path(exe_path: String) -> String {
    lib::config_path(Path::new(&exe_path))
        .to_string_lossy()
        .into_owned()
}

#[tauri::command]
pub fn get_windows_versions() -> Vec<VersionOption> {
    WindowsVersion::all()
        .iter()
        .map(|v| VersionOption {
            value: serde_json::to_value(v).unwrap_or_default(),
            label: v.label().to_string(),
        })
        .collect()
}

#[derive(Serialize)]
pub struct LaunchOutput {
    stdout: String,
    stderr: String,
    exit_code: i32,
}

#[tauri::command]
pub fn launch_exe(exe_path: String) -> Result<LaunchOutput, String> {
    let rine_bin = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("rine")))
        .unwrap_or_else(|| PathBuf::from("rine"));

    let output = std::process::Command::new(&rine_bin)
        .arg(&exe_path)
        .output()
        .map_err(|e| format!("Failed to launch rine: {e}"))?;

    Ok(LaunchOutput {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        exit_code: output.status.code().unwrap_or(-1),
    })
}

#[tauri::command]
pub fn pick_folder(start_dir: Option<String>) -> Option<String> {
    let mut dialog = rfd::FileDialog::new();
    if let Some(ref dir) = start_dir {
        let expanded = if let Some(rest) = dir.strip_prefix("~/") {
            if let Some(home) = std::env::var_os("HOME") {
                PathBuf::from(home).join(rest)
            } else {
                PathBuf::from(dir)
            }
        } else {
            PathBuf::from(dir)
        };
        if !expanded.is_dir() {
            let _ = std::fs::create_dir_all(&expanded);
        }
        if expanded.is_dir() {
            dialog = dialog.set_directory(&expanded);
        } else if let Some(home) = std::env::var_os("HOME") {
            dialog = dialog.set_directory(PathBuf::from(home));
        }
    }
    dialog
        .pick_folder()
        .map(|p| p.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn get_registry_export(
    exe_path: String,
    windows_version: Option<WindowsVersion>,
) -> Result<serde_json::Value, String> {
    let exe_path = Path::new(&exe_path);

    let version = resolve_registry_windows_version(exe_path, windows_version)?;

    // Ensure the process-wide registry reflects the selected Windows version.
    rine_types::registry::reinit_registry_for_app(exe_path, version);

    let export = registry_ui::get_registry_export_for_ui();
    serde_json::to_value(&export).map_err(|e| format!("Failed to serialize registry: {e}"))
}

#[tauri::command]
pub fn get_registry_key(
    exe_path: String,
    key_path: String,
    windows_version: Option<WindowsVersion>,
) -> Result<serde_json::Value, String> {
    let exe_path = Path::new(&exe_path);

    let version = resolve_registry_windows_version(exe_path, windows_version)?;

    // Ensure the process-wide registry reflects the selected Windows version.
    rine_types::registry::reinit_registry_for_app(exe_path, version);

    let key = registry_ui::get_registry_key_for_ui(&key_path)
        .ok_or_else(|| format!("Registry key not found: {key_path}"))?;
    serde_json::to_value(&key).map_err(|e| format!("Failed to serialize registry key: {e}"))
}

fn resolve_registry_windows_version(
    exe_path: &Path,
    windows_version: Option<WindowsVersion>,
) -> Result<WindowsVersion, String> {
    if let Some(version) = windows_version {
        return Ok(version);
    }

    let config = lib::load_config(exe_path).map_err(|e| e.to_string())?;
    Ok(config.windows_version)
}

#[tauri::command]
pub fn update_registry_value(
    exe_path: String,
    key_path: String,
    value_name: String,
    new_value: String,
    windows_version: Option<WindowsVersion>,
) -> Result<(), String> {
    let exe_path = Path::new(&exe_path);
    let version = resolve_registry_windows_version(exe_path, windows_version)?;

    // Ensure the process-wide registry reflects the selected Windows version.
    rine_types::registry::reinit_registry_for_app(exe_path, version);

    // Prevent modification of locked values.
    if registry_ui::is_locked_registry_value(&key_path, &value_name) {
        return Err(
            "This registry value is locked to the Windows version and cannot be modified"
                .to_string(),
        );
    }

    let (root_hkey, _root_name, subpath) = registry_ui::parse_registry_ui_path(&key_path)
        .ok_or_else(|| format!("Invalid registry key path: {key_path}"))?;

    let existing_value = rine_types::registry::registry_store()
        .with_root(root_hkey, |root| {
            let key = if subpath.is_empty() {
                Some(root)
            } else {
                root.open_subkey(subpath)
            };
            key.and_then(|k| k.get_value(&value_name)).cloned()
        })
        .flatten();

    let parsed_value = parse_registry_value(existing_value.as_ref(), &new_value)?;

    let updated = rine_types::registry::registry_store().with_root_mut(root_hkey, |root| {
        let key = if subpath.is_empty() {
            root
        } else {
            root.create_subkey(subpath)
        };
        key.set_value(value_name, parsed_value);
    });

    if updated.is_none() {
        return Err(format!("Unsupported registry root in path: {key_path}"));
    }

    rine_types::registry::save_registry_for_app(exe_path, version).map(|_| ())
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WinIniScope {
    Global,
    App,
}

#[derive(Serialize)]
pub struct WinIniLoadResult {
    path: String,
    exists: bool,
    content: String,
}

#[tauri::command]
pub fn load_win_ini_text(
    exe_path: Option<String>,
    scope: WinIniScope,
) -> Result<WinIniLoadResult, String> {
    let path = resolve_win_ini_path(exe_path.as_deref(), scope)?;
    let exists = path.exists();
    let content = if exists {
        std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?
    } else {
        String::new()
    };

    Ok(WinIniLoadResult {
        path: path.to_string_lossy().into_owned(),
        exists,
        content,
    })
}

#[tauri::command]
pub fn save_win_ini_text(
    exe_path: Option<String>,
    scope: WinIniScope,
    content: String,
) -> Result<String, String> {
    let path = resolve_win_ini_path(exe_path.as_deref(), scope)?;

    if content.trim().is_empty() {
        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| format!("failed to delete {}: {e}", path.display()))?;
        }
        return Ok(path.to_string_lossy().into_owned());
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create {}: {e}", parent.display()))?;
    }

    std::fs::write(&path, content)
        .map_err(|e| format!("failed to write {}: {e}", path.display()))?;

    Ok(path.to_string_lossy().into_owned())
}

fn resolve_win_ini_path(exe_path: Option<&str>, scope: WinIniScope) -> Result<PathBuf, String> {
    let root = rine_types::config::rine_root();

    match scope {
        WinIniScope::Global => Ok(root.join("win.ini")),
        WinIniScope::App => {
            let exe = exe_path.ok_or_else(|| {
                "An executable path is required for per-app WIN.INI operations".to_string()
            })?;
            let exe = Path::new(exe);
            Ok(root
                .join("apps")
                .join(rine_types::config::app_hash(exe))
                .join("win.ini"))
        }
    }
}

fn parse_registry_value(
    existing: Option<&RegistryValue>,
    input: &str,
) -> Result<RegistryValue, String> {
    let trimmed = input.trim();

    match existing {
        Some(RegistryValue::Dword(_)) => parse_u32(trimmed)
            .map(RegistryValue::Dword)
            .map_err(|e| format!("Invalid REG_DWORD value '{input}': {e}")),
        Some(RegistryValue::Qword(_)) => parse_u64(trimmed)
            .map(RegistryValue::Qword)
            .map_err(|e| format!("Invalid REG_QWORD value '{input}': {e}")),
        Some(RegistryValue::Binary(_)) => parse_binary(trimmed)
            .map(RegistryValue::Binary)
            .map_err(|e| format!("Invalid REG_BINARY value '{input}': {e}")),
        Some(RegistryValue::MultiString(_)) => {
            let values = if trimmed.is_empty() {
                Vec::new()
            } else {
                trimmed
                    .split(';')
                    .map(|part| part.trim().to_string())
                    .filter(|part| !part.is_empty())
                    .collect()
            };
            Ok(RegistryValue::MultiString(values))
        }
        Some(RegistryValue::ExpandString(_)) => Ok(RegistryValue::ExpandString(input.to_string())),
        Some(RegistryValue::String(_)) | None => Ok(RegistryValue::String(input.to_string())),
    }
}

fn parse_u32(value: &str) -> Result<u32, String> {
    if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        return u32::from_str_radix(hex, 16).map_err(|e| e.to_string());
    }
    value.parse::<u32>().map_err(|e| e.to_string())
}

fn parse_u64(value: &str) -> Result<u64, String> {
    if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        return u64::from_str_radix(hex, 16).map_err(|e| e.to_string());
    }
    value.parse::<u64>().map_err(|e| e.to_string())
}

fn parse_binary(value: &str) -> Result<Vec<u8>, String> {
    if value.is_empty() {
        return Ok(Vec::new());
    }

    value
        .split_whitespace()
        .map(|part| u8::from_str_radix(part, 16).map_err(|e| e.to_string()))
        .collect()
}
