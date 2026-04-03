#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::{BufRead, Read, Seek};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Mutex;

use rine_channel::{DevEvent, DevReceiver, OutputStream};
use rine_dev_lib::*;
use tauri::{Emitter, Manager, State};

struct AppState(Mutex<StateSnapshot>);

#[tauri::command]
fn get_state(state: State<'_, AppState>) -> StateSnapshot {
    state.0.lock().unwrap().clone()
}

#[tauri::command]
fn save_memory_dump(suggested_name: String, content: String) -> Result<Option<String>, String> {
    let path = rfd::FileDialog::new()
        .set_title("Save Memory Dump")
        .set_file_name(&suggested_name)
        .save_file();

    let Some(path) = path else {
        return Ok(None);
    };

    std::fs::write(&path, content).map_err(|e| format!("failed to write file: {e}"))?;
    Ok(Some(path.to_string_lossy().into_owned()))
}

#[tauri::command]
fn load_memory_snapshot_meta(json_path: String) -> Result<serde_json::Value, String> {
    let json = std::fs::read_to_string(&json_path)
        .map_err(|e| format!("failed to read snapshot metadata: {e}"))?;
    serde_json::from_str::<serde_json::Value>(&json)
        .map_err(|e| format!("failed to parse snapshot metadata: {e}"))
}

#[tauri::command]
fn read_memory_snapshot_chunk(
    bin_path: String,
    offset: u64,
    length: usize,
) -> Result<Vec<u8>, String> {
    let mut file = std::fs::File::open(&bin_path)
        .map_err(|e| format!("failed to open snapshot binary: {e}"))?;
    file.seek(std::io::SeekFrom::Start(offset))
        .map_err(|e| format!("failed to seek snapshot binary: {e}"))?;

    let mut buf = vec![0u8; length];
    let read = file
        .read(&mut buf)
        .map_err(|e| format!("failed to read snapshot binary: {e}"))?;
    buf.truncate(read);
    Ok(buf)
}

/// Parse CLI args, returning (socket_path, exe_path).
fn parse_args() -> (Option<String>, Option<String>) {
    let args: Vec<String> = std::env::args().collect();
    let mut socket = None;
    let mut exe = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--socket" => {
                socket = args.get(i + 1).cloned();
                i += 2;
            }
            "--exe" => {
                exe = args.get(i + 1).cloned();
                i += 2;
            }
            _ => i += 1,
        }
    }
    (socket, exe)
}

fn main() {
    let (socket_arg, exe_arg) = parse_args();

    // Determine socket path: provided explicitly, or generated for a child rine process.
    let socket_path: String = if let Some(s) = socket_arg {
        s
    } else if exe_arg.is_some() {
        let path = std::env::temp_dir().join(format!("rine-dev-{}.sock", std::process::id()));
        path.to_string_lossy().into_owned()
    } else {
        eprintln!("usage: rine-dev --socket <path>  OR  rine-dev --exe <pe-path>");
        std::process::exit(1);
    };

    tauri::Builder::default()
        .manage(AppState(Mutex::new(StateSnapshot::default())))
        .setup(move |app| {
            let socket = PathBuf::from(&socket_path);
            let handle = app.handle().clone();

            // If --exe was provided, spawn rine as a child process with piped output.
            if let Some(ref exe_path) = exe_arg {
                let exe_path = exe_path.clone();
                let socket_str = socket.to_string_lossy().to_string();

                let rine_bin = std::env::current_exe()
                    .ok()
                    .and_then(|p| {
                        let sibling = p.with_file_name("rine");
                        sibling.is_file().then_some(sibling)
                    })
                    .unwrap_or_else(|| PathBuf::from("rine"));

                let pipe_handle = handle.clone();
                std::thread::spawn(move || {
                    spawn_rine_child(&rine_bin, &exe_path, &socket_str, &pipe_handle);
                });
            }

            // Background thread: receive dev events from rine over the socket.
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
                            if let Some(state) = handle.try_state::<AppState>() {
                                let mut snap = state.0.lock().unwrap();
                                apply_event(&mut snap, event);
                            }
                            let _ = handle.emit("dev-event", event);
                        }
                        Err(e) => {
                            eprintln!("rine-dev: recv error: {e}");
                            break;
                        }
                    }
                }

                // Connection closed — rine process exited.
                if let Some(state) = handle.try_state::<AppState>() {
                    let mut snap = state.0.lock().unwrap();
                    if snap.exited.is_none() {
                        snap.exited = Some(-1);
                    }
                }
                let _ = handle.emit("rine-disconnected", ());
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_state,
            save_memory_dump,
            load_memory_snapshot_meta,
            read_memory_snapshot_chunk
        ])
        .run(tauri::generate_context!())
        .expect("error while running rine-dev");
}

