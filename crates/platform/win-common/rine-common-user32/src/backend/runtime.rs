use std::cell::RefCell;

use super::{WindowBackend, debug_log_backend};

#[cfg(feature = "backend-emulated")]
use super::emulated::EmulatedBackend;
#[cfg(feature = "backend-native")]
use super::native::NativeBackend;

#[cfg(feature = "backend-emulated")]
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

#[cfg(feature = "backend-native")]
fn panic_payload_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        return (*message).to_owned();
    }
    "unknown backend initialization panic".to_owned()
}

#[cfg(feature = "backend-native")]
fn initialize_backend() -> BackendState {
    match std::panic::catch_unwind(NativeBackend::new) {
        Ok(created) => BackendState::Available(Box::new(created)),
        Err(payload) => {
            let reason = panic_payload_message(payload.as_ref());
            eprintln!("warning: native user32 backend unavailable: {reason}");
            debug_log_backend("native backend initialization failed");
            BackendState::Failed
        }
    }
}

#[cfg(feature = "backend-emulated")]
fn initialize_backend() -> BackendState {
    let Some(socket_path) = host_socket_path() else {
        eprintln!(
            "warning: emulated user32 backend selected but RINE_WINDOW_HOST_SOCKET is not set"
        );
        debug_log_backend("emulated backend selected but host socket path is missing");
        return BackendState::Failed;
    };

    match EmulatedBackend::connect(&socket_path) {
        Ok(created) => {
            debug_log_backend(format!("using emulated window backend via {}", socket_path));
            BackendState::Available(Box::new(created))
        }
        Err(error) => {
            eprintln!("warning: emulated user32 backend unavailable: {error}");
            debug_log_backend("emulated backend initialization failed");
            BackendState::Failed
        }
    }
}

pub(super) fn with_backend<R>(f: impl FnOnce(&mut dyn WindowBackend) -> R) -> Option<R> {
    WINIT_BACKEND.with(|backend| {
        let mut backend = backend.borrow_mut();
        if matches!(*backend, BackendState::Uninitialized) {
            *backend = initialize_backend();
        }

        match &mut *backend {
            BackendState::Available(instance) => Some(f(instance.as_mut())),
            BackendState::Failed | BackendState::Uninitialized => None,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "backend-native")]
    #[test]
    fn panic_payload_extracts_string() {
        let msg = "oops".to_string();
        assert_eq!(panic_payload_message(&msg), "oops");
    }

    #[cfg(feature = "backend-native")]
    #[test]
    fn panic_payload_extracts_static_str() {
        assert_eq!(panic_payload_message(&"static error"), "static error");
    }

    #[cfg(feature = "backend-native")]
    #[test]
    fn panic_payload_fallback_for_other_types() {
        assert_eq!(
            panic_payload_message(&42),
            "unknown backend initialization panic"
        );
    }
}
