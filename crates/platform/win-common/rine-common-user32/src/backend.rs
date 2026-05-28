//! Native user32 backend orchestration.
//!
//! This module exposes the backend-facing API used by the rest of user32 and
//! selects either the native winit backend or an emulated backend at runtime.

use std::any::Any;
use std::cell::RefCell;
use std::sync::OnceLock;

use rine_types::windows::*;

mod emulated;
mod native;

use emulated::EmulatedBackend;
use native::NativeBackend;

pub(super) trait WindowBackend {
    fn create_window(&mut self, hwnd: HWND, state: &WindowState);
    fn destroy_window(&mut self, hwnd: HWND);
    fn set_visible(&mut self, hwnd: HWND, visible: bool);
    fn set_title(&mut self, hwnd: HWND, title: &str);
    fn request_redraw(&mut self, hwnd: HWND);
    fn set_window_rect(&mut self, hwnd: HWND, x: i32, y: i32, width: i32, height: i32);
    fn pump_messages(&mut self) -> Vec<Msg>;
}

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

fn host_socket_path() -> Option<String> {
    std::env::var("RINE_WINDOW_HOST_SOCKET").ok()
}

thread_local! {
    static WINIT_BACKEND: RefCell<BackendState> = const { RefCell::new(BackendState::Uninitialized) };
}

enum BackendState {
    Uninitialized,
    Available(Box<dyn WindowBackend>),
    Failed,
}

fn panic_payload_message(payload: &(dyn Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        return (*message).to_owned();
    }
    "unknown backend initialization panic".to_owned()
}

fn with_backend<R>(f: impl FnOnce(&mut dyn WindowBackend) -> R) -> Option<R> {
    WINIT_BACKEND.with(|backend| {
        let mut backend = backend.borrow_mut();
        if matches!(*backend, BackendState::Uninitialized) {
            if let Some(socket_path) = host_socket_path() {
                match EmulatedBackend::connect(&socket_path) {
                    Ok(created) => {
                        debug_log_backend(format!("using emulated window backend via {}", socket_path));
                        *backend = BackendState::Available(Box::new(created));
                    }
                    Err(error) => {
                        eprintln!("warning: emulated user32 backend unavailable; falling back to native mode: {error}");
                    }
                }
            }

            if matches!(*backend, BackendState::Uninitialized) {
                match std::panic::catch_unwind(NativeBackend::new) {
                    Ok(created) => {
                        *backend = BackendState::Available(Box::new(created));
                    }
                    Err(payload) => {
                        let reason = panic_payload_message(payload.as_ref());
                        eprintln!(
                            "warning: native user32 backend unavailable; falling back to emulated mode: {reason}"
                        );
                        debug_log_backend(
                            "native backend initialization failed; falling back to emulated user32",
                        );
                        *backend = BackendState::Failed;
                        return None;
                    }
                }
            }
        }

        match &mut *backend {
            BackendState::Available(instance) => Some(f(instance.as_mut())),
            BackendState::Failed | BackendState::Uninitialized => None,
        }
    })
}

pub fn create_native_window(hwnd: HWND, state: &WindowState) {
    let _ = with_backend(|backend| backend.create_window(hwnd, state));
}

pub fn destroy_native_window(hwnd: HWND) {
    let _ = with_backend(|backend| backend.destroy_window(hwnd));
}

pub fn set_native_visibility(hwnd: HWND, visible: bool) {
    let _ = with_backend(|backend| backend.set_visible(hwnd, visible));
}

pub fn set_native_title(hwnd: HWND, title: &str) {
    let _ = with_backend(|backend| backend.set_title(hwnd, title));
}

pub fn request_native_redraw(hwnd: HWND) {
    let _ = with_backend(|backend| backend.request_redraw(hwnd));
}

pub fn set_native_window_rect(hwnd: HWND, x: i32, y: i32, width: i32, height: i32) {
    let _ = with_backend(|backend| backend.set_window_rect(hwnd, x, y, width, height));
}

pub fn pump_backend_messages() {
    let messages = with_backend(|backend| backend.pump_messages()).unwrap_or_default();
    if messages.is_empty() {
        return;
    }

    if user32_debug_enabled() {
        for message in &messages {
            debug_log_backend(format!(
                "queue_post hwnd={:#x} msg={:#06x}",
                message.hwnd.as_raw(),
                message.message
            ));
        }
    }

    THREAD_MESSAGE_QUEUE.with(|queue| {
        for message in messages {
            queue.post_message(message);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panic_payload_extracts_string() {
        let msg = "oops".to_string();
        assert_eq!(panic_payload_message(&msg), "oops");
    }

    #[test]
    fn panic_payload_extracts_static_str() {
        assert_eq!(panic_payload_message(&"static error"), "static error");
    }

    #[test]
    fn panic_payload_fallback_for_other_types() {
        assert_eq!(
            panic_payload_message(&42),
            "unknown backend initialization panic"
        );
    }
}
