//! Winit-based native window backend for user32.
//!
//! Manages host OS windows via winit, bridging between the emulated Windows
//! window state (stored in rine-types globals) and actual screen windows.
//! Shared by both 32-bit and 64-bit user32 wrappers.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::OnceLock;

use rine_types::windows::*;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::{Window, WindowBuilder, WindowId};

pub fn user32_debug_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var("RINE_USER32_DEBUG")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
            .unwrap_or(false)
    })
}

fn native_backend_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        if let Ok(v) = std::env::var("RINE_USER32_NATIVE_BACKEND") {
            return v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes");
        }
        cfg!(target_pointer_width = "64")
    })
}

pub fn debug_log_backend(msg: impl AsRef<str>) {
    if user32_debug_enabled() {
        eprintln!("[user32/backend] {}", msg.as_ref());
    }
}

struct WinitBackend {
    event_loop: EventLoop<()>,
    windows: HashMap<Hwnd, Window>,
    reverse: HashMap<WindowId, Hwnd>,
}

impl WinitBackend {
    fn new() -> Self {
        Self {
            event_loop: EventLoop::new(),
            windows: HashMap::new(),
            reverse: HashMap::new(),
        }
    }

    fn create_window(&mut self, hwnd: Hwnd, state: &WindowState) -> Result<(), String> {
        let width = state.rect.right.saturating_sub(state.rect.left).max(1) as f64;
        let height = state.rect.bottom.saturating_sub(state.rect.top).max(1) as f64;

        let window = WindowBuilder::new()
            .with_title(state.title.clone())
            .with_visible(state.visible)
            .with_inner_size(LogicalSize::new(width, height))
            .with_position(LogicalPosition::new(
                state.rect.left as f64,
                state.rect.top as f64,
            ))
            .build(&self.event_loop)
            .map_err(|e| format!("failed to create window: {e}"))?;

        let id = window.id();
        self.reverse.insert(id, hwnd);
        self.windows.insert(hwnd, window);
        debug_log_backend(format!(
            "create_window hwnd={:#x} title=\"{}\" visible={}",
            hwnd.as_raw(),
            state.title,
            state.visible
        ));
        Ok(())
    }

    fn destroy_window(&mut self, hwnd: Hwnd) {
        if let Some(window) = self.windows.remove(&hwnd) {
            self.reverse.remove(&window.id());
            debug_log_backend(format!("destroy_window hwnd={:#x}", hwnd.as_raw()));
        }
    }

    fn set_visible(&self, hwnd: Hwnd, visible: bool) {
        if let Some(window) = self.windows.get(&hwnd) {
            window.set_visible(visible);
        }
    }

    fn set_title(&self, hwnd: Hwnd, title: &str) {
        if let Some(window) = self.windows.get(&hwnd) {
            window.set_title(title);
        }
    }

    fn request_redraw(&self, hwnd: Hwnd) {
        if let Some(window) = self.windows.get(&hwnd) {
            window.request_redraw();
        }
    }

    fn pump_messages(&mut self) -> Vec<Msg> {
        let mut out = Vec::new();
        let reverse = &self.reverse;

        self.event_loop.run_return(|event, _target, control_flow| {
            // Poll once and drain pending events without blocking.
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent { window_id, event } => {
                    let Some(hwnd) = reverse.get(&window_id).copied() else {
                        return;
                    };

                    match event {
                        WindowEvent::CloseRequested => {
                            debug_log_backend(format!(
                                "winit CloseRequested hwnd={:#x} -> WM_CLOSE",
                                hwnd.as_raw()
                            ));
                            out.push(Msg {
                                hwnd,
                                message: window_message::WM_CLOSE,
                                w_param: 0,
                                l_param: 0,
                                time: 0,
                                pt: Point::default(),
                            })
                        }
                        WindowEvent::Destroyed => out.push(Msg {
                            hwnd,
                            message: window_message::WM_DESTROY,
                            w_param: 0,
                            l_param: 0,
                            time: 0,
                            pt: Point::default(),
                        }),
                        WindowEvent::Resized(size) => {
                            let l_param =
                                ((size.height as isize) << 16) | ((size.width as isize) & 0xFFFF);
                            out.push(Msg {
                                hwnd,
                                message: window_message::WM_SIZE,
                                w_param: 0,
                                l_param,
                                time: 0,
                                pt: Point::default(),
                            });
                        }
                        WindowEvent::Moved(pos) => {
                            let x = (pos.x as i16) as u16 as usize;
                            let y = (pos.y as i16) as u16 as usize;
                            out.push(Msg {
                                hwnd,
                                message: window_message::WM_MOVE,
                                w_param: 0,
                                l_param: ((y << 16) | x) as isize,
                                time: 0,
                                pt: Point { x: pos.x, y: pos.y },
                            });
                        }
                        WindowEvent::Focused(true) => out.push(Msg {
                            hwnd,
                            message: window_message::WM_SETFOCUS,
                            w_param: 0,
                            l_param: 0,
                            time: 0,
                            pt: Point::default(),
                        }),
                        WindowEvent::Focused(false) => out.push(Msg {
                            hwnd,
                            message: window_message::WM_KILLFOCUS,
                            w_param: 0,
                            l_param: 0,
                            time: 0,
                            pt: Point::default(),
                        }),
                        _ => {}
                    }
                }
                Event::RedrawRequested(window_id) => {
                    if let Some(hwnd) = reverse.get(&window_id).copied() {
                        out.push(Msg {
                            hwnd,
                            message: window_message::WM_PAINT,
                            w_param: 0,
                            l_param: 0,
                            time: 0,
                            pt: Point::default(),
                        });
                    }
                }
                Event::MainEventsCleared | Event::RedrawEventsCleared => {
                    // End this pump pass after one complete event-cycle.
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            }
        });

        out
    }
}

thread_local! {
    static WINIT_BACKEND: RefCell<Option<WinitBackend>> = const { RefCell::new(None) };
}

fn with_backend<R>(f: impl FnOnce(&mut WinitBackend) -> R) -> Option<R> {
    if !native_backend_enabled() {
        debug_log_backend("native backend disabled; using emulated user32 only");
        return None;
    }

    WINIT_BACKEND.with(|backend| {
        let mut backend = backend.borrow_mut();
        if backend.is_none() {
            let created = std::panic::catch_unwind(WinitBackend::new).ok()?;
            *backend = Some(created);
        }

        backend.as_mut().map(f)
    })
}

pub fn create_native_window(hwnd: Hwnd, state: &WindowState) {
    let _ = with_backend(|backend| backend.create_window(hwnd, state));
}

pub fn destroy_native_window(hwnd: Hwnd) {
    let _ = with_backend(|backend| backend.destroy_window(hwnd));
}

pub fn set_native_visibility(hwnd: Hwnd, visible: bool) {
    let _ = with_backend(|backend| backend.set_visible(hwnd, visible));
}

pub fn set_native_title(hwnd: Hwnd, title: &str) {
    let _ = with_backend(|backend| backend.set_title(hwnd, title));
}

pub fn request_native_redraw(hwnd: Hwnd) {
    let _ = with_backend(|backend| backend.request_redraw(hwnd));
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
