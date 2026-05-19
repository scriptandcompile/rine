//! Winit-based native window backend for user32.
//!
//! Manages host OS windows via winit, bridging between the emulated Windows
//! window state (stored in rine-types globals) and actual screen windows.
//! Shared by both 32-bit and 64-bit user32 wrappers.

use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rine_channel::{HostWindowCommand, HostWindowEvent, HostWindowSender};
use rine_types::windows::*;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::platform::pump_events::EventLoopExtPumpEvents;
use winit::window::{Window, WindowId};

use crate::accelerator;

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
        !user32_debug_enabled()
    })
}

pub fn debug_log_backend(msg: impl AsRef<str>) {
    if user32_debug_enabled() {
        eprintln!("[user32/backend] {}", msg.as_ref());
    }
}

fn host_socket_path() -> Option<String> {
    std::env::var("RINE_WINDOW_HOST_SOCKET").ok()
}

struct WinitBackend {
    event_loop: EventLoop<()>,
    state: WinitBackendState,
}

struct WinitBackendState {
    windows: HashMap<HWND, Window>,
    reverse: HashMap<WindowId, HWND>,
    pending_windows: HashMap<HWND, PendingWindow>,
    ready_for_windows: bool,
}

struct PendingWindow {
    state: WindowState,
    redraw_requested: bool,
}

struct HostedBackend {
    sender: Arc<Mutex<HostWindowSender>>,
    events: mpsc::Receiver<HostWindowEvent>,
}

#[allow(clippy::large_enum_variant)]
enum Backend {
    Local(WinitBackend),
    Hosted(HostedBackend),
}

impl WinitBackend {
    fn new() -> Self {
        Self {
            event_loop: EventLoop::new().expect("failed to initialize winit event loop"),
            state: WinitBackendState {
                windows: HashMap::new(),
                reverse: HashMap::new(),
                pending_windows: HashMap::new(),
                ready_for_windows: false,
            },
        }
    }

    fn create_window(&mut self, hwnd: HWND, state: &WindowState) -> Result<(), String> {
        self.state.pending_windows.insert(
            hwnd,
            PendingWindow {
                state: state.clone(),
                redraw_requested: false,
            },
        );
        Ok(())
    }

    fn destroy_window(&mut self, hwnd: HWND) {
        self.state.pending_windows.remove(&hwnd);
        if let Some(window) = self.state.windows.remove(&hwnd) {
            self.state.reverse.remove(&window.id());
            debug_log_backend(format!("destroy_window hwnd={:#x}", hwnd.as_raw()));
        }
    }

    fn set_visible(&self, hwnd: HWND, visible: bool) {
        if let Some(window) = self.state.windows.get(&hwnd) {
            window.set_visible(visible);
        }
    }

    fn set_title(&mut self, hwnd: HWND, title: &str) {
        if let Some(window) = self.state.windows.get(&hwnd) {
            window.set_title(title);
        }
        if let Some(pending) = self.state.pending_windows.get_mut(&hwnd) {
            pending.state.title = title.to_owned();
        }
    }

    fn request_redraw(&mut self, hwnd: HWND) {
        if let Some(window) = self.state.windows.get(&hwnd) {
            window.request_redraw();
        }
        if let Some(pending) = self.state.pending_windows.get_mut(&hwnd) {
            pending.redraw_requested = true;
        }
    }

    fn pump_messages(&mut self) -> Vec<Msg> {
        let mut out = Vec::new();
        let mut app = WinitPumpApp {
            state: &mut self.state,
            out: &mut out,
        };
        self.event_loop
            .pump_app_events(Some(Duration::ZERO), &mut app);

        out
    }
}

impl WinitBackendState {
    fn create_window(
        &mut self,
        event_loop: &ActiveEventLoop,
        hwnd: HWND,
        state: &WindowState,
    ) -> Result<(), String> {
        let width = state.rect.right.saturating_sub(state.rect.left).max(1) as f64;
        let height = state.rect.bottom.saturating_sub(state.rect.top).max(1) as f64;

        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_title(state.title.clone())
                    .with_visible(state.visible)
                    .with_inner_size(LogicalSize::new(width, height))
                    .with_position(LogicalPosition::new(
                        state.rect.left as f64,
                        state.rect.top as f64,
                    )),
            )
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

