use std::process::ExitCode;

use tracing::error;

use crate::integration::context_menu;

pub fn context_menu_status_cmd() -> ExitCode {
    match context_menu::status() {
        Ok(s) => {
            println!("context menu: {s}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            error!("{e}");
            ExitCode::FAILURE
        }
    }
}

pub fn install_context_menu_cmd() -> ExitCode {
    match context_menu::install(None) {
        Ok(s) => {
            println!("installed context menu entries:\n{s}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            error!("{e}");
            ExitCode::FAILURE
        }
    }
}

pub fn uninstall_context_menu_cmd() -> ExitCode {
    match context_menu::uninstall() {
        Ok(()) => {
            println!("removed context menu entries");
            ExitCode::SUCCESS
        }
        Err(e) => {
            error!("{e}");
            ExitCode::FAILURE
        }
    }
}
