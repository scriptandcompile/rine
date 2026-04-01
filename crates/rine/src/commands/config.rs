use std::path::Path;
use std::process::ExitCode;

use tracing::{error, info};

use crate::config::manager::ConfigManager;

/// Open the Tauri config editor for the given exe.
/// Creates a default config file if one does not yet exist, then launches
/// the `rine-config` GUI binary.
pub fn show_config(exe_path: &Path) -> ExitCode {
    let mgr = ConfigManager::new();

    // Ensure the config file exists (create default if missing).
    let cfg = match mgr.load(exe_path) {
        Ok(c) => c,
        Err(e) => {
            error!("{e}");
            return ExitCode::FAILURE;
        }
    };

    let path = mgr.config_path(exe_path);
    if !path.exists() {
        match mgr.save(exe_path, &cfg) {
            Ok(p) => info!("created default config: {}", p.display()),
            Err(e) => {
                error!("{e}");
                return ExitCode::FAILURE;
            }
        }
    }

    // Resolve absolute path so rine-config gets a canonical reference.
    let abs_exe = exe_path
        .canonicalize()
        .unwrap_or_else(|_| exe_path.to_path_buf());

    // Look for rine-config next to the current binary first, then fall back
    // to $PATH.
    let config_bin = std::env::current_exe()
        .ok()
        .and_then(|p| {
            let sibling = p.with_file_name("rine-config");
            sibling.is_file().then_some(sibling)
        })
        .unwrap_or_else(|| "rine-config".into());

    info!(
        "launching {} for {}",
        config_bin.display(),
        abs_exe.display()
    );

    match std::process::Command::new(&config_bin)
        .arg(abs_exe.as_os_str())
        .spawn()
    {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            error!(
                "failed to launch rine-config ({}): {e}\n\
                 hint: make sure rine-config is built (`cargo build -p rine-config`)",
                config_bin.display()
            );
            ExitCode::FAILURE
        }
    }
}
