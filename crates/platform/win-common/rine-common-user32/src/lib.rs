#![allow(unsafe_op_in_unsafe_fn)]

//! Shared architecture-neutral user32 (Windows windowing) logic for rine.
//!
//! This crate contains the window manager, class registry, message queue,
//! window text operations, and the winit backend. Both the 32-bit and 64-bit
//! user32 DLL wrappers delegate all logic here; they only supply the
//! architecture-specific `extern "C"` / `extern "win64"` ABI shims and pass
//! in a callback for wnd_proc invocations.

pub mod accelerator;
pub mod backend;
pub mod char_navigation;
pub mod class_registration;
pub mod clipboard;
pub mod cursor;
pub mod device_context;
pub mod dialogs;
pub mod focus;
pub mod format;
pub mod icon;
pub mod menu;
pub mod message_queue;
pub mod message_registration;
pub mod sound;
pub mod string_resource;
pub mod system_metrics;
pub mod window_lifecycle;
pub mod window_long;
pub mod window_text;

pub use char_navigation::{char_next_a, char_prev_a};
pub use class_registration::unregister_class;
pub use device_context::{get_dc, get_window_dc, release_dc};
pub use focus::{get_focus, get_last_active_popup, set_focus, set_foreground_window};
pub use message_queue::{
    def_window_proc, dispatch_message, get_message, is_dialog_message, peek_message, post_message,
    post_quit_message, send_message, translate_message,
};
pub use message_registration::register_window_message;
pub use window_lifecycle::{
    child_window_from_point, create_window, destroy_window, enable_window, get_client_rect,
    invalidate_rect, is_iconic, move_window, screen_to_client, set_window_control_id, show_window,
    update_window,
};
pub use window_long::set_window_long_a;
pub use window_text::{
    get_window_text_a, get_window_text_length, get_window_text_w, set_window_text,
};
