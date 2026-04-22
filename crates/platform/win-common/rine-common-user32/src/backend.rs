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
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::platform::pump_events::EventLoopExtPumpEvents;
use winit::window::{Window, WindowId};

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
    windows: HashMap<Hwnd, Window>,
    reverse: HashMap<WindowId, Hwnd>,
    pending_windows: HashMap<Hwnd, PendingWindow>,
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

    fn create_window(&mut self, hwnd: Hwnd, state: &WindowState) -> Result<(), String> {
        self.state.pending_windows.insert(
            hwnd,
            PendingWindow {
                state: state.clone(),
                redraw_requested: false,
            },
        );
        Ok(())
    }

    fn destroy_window(&mut self, hwnd: Hwnd) {
        self.state.pending_windows.remove(&hwnd);
        if let Some(window) = self.state.windows.remove(&hwnd) {
            self.state.reverse.remove(&window.id());
            debug_log_backend(format!("destroy_window hwnd={:#x}", hwnd.as_raw()));
        }
    }

    fn set_visible(&self, hwnd: Hwnd, visible: bool) {
        if let Some(window) = self.state.windows.get(&hwnd) {
            window.set_visible(visible);
        }
    }

    fn set_title(&mut self, hwnd: Hwnd, title: &str) {
        if let Some(window) = self.state.windows.get(&hwnd) {
            window.set_title(title);
        }
        if let Some(pending) = self.state.pending_windows.get_mut(&hwnd) {
            pending.state.title = title.to_owned();
        }
    }

    fn request_redraw(&mut self, hwnd: Hwnd) {
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
        hwnd: Hwnd,
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

    fn create_window(&self, hwnd: Hwnd, state: &WindowState) {
        self.send(HostWindowCommand::CreateWindow {
            runtime_hwnd: hwnd.as_raw() as u64,
            title: state.title.clone(),
            rect: state.rect,
            visible: state.visible,
            style: state.style,
            ex_style: state.ex_style,
        });
    }

    fn destroy_window(&self, hwnd: Hwnd) {
        self.send(HostWindowCommand::DestroyWindow {
            runtime_hwnd: hwnd.as_raw() as u64,
        });
    }

    fn set_visible(&self, hwnd: Hwnd, visible: bool) {
        self.send(HostWindowCommand::SetVisible {
            runtime_hwnd: hwnd.as_raw() as u64,
            visible,
        });
    }

    fn set_title(&self, hwnd: Hwnd, title: &str) {
        self.send(HostWindowCommand::SetTitle {
            runtime_hwnd: hwnd.as_raw() as u64,
            title: title.to_owned(),
        });
    }

    fn request_redraw(&self, hwnd: Hwnd) {
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
                    hwnd: Hwnd::from_raw(runtime_hwnd as usize),
                    message: window_message::WM_CLOSE,
                    w_param: 0,
                    l_param: 0,
                    time: 0,
                    pt: Point::default(),
                }),
                HostWindowEvent::Destroyed { runtime_hwnd } => out.push(Msg {
                    hwnd: Hwnd::from_raw(runtime_hwnd as usize),
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
                    hwnd: Hwnd::from_raw(runtime_hwnd as usize),
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
                        hwnd: Hwnd::from_raw(runtime_hwnd as usize),
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
                    hwnd: Hwnd::from_raw(runtime_hwnd as usize),
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
                    hwnd: Hwnd::from_raw(runtime_hwnd as usize),
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

pub fn create_native_window(hwnd: Hwnd, state: &WindowState) {
    let _ = with_backend(|backend| match backend {
        Backend::Local(local) => local.create_window(hwnd, state),
        Backend::Hosted(hosted) => {
            hosted.create_window(hwnd, state);
            Ok(())
        }
    });
}

pub fn destroy_native_window(hwnd: Hwnd) {
    let _ = with_backend(|backend| match backend {
        Backend::Local(local) => local.destroy_window(hwnd),
        Backend::Hosted(hosted) => hosted.destroy_window(hwnd),
    });
}

pub fn set_native_visibility(hwnd: Hwnd, visible: bool) {
    let _ = with_backend(|backend| match backend {
        Backend::Local(local) => local.set_visible(hwnd, visible),
        Backend::Hosted(hosted) => hosted.set_visible(hwnd, visible),
    });
}

pub fn set_native_title(hwnd: Hwnd, title: &str) {
    let _ = with_backend(|backend| match backend {
        Backend::Local(local) => local.set_title(hwnd, title),
        Backend::Hosted(hosted) => hosted.set_title(hwnd, title),
    });
}

pub fn request_native_redraw(hwnd: Hwnd) {
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
