use std::path::Path;
use std::process::ExitCode;

use tracing::error;

use crate::config::manager::ConfigManager;

/// Print the path and contents of the per-app config. Creates a default
/// config file if one does not yet exist.
pub fn show_config(exe_path: &Path) -> ExitCode {
    let mgr = ConfigManager::new();
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
            Ok(p) => eprintln!("created default config: {}", p.display()),
            Err(e) => {
                error!("{e}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        eprintln!("config: {}", path.display());
    }

    match toml::to_string_pretty(&cfg) {
        Ok(s) => print!("{s}"),
        Err(e) => {
            error!("failed to serialise config: {e}");
            return ExitCode::FAILURE;
        }
    }

    ExitCode::SUCCESS
}