    fn flush_pending_windows(&mut self, event_loop: &ActiveEventLoop) {
        if !self.ready_for_windows || self.pending_windows.is_empty() {
            return;
        }

        let pending = std::mem::take(&mut self.pending_windows);
        for (hwnd, pending_window) in pending {
            match self.create_window(event_loop, hwnd, &pending_window.state) {
                Ok(()) => {
                    if pending_window.redraw_requested
                        && let Some(window) = self.windows.get(&hwnd)
                    {
                        window.request_redraw();
                    }
                }
                Err(error) => {
                    eprintln!(
                        "warning: failed to create native user32 window {:#x}: {error}",
                        hwnd.as_raw()
                    );
                }
            }
        }
    }
}

struct WinitPumpApp<'a> {
    state: &'a mut WinitBackendState,
    out: &'a mut Vec<Msg>,
}

impl ApplicationHandler for WinitPumpApp<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.state.ready_for_windows = true;
        self.state.flush_pending_windows(event_loop);
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.state.ready_for_windows = false;
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.state.flush_pending_windows(event_loop);
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(hwnd) = self.state.reverse.get(&window_id).copied() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                debug_log_backend(format!(
                    "winit CloseRequested hwnd={:#x} -> WM_CLOSE",
                    hwnd.as_raw()
                ));
                self.out.push(Msg {
                    hwnd,
                    message: window_message::WM_CLOSE,
                    w_param: 0,
                    l_param: 0,
                    time: 0,
                    pt: Point::default(),
                });
            }
            WindowEvent::Destroyed => self.out.push(Msg {
                hwnd,
                message: window_message::WM_DESTROY,
                w_param: 0,
                l_param: 0,
                time: 0,
                pt: Point::default(),
            }),
            WindowEvent::Resized(size) => {
                let l_param = ((size.height as isize) << 16) | ((size.width as isize) & 0xFFFF);
                self.out.push(Msg {
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
                self.out.push(Msg {
                    hwnd,
                    message: window_message::WM_MOVE,
                    w_param: 0,
                    l_param: ((y << 16) | x) as isize,
                    time: 0,
                    pt: Point { x: pos.x, y: pos.y },
                });
            }
            WindowEvent::Focused(true) => self.out.push(Msg {
                hwnd,
                message: window_message::WM_SETFOCUS,
                w_param: 0,
                l_param: 0,
                time: 0,
                pt: Point::default(),
            }),
            WindowEvent::Focused(false) => self.out.push(Msg {
                hwnd,
                message: window_message::WM_KILLFOCUS,
                w_param: 0,
                l_param: 0,
                time: 0,
                pt: Point::default(),
            }),
            WindowEvent::ModifiersChanged(modifiers) => {
                let state = modifiers.state();
                accelerator::update_modifier_state(
                    state.shift_key(),
                    state.control_key(),
                    state.alt_key(),
                );
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state != ElementState::Pressed {
                    return;
                }

                let PhysicalKey::Code(code) = event.physical_key else {
                    return;
                };

                let Some(vk) = key_code_to_virtual_key(code) else {
                    return;
                };

                accelerator::update_modifier_key(vk, true);

                let message = if accelerator::is_alt_down() {
                    window_message::WM_SYSKEYDOWN
                } else {
                    window_message::WM_KEYDOWN
                };

                self.out.push(Msg {
                    hwnd,
                    message,
                    w_param: vk as usize,
                    l_param: 0,
                    time: 0,
                    pt: Point::default(),
                });
            }
            WindowEvent::RedrawRequested => self.out.push(Msg {
                hwnd,
                message: window_message::WM_PAINT,
                w_param: 0,
                l_param: 0,
                time: 0,
                pt: Point::default(),
            }),
            _ => {}
        }
    }
}

