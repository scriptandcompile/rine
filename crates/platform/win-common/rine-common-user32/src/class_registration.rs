//! Window class registration — shared logic for RegisterClass(Ex)A/W.

use rine_types::windows::*;

/// Register a window class by name.
///
/// Returns 1 on success, 0 if the name is empty.
pub fn register_class(name: String, class: WindowClass) -> u16 {
    if name.is_empty() {
        return 0;
    }
    WINDOW_CLASS_REGISTRY.register(name, class);
    1
}

/// Unregister a previously registered window class.
///
/// Returns 1 if found and removed, 0 if not found.
pub fn unregister_class(name: &str) -> i32 {
    if WINDOW_CLASS_REGISTRY.unregister(name) {
        1
    } else {
        0
    }
}
