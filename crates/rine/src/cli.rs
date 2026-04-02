//! Command-line argument parsing.

use std::path::PathBuf;

use clap::Parser;

/// rine — Windows PE executable loader for Linux.
#[derive(Parser, Debug)]
#[command(
    name = "rine",
    version,
    about,
    override_usage = "rine [OPTIONS] <EXE_PATH> [EXE_ARGS]...\n       rine <--install-binfmt | --uninstall-binfmt | --binfmt-status>\n       rine <--install-desktop | --uninstall-desktop | --desktop-status>\n       rine <--install-context-menu | --uninstall-context-menu | --context-menu-status>"
)]
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

    /// Install right-click context menu entries for file managers.
    #[arg(long)]
    pub install_context_menu: bool,

    /// Remove right-click context menu entries.
    #[arg(long)]
    pub uninstall_context_menu: bool,

    /// Show the current context menu integration status.
    #[arg(long)]
    pub context_menu_status: bool,

    /// Launch the developer dashboard alongside the PE.
    #[arg(long)]
    pub dev: bool,

    /// Arguments to pass to the Windows executable.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub exe_args: Vec<String>,
}