fn key_code_to_virtual_key(code: KeyCode) -> Option<u16> {
    let vk = match code {
        KeyCode::Backspace => virtual_key::VK_BACK,
        KeyCode::Tab => virtual_key::VK_TAB,
        KeyCode::Enter => virtual_key::VK_RETURN,
        KeyCode::ShiftLeft | KeyCode::ShiftRight => virtual_key::VK_SHIFT,
        KeyCode::ControlLeft | KeyCode::ControlRight => virtual_key::VK_CONTROL,
        KeyCode::AltLeft | KeyCode::AltRight => virtual_key::VK_MENU,
        KeyCode::Pause => virtual_key::VK_PAUSE,
        KeyCode::CapsLock => virtual_key::VK_CAPITAL,
        KeyCode::Escape => virtual_key::VK_ESCAPE,
        KeyCode::Space => virtual_key::VK_SPACE,
        KeyCode::PageUp => virtual_key::VK_PRIOR,
        KeyCode::PageDown => virtual_key::VK_NEXT,
        KeyCode::End => virtual_key::VK_END,
        KeyCode::Home => virtual_key::VK_HOME,
        KeyCode::ArrowLeft => virtual_key::VK_LEFT,
        KeyCode::ArrowUp => virtual_key::VK_UP,
        KeyCode::ArrowRight => virtual_key::VK_RIGHT,
        KeyCode::ArrowDown => virtual_key::VK_DOWN,
        KeyCode::Insert => virtual_key::VK_INSERT,
        KeyCode::Delete => virtual_key::VK_DELETE,
        KeyCode::Digit0 => virtual_key::VK_0,
        KeyCode::Digit1 => virtual_key::VK_1,
        KeyCode::Digit2 => virtual_key::VK_2,
        KeyCode::Digit3 => virtual_key::VK_3,
        KeyCode::Digit4 => virtual_key::VK_4,
        KeyCode::Digit5 => virtual_key::VK_5,
        KeyCode::Digit6 => virtual_key::VK_6,
        KeyCode::Digit7 => virtual_key::VK_7,
        KeyCode::Digit8 => virtual_key::VK_8,
        KeyCode::Digit9 => virtual_key::VK_9,
        KeyCode::KeyA => virtual_key::VK_A,
        KeyCode::KeyB => virtual_key::VK_B,
        KeyCode::KeyC => virtual_key::VK_C,
        KeyCode::KeyD => virtual_key::VK_D,
        KeyCode::KeyE => virtual_key::VK_E,
        KeyCode::KeyF => virtual_key::VK_F,
        KeyCode::KeyG => virtual_key::VK_G,
        KeyCode::KeyH => virtual_key::VK_H,
        KeyCode::KeyI => virtual_key::VK_I,
        KeyCode::KeyJ => virtual_key::VK_J,
        KeyCode::KeyK => virtual_key::VK_K,
        KeyCode::KeyL => virtual_key::VK_L,
        KeyCode::KeyM => virtual_key::VK_M,
        KeyCode::KeyN => virtual_key::VK_N,
        KeyCode::KeyO => virtual_key::VK_O,
        KeyCode::KeyP => virtual_key::VK_P,
        KeyCode::KeyQ => virtual_key::VK_Q,
        KeyCode::KeyR => virtual_key::VK_R,
        KeyCode::KeyS => virtual_key::VK_S,
        KeyCode::KeyT => virtual_key::VK_T,
        KeyCode::KeyU => virtual_key::VK_U,
        KeyCode::KeyV => virtual_key::VK_V,
        KeyCode::KeyW => virtual_key::VK_W,
        KeyCode::KeyX => virtual_key::VK_X,
        KeyCode::KeyY => virtual_key::VK_Y,
        KeyCode::KeyZ => virtual_key::VK_Z,
        KeyCode::F1 => virtual_key::VK_F1,
        KeyCode::F2 => virtual_key::VK_F2,
        KeyCode::F3 => virtual_key::VK_F3,
        KeyCode::F4 => virtual_key::VK_F4,
        KeyCode::F5 => virtual_key::VK_F5,
        KeyCode::F6 => virtual_key::VK_F6,
        KeyCode::F7 => virtual_key::VK_F7,
        KeyCode::F8 => virtual_key::VK_F8,
        KeyCode::F9 => virtual_key::VK_F9,
        KeyCode::F10 => virtual_key::VK_F10,
        KeyCode::F11 => virtual_key::VK_F11,
        KeyCode::F12 => virtual_key::VK_F12,
        _ => return None,
    };

    Some(vk)
}

