use std::path::{Path, PathBuf};

pub fn pick_open_configuration_path() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Open Configuration or Executable")
        .add_filter("Supported", &["exe", "toml"])
        .add_filter("Windows Executable", &["exe"])
        .add_filter("Config File", &["toml"])
        .pick_file()
}

pub fn relaunch_rine_config_with_exe(exe_path: &Path) -> Result<(), String> {
    relaunch_rine_config_with_path(exe_path)
}

pub fn relaunch_rine_config_with_path(path: &Path) -> Result<(), String> {
    let config_bin = std::env::current_exe()
        .map_err(|e| format!("failed to resolve current executable: {e}"))?;
    std::process::Command::new(&config_bin)
        .arg(path)
        .spawn()
        .map_err(|e| format!("failed to spawn {}: {e}", config_bin.display()))?;
    Ok(())
}
