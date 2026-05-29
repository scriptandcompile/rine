//! Native user32 backend orchestration.
//!
//! This module exposes the backend-facing API used by the rest of user32 and
//! selects either the native winit backend or an emulated backend at compile time.

use rine_types::windows::*;

#[cfg(all(feature = "backend-native", feature = "backend-emulated"))]
compile_error!("backend-native and backend-emulated cannot both be enabled");

#[cfg(not(any(feature = "backend-native", feature = "backend-emulated")))]
compile_error!("either backend-native or backend-emulated must be enabled");

mod contracts;
#[cfg(feature = "backend-emulated")]
mod emulated;
mod logging;
#[cfg(feature = "backend-native")]
mod native;
mod runtime;

use contracts::WindowBackend;
use logging::{debug_log_backend, user32_debug_enabled};
use runtime::with_backend;

use crate::queue_core::THREAD_MESSAGE_QUEUE;

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
