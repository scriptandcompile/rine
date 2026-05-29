use std::sync::OnceLock;

pub(super) fn user32_debug_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var("RINE_USER32_DEBUG")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
            .unwrap_or(false)
    })
}

pub(super) fn debug_log_backend(msg: impl AsRef<str>) {
    if user32_debug_enabled() {
        eprintln!("[user32/backend] {}", msg.as_ref());
    }
}