/// Spawn rine as a child process with `--dev`, piping stdout/stderr.
fn spawn_rine_child(
    rine_bin: &std::path::Path,
    exe_path: &str,
    socket_path: &str,
    handle: &tauri::AppHandle,
) {
    let mut child = match Command::new(rine_bin)
        .env("RINE_DEV_SOCKET", socket_path)
        .arg("--dev")
        .arg(exe_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("rine-dev: failed to spawn rine: {e}");
            return;
        }
    };

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    // Forward stdout lines.
    let h1 = handle.clone();
    let stdout_thread = std::thread::spawn(move || {
        let reader = std::io::BufReader::new(stdout);
        for line in reader.split(b'\n') {
            match line {
                Ok(data) => {
                    let text = String::from_utf8_lossy(&data).into_owned();
                    let text_nl = format!("{text}\n");
                    if let Some(state) = h1.try_state::<AppState>() {
                        state.0.lock().unwrap().stdout.push_str(&text_nl);
                    }
                    let _ = h1.emit(
                        "dev-event",
                        &DevEvent::OutputData {
                            stream: OutputStream::Stdout,
                            data: text_nl,
                        },
                    );
                }
                Err(_) => break,
            }
        }
    });

    // Forward stderr lines.
    let h2 = handle.clone();
    let stderr_thread = std::thread::spawn(move || {
        let reader = std::io::BufReader::new(stderr);
        for line in reader.split(b'\n') {
            match line {
                Ok(data) => {
                    let text = String::from_utf8_lossy(&data).into_owned();
                    let text_nl = format!("{text}\n");
                    if let Some(state) = h2.try_state::<AppState>() {
                        state.0.lock().unwrap().stderr.push_str(&text_nl);
                    }
                    let _ = h2.emit(
                        "dev-event",
                        &DevEvent::OutputData {
                            stream: OutputStream::Stderr,
                            data: text_nl,
                        },
                    );
                }
                Err(_) => break,
            }
        }
    });

    let _ = stdout_thread.join();
    let _ = stderr_thread.join();
    let _ = child.wait();
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
        DevEvent::OutputData { stream, data } => match stream {
            OutputStream::Stdout => snap.stdout.push_str(data),
            OutputStream::Stderr => snap.stderr.push_str(data),
        },
        DevEvent::HandleCreated {
            handle,
            kind,
            detail,
        } => {
            snap.handles.push(rine_dev_lib::HandleInfo {
                handle: *handle,
                kind: kind.clone(),
                detail: detail.clone(),
                closed: false,
            });
        }
        DevEvent::HandleClosed { handle } => {
            if let Some(h) = snap.handles.iter_mut().find(|h| h.handle == *handle) {
                h.closed = true;
            }
        }
        DevEvent::ThreadCreated {
            handle,
            thread_id,
            entry_point,
        } => {
            snap.threads.push(rine_dev_lib::ThreadInfo {
                handle: *handle,
                thread_id: *thread_id,
                entry_point: *entry_point,
                exit_code: None,
            });
        }
        DevEvent::ThreadExited {
            thread_id,
            exit_code,
        } => {
            if let Some(t) = snap.threads.iter_mut().find(|t| t.thread_id == *thread_id) {
                t.exit_code = Some(*exit_code);
            }
        }
        DevEvent::TlsAllocated { index } => {
            snap.tls_slots.push(*index);
        }
        DevEvent::TlsFreed { index } => {
            snap.tls_slots.retain(|i| i != index);
        }
        DevEvent::MemoryAllocated {
            address,
            size,
            source,
        } => {
            snap.memory_regions.push(rine_dev_lib::MemoryRegionInfo {
                address: *address,
                size: *size,
                source: source.clone(),
                freed: false,
            });
            snap.memory_total_allocated = snap.memory_total_allocated.saturating_add(*size);
            snap.memory_current_usage = snap.memory_current_usage.saturating_add(*size);
            snap.memory_peak_usage = snap.memory_peak_usage.max(snap.memory_current_usage);
        }
        DevEvent::MemoryFreed {
            address,
            size,
            source: _,
        } => {
            let mut freed_size = *size;
            if let Some(region) = snap
                .memory_regions
                .iter_mut()
                .rev()
                .find(|r| r.address == *address && !r.freed)
            {
                region.freed = true;
                freed_size = region.size;
            }

            snap.memory_total_freed = snap.memory_total_freed.saturating_add(freed_size);
            snap.memory_current_usage = snap.memory_current_usage.saturating_sub(freed_size);
        }
        DevEvent::MemorySnapshotReady {
            json_path,
            bin_path,
            region_count,
            total_bytes,
        } => {
            snap.memory_snapshot = Some(rine_dev_lib::MemorySnapshotInfo {
                json_path: json_path.clone(),
                bin_path: bin_path.clone(),
                region_count: *region_count,
                total_bytes: *total_bytes,
            });
        }
        DevEvent::DialogOpened {
            api,
            theme,
            native_backend,
            windows_theme,
        } => {
            snap.dialog_calls.push(rine_dev_lib::DialogCallInfo {
                phase: "opened".to_owned(),
                api: api.clone(),
                theme: theme.clone(),
                native_backend: native_backend.clone(),
                windows_theme: windows_theme.clone(),
                success: None,
                error_code: None,
                selected_path: None,
            });
        }
        DevEvent::DialogResult {
            api,
            theme,
            native_backend,
            windows_theme,
            success,
            error_code,
            selected_path,
        } => {
            snap.dialog_calls.push(rine_dev_lib::DialogCallInfo {
                phase: "result".to_owned(),
                api: api.clone(),
                theme: theme.clone(),
                native_backend: native_backend.clone(),
                windows_theme: windows_theme.clone(),
                success: Some(*success),
                error_code: Some(*error_code),
                selected_path: selected_path.clone(),
            });
        }
    }
}
