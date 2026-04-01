use std::process::ExitCode;

use tracing::error;

use crate::integration::binfmt;

pub fn binfmt_status_cmd() -> ExitCode {
    match binfmt::status() {
        Ok(s) => {
            println!("binfmt_misc: {s}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            error!("{e}");
            ExitCode::FAILURE
        }
    }
}

pub fn install_binfmt_cmd() -> ExitCode {
    match binfmt::install(None) {
        Ok(interpreter) => {
            println!("registered binfmt_misc handler: {}", interpreter.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            error!("{e}");
            ExitCode::FAILURE
        }
    }
}

pub fn uninstall_binfmt_cmd() -> ExitCode {
    match binfmt::uninstall() {
        Ok(()) => {
            println!("removed binfmt_misc handler");
            ExitCode::SUCCESS
        }
        Err(e) => {
            error!("{e}");
            ExitCode::FAILURE
        }
    }
}