impl HostedBackend {
    fn connect(socket_path: &str) -> Result<Self, String> {
        let sender = HostWindowSender::connect(std::path::Path::new(socket_path))
            .map_err(|e| format!("failed to connect to window host: {e}"))?;
        let mut event_reader = sender
            .try_clone()
            .map_err(|e| format!("failed to clone host window channel: {e}"))?;
        let (event_tx, event_rx) = mpsc::channel();

        std::thread::spawn(move || {
            loop {
                match event_reader.recv_event() {
                    Ok(event) => {
                        if event_tx.send(event).is_err() {
                            break;
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                    Err(e) => {
                        if user32_debug_enabled() {
                            eprintln!("[user32/backend] window host event stream closed: {e}");
                        }
                        break;
                    }
                }
            }
        });

        Ok(Self {
            sender: Arc::new(Mutex::new(sender)),
            events: event_rx,
        })
    }

    fn send(&self, command: HostWindowCommand) {
        let Ok(mut sender) = self.sender.lock() else {
            return;
        };
        if let Err(error) = sender.send_command(&command) {
            eprintln!(
                "warning: window host command failed; native backend falling back to emulated behavior: {error}"
            );
        }
    }

    fn create_window(&self, hwnd: HWND, state: &WindowState) {
        self.send(HostWindowCommand::CreateWindow {
            runtime_hwnd: hwnd.as_raw() as u64,
            title: state.title.clone(),
            rect: state.rect,
            visible: state.visible,
            style: state.style,
            ex_style: state.ex_style,
        });
    }

    fn destroy_window(&self, hwnd: HWND) {
        self.send(HostWindowCommand::DestroyWindow {
            runtime_hwnd: hwnd.as_raw() as u64,
        });
    }

    fn set_visible(&self, hwnd: HWND, visible: bool) {
        self.send(HostWindowCommand::SetVisible {
            runtime_hwnd: hwnd.as_raw() as u64,
            visible,
        });
    }

    fn set_title(&self, hwnd: HWND, title: &str) {
        self.send(HostWindowCommand::SetTitle {
            runtime_hwnd: hwnd.as_raw() as u64,
            title: title.to_owned(),
        });
    }

    fn request_redraw(&self, hwnd: HWND) {
        self.send(HostWindowCommand::RequestRedraw {
            runtime_hwnd: hwnd.as_raw() as u64,
        });
    }

    fn pump_messages(&mut self) -> Vec<Msg> {
        let mut out = Vec::new();
        while let Ok(event) = self.events.try_recv() {
            match event {
                HostWindowEvent::Created { .. } => {}
                HostWindowEvent::CloseRequested { runtime_hwnd } => out.push(Msg {
                    hwnd: HWND::from_raw(runtime_hwnd as usize),
                    message: window_message::WM_CLOSE,
                    w_param: 0,
                    l_param: 0,
                    time: 0,
                    pt: Point::default(),
                }),
                HostWindowEvent::Destroyed { runtime_hwnd } => out.push(Msg {
                    hwnd: HWND::from_raw(runtime_hwnd as usize),
                    message: window_message::WM_DESTROY,
                    w_param: 0,
                    l_param: 0,
                    time: 0,
                    pt: Point::default(),
                }),
                HostWindowEvent::Resized {
                    runtime_hwnd,
                    width,
                    height,
                } => out.push(Msg {
                    hwnd: HWND::from_raw(runtime_hwnd as usize),
                    message: window_message::WM_SIZE,
                    w_param: 0,
                    l_param: ((height as isize) << 16) | ((width as isize) & 0xFFFF),
                    time: 0,
                    pt: Point::default(),
                }),
                HostWindowEvent::Moved { runtime_hwnd, x, y } => {
                    let x_word = (x as i16) as u16 as usize;
                    let y_word = (y as i16) as u16 as usize;
                    out.push(Msg {
                        hwnd: HWND::from_raw(runtime_hwnd as usize),
                        message: window_message::WM_MOVE,
                        w_param: 0,
                        l_param: ((y_word << 16) | x_word) as isize,
                        time: 0,
                        pt: Point { x, y },
                    });
                }
                HostWindowEvent::Focused {
                    runtime_hwnd,
                    focused,
                } => out.push(Msg {
                    hwnd: HWND::from_raw(runtime_hwnd as usize),
                    message: if focused {
                        window_message::WM_SETFOCUS
                    } else {
                        window_message::WM_KILLFOCUS
                    },
                    w_param: 0,
                    l_param: 0,
                    time: 0,
                    pt: Point::default(),
                }),
                HostWindowEvent::RedrawRequested { runtime_hwnd } => out.push(Msg {
                    hwnd: HWND::from_raw(runtime_hwnd as usize),
                    message: window_message::WM_PAINT,
                    w_param: 0,
                    l_param: 0,
                    time: 0,
                    pt: Point::default(),
                }),
            }
        }
        out
    }
}

thread_local! {
    static WINIT_BACKEND: RefCell<BackendState> = const { RefCell::new(BackendState::Uninitialized) };
}

#[allow(clippy::large_enum_variant)]
enum BackendState {
    Uninitialized,
    Available(Backend),
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

fn with_backend<R>(f: impl FnOnce(&mut Backend) -> R) -> Option<R> {
    WINIT_BACKEND.with(|backend| {
        let mut backend = backend.borrow_mut();
        if matches!(*backend, BackendState::Uninitialized) {
            if let Some(socket_path) = host_socket_path() {
                match HostedBackend::connect(&socket_path) {
                    Ok(created) => {
                        debug_log_backend(format!("using hosted window backend via {}", socket_path));
                        *backend = BackendState::Available(Backend::Hosted(created));
                    }
                    Err(error) => {
                        eprintln!("warning: hosted user32 backend unavailable; falling back to local mode: {error}");
                    }
                }
            }

            if matches!(*backend, BackendState::Uninitialized) {
                if !native_backend_enabled() {
                    debug_log_backend("native backend disabled; using emulated user32 only");
                    *backend = BackendState::Failed;
                    return None;
                }

                match std::panic::catch_unwind(WinitBackend::new) {
                    Ok(created) => {
                        *backend = BackendState::Available(Backend::Local(created));
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
            BackendState::Available(instance) => Some(f(instance)),
            BackendState::Failed | BackendState::Uninitialized => None,
        }
    })
}

pub fn create_native_window(hwnd: HWND, state: &WindowState) {
    let _ = with_backend(|backend| match backend {
        Backend::Local(local) => local.create_window(hwnd, state),
        Backend::Hosted(hosted) => {
            hosted.create_window(hwnd, state);
            Ok(())
        }
    });
}

pub fn destroy_native_window(hwnd: HWND) {
    let _ = with_backend(|backend| match backend {
        Backend::Local(local) => local.destroy_window(hwnd),
        Backend::Hosted(hosted) => hosted.destroy_window(hwnd),
    });
}

pub fn set_native_visibility(hwnd: HWND, visible: bool) {
    let _ = with_backend(|backend| match backend {
        Backend::Local(local) => local.set_visible(hwnd, visible),
        Backend::Hosted(hosted) => hosted.set_visible(hwnd, visible),
    });
}

pub fn set_native_title(hwnd: HWND, title: &str) {
    let _ = with_backend(|backend| match backend {
        Backend::Local(local) => local.set_title(hwnd, title),
        Backend::Hosted(hosted) => hosted.set_title(hwnd, title),
    });
}

pub fn request_native_redraw(hwnd: HWND) {
    let _ = with_backend(|backend| match backend {
        Backend::Local(local) => local.request_redraw(hwnd),
        Backend::Hosted(hosted) => hosted.request_redraw(hwnd),
    });
}

pub fn pump_backend_messages() {
    let messages = with_backend(|backend| match backend {
        Backend::Local(local) => local.pump_messages(),
        Backend::Hosted(hosted) => hosted.pump_messages(),
    })
    .unwrap_or_default();
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
    use winit::keyboard::KeyCode;

    // -- key_code_to_virtual_key --

    #[test]
    fn maps_backspace() {
        assert_eq!(key_code_to_virtual_key(KeyCode::Backspace), Some(0x08));
    }

    #[test]
    fn maps_tab() {
        assert_eq!(key_code_to_virtual_key(KeyCode::Tab), Some(0x09));
    }

    #[test]
    fn maps_enter() {
        assert_eq!(key_code_to_virtual_key(KeyCode::Enter), Some(0x0D));
    }

    #[test]
    fn maps_escape() {
        assert_eq!(key_code_to_virtual_key(KeyCode::Escape), Some(0x1B));
    }

    #[test]
    fn maps_space() {
        assert_eq!(key_code_to_virtual_key(KeyCode::Space), Some(0x20));
    }

    #[test]
    fn maps_caps_lock() {
        assert_eq!(key_code_to_virtual_key(KeyCode::CapsLock), Some(0x14));
    }

    #[test]
    fn maps_modifier_keys() {
        assert_eq!(key_code_to_virtual_key(KeyCode::ShiftLeft), Some(0x10));
        assert_eq!(key_code_to_virtual_key(KeyCode::ShiftRight), Some(0x10));
        assert_eq!(key_code_to_virtual_key(KeyCode::ControlLeft), Some(0x11));
        assert_eq!(key_code_to_virtual_key(KeyCode::ControlRight), Some(0x11));
        assert_eq!(key_code_to_virtual_key(KeyCode::AltLeft), Some(0x12));
        assert_eq!(key_code_to_virtual_key(KeyCode::AltRight), Some(0x12));
    }

    #[test]
    fn maps_navigation_keys() {
        assert_eq!(key_code_to_virtual_key(KeyCode::PageUp), Some(0x21));
        assert_eq!(key_code_to_virtual_key(KeyCode::PageDown), Some(0x22));
        assert_eq!(key_code_to_virtual_key(KeyCode::End), Some(0x23));
        assert_eq!(key_code_to_virtual_key(KeyCode::Home), Some(0x24));
        assert_eq!(key_code_to_virtual_key(KeyCode::ArrowLeft), Some(0x25));
        assert_eq!(key_code_to_virtual_key(KeyCode::ArrowUp), Some(0x26));
        assert_eq!(key_code_to_virtual_key(KeyCode::ArrowRight), Some(0x27));
        assert_eq!(key_code_to_virtual_key(KeyCode::ArrowDown), Some(0x28));
    }

    #[test]
    fn maps_insert_delete() {
        assert_eq!(key_code_to_virtual_key(KeyCode::Insert), Some(0x2D));
        assert_eq!(key_code_to_virtual_key(KeyCode::Delete), Some(0x2E));
    }

    #[test]
    fn maps_pause() {
        assert_eq!(key_code_to_virtual_key(KeyCode::Pause), Some(0x13));
    }

    #[test]
    fn maps_digit_keys() {
        assert_eq!(key_code_to_virtual_key(KeyCode::Digit0), Some(0x30));
        assert_eq!(key_code_to_virtual_key(KeyCode::Digit1), Some(0x31));
        assert_eq!(key_code_to_virtual_key(KeyCode::Digit2), Some(0x32));
        assert_eq!(key_code_to_virtual_key(KeyCode::Digit3), Some(0x33));
        assert_eq!(key_code_to_virtual_key(KeyCode::Digit4), Some(0x34));
        assert_eq!(key_code_to_virtual_key(KeyCode::Digit5), Some(0x35));
        assert_eq!(key_code_to_virtual_key(KeyCode::Digit6), Some(0x36));
        assert_eq!(key_code_to_virtual_key(KeyCode::Digit7), Some(0x37));
        assert_eq!(key_code_to_virtual_key(KeyCode::Digit8), Some(0x38));
        assert_eq!(key_code_to_virtual_key(KeyCode::Digit9), Some(0x39));
    }

    #[test]
    fn maps_letter_keys() {
        assert_eq!(key_code_to_virtual_key(KeyCode::KeyA), Some(0x41));
        assert_eq!(key_code_to_virtual_key(KeyCode::KeyB), Some(0x42));
        assert_eq!(key_code_to_virtual_key(KeyCode::KeyZ), Some(0x5A));
    }

    #[test]
    fn maps_function_keys() {
        assert_eq!(key_code_to_virtual_key(KeyCode::F1), Some(0x70));
        assert_eq!(key_code_to_virtual_key(KeyCode::F2), Some(0x71));
        assert_eq!(key_code_to_virtual_key(KeyCode::F3), Some(0x72));
        assert_eq!(key_code_to_virtual_key(KeyCode::F4), Some(0x73));
        assert_eq!(key_code_to_virtual_key(KeyCode::F5), Some(0x74));
        assert_eq!(key_code_to_virtual_key(KeyCode::F6), Some(0x75));
        assert_eq!(key_code_to_virtual_key(KeyCode::F7), Some(0x76));
        assert_eq!(key_code_to_virtual_key(KeyCode::F8), Some(0x77));
        assert_eq!(key_code_to_virtual_key(KeyCode::F9), Some(0x78));
        assert_eq!(key_code_to_virtual_key(KeyCode::F10), Some(0x79));
        assert_eq!(key_code_to_virtual_key(KeyCode::F11), Some(0x7A));
        assert_eq!(key_code_to_virtual_key(KeyCode::F12), Some(0x7B));
    }

    #[test]
    fn unmapped_key_returns_none() {
        assert_eq!(key_code_to_virtual_key(KeyCode::F13), None);
        assert_eq!(key_code_to_virtual_key(KeyCode::AudioVolumeUp), None);
        assert_eq!(key_code_to_virtual_key(KeyCode::BrowserHome), None);
        assert_eq!(key_code_to_virtual_key(KeyCode::MediaPlayPause), None);
    }

    // -- panic_payload_message --

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
