mod cli;
mod commands;
mod compat;
mod config;
mod integration;
mod loader;
mod pe;
mod subsys;

use std::process::ExitCode;

use clap::{CommandFactory, Parser};
use tracing::error;

use crate::cli::Cli;
use crate::commands::*;

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_writer(std::io::stderr)
        .init();

    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => e.exit(),
    };

    // Handle binfmt_misc commands (no exe_path required).
    if cli.binfmt_status {
        return binfmt_status_cmd();
    }
    if cli.install_binfmt {
        return install_binfmt_cmd();
    }
    if cli.uninstall_binfmt {
        return uninstall_binfmt_cmd();
    }
    if cli.desktop_status {
        return desktop_status_cmd();
    }
    if cli.install_desktop {
        return install_desktop_cmd();
    }
    if cli.uninstall_desktop {
        return uninstall_desktop_cmd();
    }
    if cli.context_menu_status {
        return context_menu_status_cmd();
    }
    if cli.install_context_menu {
        return install_context_menu_cmd();
    }
    if cli.uninstall_context_menu {
        return uninstall_context_menu_cmd();
    }

    let Some(ref exe_path) = cli.exe_path else {
        let _ = Cli::command().print_help();
        return ExitCode::FAILURE;
    };

    // Handle `--config`: print/create the per-app config and exit.
    if cli.show_config {
        return show_config(exe_path);
    }

    // Warn if --dev is used without the dev feature.
    #[cfg(not(feature = "dev"))]
    if cli.dev {
        error!(
            "--dev requires rine to be built with the `dev` feature: cargo build --features dev"
        );
        return ExitCode::FAILURE;
    }

    // --dev: launch rine-dev which will spawn us back as a child with
    // piped stdout/stderr for output capture.
    #[cfg(feature = "dev")]
    if cli.dev && std::env::var_os("RINE_DEV_SOCKET").is_none() {
        let dev_bin = std::env::current_exe()
            .ok()
            .and_then(|p| {
                let sibling = p.with_file_name("rine-dev");
                sibling.is_file().then_some(sibling)
            })
            .unwrap_or_else(|| std::path::PathBuf::from("rine-dev"));

        match std::process::Command::new(&dev_bin)
            .arg("--exe")
            .arg(exe_path)
            .status()
        {
            Ok(status) => {
                return if status.success() {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::FAILURE
                };
            }
            Err(e) => {
                error!(
                    "failed to launch rine-dev ({}): {e}\n\
                     hint: make sure rine-dev is built (`cargo build -p rine-dev`)",
                    dev_bin.display()
                );
                return ExitCode::FAILURE;
            }
        }
    }

    match run(exe_path, &cli) {
        Ok(infallible) => match infallible {},
        Err(e) => {
            error!("{e}");
            ExitCode::FAILURE
        }
    }
}
