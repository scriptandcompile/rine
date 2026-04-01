//! Command-line argument parsing.

use std::path::PathBuf;

use clap::Parser;

/// rine — Windows PE executable loader for Linux.
#[derive(Parser, Debug)]
#[command(name = "rine", version, about)]
pub struct Cli {
    /// Path to the Windows .exe to run.
    pub exe_path: PathBuf,

    /// Show or create the per-app config file instead of running the exe.
    #[arg(long = "config")]
    pub show_config: bool,

    /// Arguments to pass to the Windows executable.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub exe_args: Vec<String>,
}
