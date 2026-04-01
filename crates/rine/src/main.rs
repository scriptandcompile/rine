mod cli;
mod commands;
mod compat;
mod config;
mod integration;
mod loader;
mod pe;
mod subsys;

use std::process::ExitCode;

use clap::Parser;
use tracing::error;

use crate::cli::Cli;
use crate::commands::*;

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

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
        error!("no .exe path provided");
        eprintln!("Usage: rine <EXE_PATH> [EXE_ARGS]...");
        return ExitCode::FAILURE;
    };

    // Handle `--config`: print/create the per-app config and exit.
    if cli.show_config {
        return show_config(exe_path);
    }

    match run(exe_path, &cli) {
        Ok(infallible) => match infallible {},
        Err(e) => {
            error!("{e}");
            ExitCode::FAILURE
        }
    }
}
