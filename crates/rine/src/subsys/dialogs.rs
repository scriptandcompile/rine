use std::sync::OnceLock;

use rine_types::config::DialogConfig;

static DIALOG_POLICY: OnceLock<DialogConfig> = OnceLock::new();

/// Initialize dialog policy from app config.
pub fn init_policy(cfg: DialogConfig) {
    let _ = DIALOG_POLICY.set(cfg);
}

/// Get the resolved dialog policy, if initialized.
pub fn policy() -> Option<&'static DialogConfig> {
    DIALOG_POLICY.get()
}
