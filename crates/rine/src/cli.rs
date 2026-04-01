//! Command-line argument parsing.

use std::path::PathBuf;

use clap::Parser;

/// rine — Windows PE executable loader for Linux.
#[derive(Parser, Debug)]
#[command(name = "rine", version, about)]
pub struct Cli {
    /// Path to the Windows .exe to run.
    pub exe_path: Option<PathBuf>,

    /// Show or create the per-app config file instead of running the exe.
    #[arg(long = "config")]
    pub show_config: bool,

    /// Register rine with binfmt_misc so .exe files can be executed directly.
    /// Requires root.
    #[arg(long)]
    pub install_binfmt: bool,

    /// Remove the binfmt_misc registration for rine. Requires root.
    #[arg(long)]
    pub uninstall_binfmt: bool,

    /// Show the current binfmt_misc registration status.
    #[arg(long)]
    pub binfmt_status: bool,

    /// Install .desktop file and MIME type for opening .exe files from file managers.
    #[arg(long)]
    pub install_desktop: bool,

    /// Remove .desktop file and MIME type registration.
    #[arg(long)]
    pub uninstall_desktop: bool,

    /// Show the current .desktop integration status.
    #[arg(long)]
    pub desktop_status: bool,

    /// Arguments to pass to the Windows executable.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub exe_args: Vec<String>,
}
