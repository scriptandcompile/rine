#![allow(unsafe_op_in_unsafe_fn)]

//! Shared architecture-neutral user32 (Windows windowing) logic for rine.
//!
//! This crate contains the window manager, class registry, message queue,
//! window text operations, and the winit backend. Both the 32-bit and 64-bit
//! user32 DLL wrappers delegate all logic here; they only supply the
//! architecture-specific `extern "C"` / `extern "win64"` ABI shims and pass
//! in a callback for wnd_proc invocations.

pub mod backend;
pub mod class_registration;
pub mod message_queue;
pub mod window_manager;
pub mod window_text;

pub use class_registration::{register_class, unregister_class};
pub use message_queue::{
    def_window_proc, dispatch_message, get_message, peek_message, post_message, post_quit_message,
    send_message, translate_message,
};
pub use window_manager::{create_window, destroy_window, show_window, update_window};
pub use window_text::{
    get_window_text_a, get_window_text_length, get_window_text_w, set_window_text,
};
