#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use rine_channel::{DevEvent, DevReceiver};
use rine_dev_lib::*;
use tauri::{Emitter, Manager, State};

struct AppState(Mutex<StateSnapshot>);

#[tauri::command]
fn get_state(state: State<'_, AppState>) -> StateSnapshot {
    state.0.lock().unwrap().clone()
}

fn main() {
    let socket_path: String = std::env::args()
        .nth(2)
        .or_else(|| {
            std::env::args().enumerate().find_map(|(i, a)| {
                if a == "--socket" {
                    std::env::args().nth(i + 1)
                } else {
                    None
                }
            })
        })
        .expect("usage: rine-dev --socket <path>");

    tauri::Builder::default()
        .manage(AppState(Mutex::new(StateSnapshot::default())))
        .setup(move |app| {
            let socket = std::path::PathBuf::from(&socket_path);
            let handle = app.handle().clone();

            // Background thread: receive events from rine and emit to frontend.
            std::thread::spawn(move || {
                let receiver = match DevReceiver::bind(&socket) {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("rine-dev: failed to bind socket: {e}");
                        return;
                    }
                };

                for result in receiver.into_stream() {
                    match result {
                        Ok(ref event) => {
                            // Update accumulated state.
                            if let Some(state) = handle.try_state::<AppState>() {
                                let mut snap = state.0.lock().unwrap();
                                apply_event(&mut snap, event);
                            }
                            // Forward to frontend.
                            let _ = handle.emit("dev-event", event);
                        }
                        Err(e) => {
                            eprintln!("rine-dev: recv error: {e}");
                            break;
                        }
                    }
                }

                // Connection closed — rine process exited.
                let _ = handle.emit("rine-disconnected", ());
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_state])
        .run(tauri::generate_context!())
        .expect("error while running rine-dev");
}

fn apply_event(snap: &mut StateSnapshot, event: &DevEvent) {
    match event {
        DevEvent::PeLoaded {
            exe_path,
            image_base,
            image_size,
            entry_rva,
            relocation_delta,
            sections,
        } => {
            snap.pe = Some(PeInfo {
                exe_path: exe_path.clone(),
                image_base: *image_base,
                image_size: *image_size,
                entry_rva: *entry_rva,
                relocation_delta: *relocation_delta,
                sections: sections.clone(),
            });
        }
        DevEvent::ConfigLoaded {
            config_path,
            windows_version,
            environment_overrides,
        } => {
            snap.config = Some(ConfigInfo {
                config_path: config_path.clone(),
                windows_version: windows_version.clone(),
                environment_overrides: environment_overrides.clone(),
            });
        }
        DevEvent::ImportsResolved {
            summaries,
            total_resolved,
            total_stubbed,
        } => {
            snap.imports = Some(ImportsInfo {
                summaries: summaries.clone(),
                total_resolved: *total_resolved,
                total_stubbed: *total_stubbed,
            });
        }
        DevEvent::ProcessExited { exit_code } => {
            snap.exited = Some(*exit_code);
        }
    }
}
