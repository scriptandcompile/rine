#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod registry_ui;

use rine_config_lib::{self as lib, AppConfig, VersionOption, WindowsVersion};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::{AppHandle, DragDropEvent, Emitter, Manager, State, WindowEvent};

/// Holds the exe path passed as a CLI argument.
struct OpenPath(Mutex<Option<String>>);

#[tauri::command]
fn get_exe_path(state: State<'_, OpenPath>) -> Option<String> {
    state.0.lock().unwrap().clone()
}

#[derive(Serialize)]
struct OpenedConfig {
    exe_path: Option<String>,
    config_path: String,
    config: AppConfig,
}

#[tauri::command]
fn open_config_target(path: String) -> Result<OpenedConfig, String> {
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
fn set_menu_enabled(app: AppHandle, id: String, enabled: bool) -> Result<(), String> {
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
fn get_config(exe_path: String) -> Result<AppConfig, String> {
    lib::load_config(Path::new(&exe_path)).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_config_cmd(exe_path: String, config: AppConfig) -> Result<(), String> {
    lib::save_config(Path::new(&exe_path), &config)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn save_config_file(config_path: String, config: AppConfig) -> Result<(), String> {
    let path = PathBuf::from(&config_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create {}: {e}", parent.display()))?;
    }
    let content = toml::to_string_pretty(&config).map_err(|e| format!("serialize error: {e}"))?;
    std::fs::write(&path, content).map_err(|e| format!("failed to write {}: {e}", path.display()))
}

#[tauri::command]
fn get_config_path(exe_path: String) -> String {
    lib::config_path(Path::new(&exe_path))
        .to_string_lossy()
        .into_owned()
}

#[tauri::command]
fn get_windows_versions() -> Vec<VersionOption> {
    WindowsVersion::all()
        .iter()
        .map(|v| VersionOption {
            value: serde_json::to_value(v).unwrap_or_default(),
            label: v.label().to_string(),
        })
        .collect()
}

#[derive(Serialize)]
struct LaunchOutput {
    stdout: String,
    stderr: String,
    exit_code: i32,
}

#[tauri::command]
fn launch_exe(exe_path: String) -> Result<LaunchOutput, String> {
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
fn pick_folder(start_dir: Option<String>) -> Option<String> {
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
fn get_registry_export(
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
fn get_registry_key(
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
fn update_registry_value(
    _exe_path: String,
    key_path: String,
    value_name: String,
    _new_value: String,
) -> Result<(), String> {
    // Prevent modification of locked values
    if registry_ui::is_locked_registry_value(&key_path, &value_name) {
        return Err(
            "This registry value is locked to the Windows version and cannot be modified"
                .to_string(),
        );
    }

    // TODO: Implement value update and save to registry JSON
    Err("Registry update not yet implemented".to_string())
}

fn main() {
    // First supported file argument is an open target path (.exe or config .toml).
    // Runtime/tooling flags may be prepended, so only accept known target types.
    let open_path = std::env::args().skip(1).find(|arg| {
        let path = Path::new(arg);
        is_exe_path(path) || is_config_toml_path(path)
    });

    tauri::Builder::default()
        .manage(OpenPath(Mutex::new(open_path)))
        .setup(|app| {
            let open_item = MenuItemBuilder::with_id("open-configuration", "Open Configuration...")
                .accelerator("CmdOrCtrl+O")
                .build(app)?;
            let save_item = MenuItemBuilder::with_id("save", "Save")
                .accelerator("CmdOrCtrl+S")
                .enabled(false)
                .build(app)?;
            let reset_item = MenuItemBuilder::with_id("reset", "Reset to Defaults")
                .enabled(false)
                .build(app)?;
            let exit_item = MenuItemBuilder::with_id("exit", "Exit")
                .accelerator("CmdOrCtrl+Q")
                .build(app)?;
            let file_menu = SubmenuBuilder::new(app, "File")
                .item(&open_item)
                .separator()
                .item(&save_item)
                .separator()
                .item(&reset_item)
                .separator()
                .item(&exit_item)
                .build()?;
            let menu = MenuBuilder::new(app).item(&file_menu).build()?;
            app.set_menu(menu)?;

            let handle = app.handle().clone();
            app.on_menu_event(move |_app, event| match event.id().as_ref() {
                "open-configuration" => {
                    if let Some(path) = pick_open_configuration_path() {
                        if let Err(err) = relaunch_rine_config_with_path(&path) {
                            eprintln!("rine-config: failed to relaunch from open target: {err}");
                            return;
                        }
                        handle.exit(0);
                    }
                }
                "save" => {
                    let _ = handle.emit("menu-save", ());
                }
                "reset" => {
                    let _ = handle.emit("menu-reset", ());
                }
                "exit" => {
                    std::process::exit(0);
                }
                _ => {}
            });

            if let Some(window) = app.get_webview_window("main") {
                let drop_handle = app.handle().clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::DragDrop(DragDropEvent::Drop { paths, .. }) = event {
                        if let Some(config_path) = paths.iter().find(|p| is_config_toml_path(p)) {
                            if let Err(err) = relaunch_rine_config_with_path(config_path) {
                                eprintln!(
                                    "rine-config: failed to relaunch from dropped config file: {err}"
                                );
                                return;
                            }
                            drop_handle.exit(0);
                            return;
                        }

                        if let Some(exe_path) = paths.iter().find(|p| is_exe_path(p)) {
                        let should_relaunch = matches!(
                            rfd::MessageDialog::new()
                                .set_title("Relaunch rine-config?")
                                .set_description(format!(
                                    "Relaunch rine-config with this executable?\n\n{}",
                                    exe_path.display()
                                ))
                                .set_buttons(rfd::MessageButtons::YesNo)
                                .set_level(rfd::MessageLevel::Info)
                                .show(),
                            rfd::MessageDialogResult::Yes
                        );
                        if !should_relaunch {
                            return;
                        }

                        if let Err(err) = relaunch_rine_config_with_exe(exe_path) {
                            eprintln!("rine-config: failed to relaunch from dropped file: {err}");
                            return;
                        }
                        drop_handle.exit(0);
                    }
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_exe_path,
            set_menu_enabled,
            open_config_target,
            get_config,
            save_config_cmd,
            save_config_file,
            get_config_path,
            get_windows_versions,
            launch_exe,
            pick_folder,
            get_registry_export,
            get_registry_key,
            update_registry_value,
        ])
        .run(tauri::generate_context!())
        .expect("error while running rine-config");
}

fn is_exe_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("exe"))
        .unwrap_or(false)
}

fn is_config_toml_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("toml"))
        .unwrap_or(false)
}

fn pick_open_configuration_path() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Open Configuration or Executable")
        .add_filter("Supported", &["exe", "toml"])
        .add_filter("Windows Executable", &["exe"])
        .add_filter("Config File", &["toml"])
        .pick_file()
}

fn relaunch_rine_config_with_exe(exe_path: &Path) -> Result<(), String> {
    relaunch_rine_config_with_path(exe_path)
}

fn relaunch_rine_config_with_path(path: &Path) -> Result<(), String> {
    let config_bin = std::env::current_exe()
        .map_err(|e| format!("failed to resolve current executable: {e}"))?;
    std::process::Command::new(&config_bin)
        .arg(path)
        .spawn()
        .map_err(|e| format!("failed to spawn {}: {e}", config_bin.display()))?;
    Ok(())
}
