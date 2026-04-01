use std::process::ExitCode;

use tracing::error;

use crate::integration::desktop;

pub fn desktop_status_cmd() -> ExitCode {
    match desktop::status() {
        Ok(s) => {
            println!("desktop integration: {s}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            error!("{e}");
            ExitCode::FAILURE
        }
    }
}

pub fn install_desktop_cmd() -> ExitCode {
    match desktop::install(None) {
        Ok(interpreter) => {
            println!(
                "installed .desktop entry (interpreter: {})",
                interpreter.display()
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            error!("{e}");
            ExitCode::FAILURE
        }
    }
}

pub fn uninstall_desktop_cmd() -> ExitCode {
    match desktop::uninstall() {
        Ok(()) => {
            println!("removed .desktop entry and MIME type");
            ExitCode::SUCCESS
        }
        Err(e) => {
            error!("{e}");
            ExitCode::FAILURE
        }
    }
}
