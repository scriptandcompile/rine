#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rine_config_lib::{self as lib, AppConfig, VersionOption, WindowsVersion};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::State;

/// Holds the exe path passed as a CLI argument.
struct ExePath(Mutex<Option<String>>);

#[tauri::command]
fn get_exe_path(state: State<'_, ExePath>) -> Option<String> {
    state.0.lock().unwrap().clone()
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

#[tauri::command]
fn launch_exe(exe_path: String) -> Result<String, String> {
    let rine_bin = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("rine")))
        .unwrap_or_else(|| PathBuf::from("rine"));

    let output = std::process::Command::new(&rine_bin)
        .arg(&exe_path)
        .output()
        .map_err(|e| format!("Failed to launch rine: {e}"))?;

    let mut result = String::new();
    if !output.stdout.is_empty() {
        result.push_str(&String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(&String::from_utf8_lossy(&output.stderr));
    }
    if result.is_empty() {
        result = format!(
            "Process exited with code {}",
            output.status.code().unwrap_or(-1)
        );
    }
    Ok(result)
}

fn main() {
    // First non-flag argument is the exe path (e.g. `rine-config /path/to/app.exe`)
    let exe_path = std::env::args().nth(1).and_then(|arg| {
        if arg.starts_with('-') {
            None
        } else {
            Some(arg)
        }
    });

    tauri::Builder::default()
        .manage(ExePath(Mutex::new(exe_path)))
        .invoke_handler(tauri::generate_handler![
            get_exe_path,
            get_config,
            save_config_cmd,
            get_config_path,
            get_windows_versions,
            launch_exe,
        ])
        .run(tauri::generate_context!())
        .expect("error while running rine-config");
}
