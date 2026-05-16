#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod file_kind;
mod registry_ui;
mod relaunch;

use commands::{
    CloseApproval, OpenPath, get_config, get_config_path, get_exe_path, get_registry_export,
    get_registry_key, get_windows_versions, launch_exe, load_win_ini_text, open_config_target,
    pick_folder, request_app_exit, save_config_cmd, save_config_file, save_win_ini_text,
    set_menu_enabled, update_registry_value,
};
use file_kind::{is_config_toml_path, is_exe_path};
use relaunch::{
    pick_open_configuration_path, relaunch_rine_config_with_exe, relaunch_rine_config_with_path,
};
use std::path::Path;
use std::sync::Mutex;
use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::{DragDropEvent, Emitter, Manager, WindowEvent};

fn main() {
    // First supported file argument is an open target path (.exe or config .toml).
    // Runtime/tooling flags may be prepended, so only accept known target types.
    let open_path = std::env::args().skip(1).find(|arg| {
        let path = Path::new(arg);
        is_exe_path(path) || is_config_toml_path(path)
    });

    tauri::Builder::default()
        .manage(OpenPath(Mutex::new(open_path)))
        .manage(CloseApproval(Mutex::new(false)))
        .setup(|app| {
            let open_item = MenuItemBuilder::with_id("open-configuration", "Open Configuration...")
                .accelerator("CmdOrCtrl+O")
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
            load_win_ini_text,
            save_win_ini_text,
            request_app_exit,
        ])
        .run(tauri::generate_context!())
        .expect("error while running rine-config");
}
