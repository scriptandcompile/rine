#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rine_config_lib::{self as lib, AppConfig, VersionOption, WindowsVersion};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::{AppHandle, DragDropEvent, Emitter, Manager, State, WindowEvent};

/// Holds the exe path passed as a CLI argument.
struct ExePath(Mutex<Option<String>>);

#[tauri::command]
fn get_exe_path(state: State<'_, ExePath>) -> Option<String> {
    state.0.lock().unwrap().clone()
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

fn main() {
    // First non-flag argument is the exe path (e.g. `rine-config /path/to/app.exe`)
    let exe_path = std::env::args().nth(1).filter(|arg| !arg.starts_with('-'));

    tauri::Builder::default()
        .manage(ExePath(Mutex::new(exe_path)))
        .setup(|app| {
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
                    if let WindowEvent::DragDrop(DragDropEvent::Drop { paths, .. }) = event
                        && let Some(exe_path) = paths.iter().find(|p| is_exe_path(p))
                    {
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
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_exe_path,
            set_menu_enabled,
            get_config,
            save_config_cmd,
            get_config_path,
            get_windows_versions,
            launch_exe,
            pick_folder,
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
fn relaunch_rine_config_with_exe(exe_path: &Path) -> Result<(), String> {
    let config_bin = std::env::current_exe()
        .map_err(|e| format!("failed to resolve current executable: {e}"))?;
    std::process::Command::new(&config_bin)
        .arg(exe_path)
        .spawn()
        .map_err(|e| format!("failed to spawn {}: {e}", config_bin.display()))?;
    Ok(())
}
