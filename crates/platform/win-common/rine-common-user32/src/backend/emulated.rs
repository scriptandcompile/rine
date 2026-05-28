use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use rine_channel::{WindowHostCommand, WindowHostEvent, WindowHostSender};
use rine_types::windows::*;

use super::{WindowBackend, user32_debug_enabled};

pub(super) struct EmulatedBackend {
    sender: Arc<Mutex<WindowHostSender>>,
    events: mpsc::Receiver<WindowHostEvent>,
}

impl EmulatedBackend {
    pub(super) fn connect(socket_path: &str) -> Result<Self, String> {
        let sender = WindowHostSender::connect(std::path::Path::new(socket_path))
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

    fn send(&self, command: WindowHostCommand) {
        let Ok(mut sender) = self.sender.lock() else {
            return;
        };
        if let Err(error) = sender.send_command(&command) {
            eprintln!(
                "warning: window host command failed; native backend falling back to emulated behavior: {error}"
            );
        }
    }

    pub(super) fn create_window(&self, hwnd: HWND, state: &WindowState) {
        self.send(WindowHostCommand::CreateWindow {
            runtime_hwnd: hwnd.as_raw() as u64,
            title: state.title.clone(),
            rect: state.rect,
            visible: state.visible,
            style: state.style,
            ex_style: state.ex_style,
        });
    }

    pub(super) fn destroy_window(&self, hwnd: HWND) {
        self.send(WindowHostCommand::DestroyWindow {
            runtime_hwnd: hwnd.as_raw() as u64,
        });
    }

    pub(super) fn set_visible(&self, hwnd: HWND, visible: bool) {
        self.send(WindowHostCommand::SetVisible {
            runtime_hwnd: hwnd.as_raw() as u64,
            visible,
        });
    }

    pub(super) fn set_title(&self, hwnd: HWND, title: &str) {
        self.send(WindowHostCommand::SetTitle {
            runtime_hwnd: hwnd.as_raw() as u64,
            title: title.to_owned(),
        });
    }

    pub(super) fn request_redraw(&self, hwnd: HWND) {
        self.send(WindowHostCommand::RequestRedraw {
            runtime_hwnd: hwnd.as_raw() as u64,
        });
    }

    pub(super) fn set_window_rect(&self, hwnd: HWND, x: i32, y: i32, width: i32, height: i32) {
        self.send(WindowHostCommand::SetRect {
            runtime_hwnd: hwnd.as_raw() as u64,
            rect: Rect {
                left: x,
                top: y,
                right: x + width,
                bottom: y + height,
            },
        });
    }

    pub(super) fn pump_messages(&mut self) -> Vec<Msg> {
        let mut out = Vec::new();
        while let Ok(event) = self.events.try_recv() {
            match event {
                WindowHostEvent::Created { .. } => {}
                WindowHostEvent::CloseRequested { runtime_hwnd } => out.push(Msg {
                    hwnd: HWND::from_raw(runtime_hwnd as usize),
                    message: window_message::WM_CLOSE,
                    w_param: 0,
                    l_param: 0,
                    time: 0,
                    pt: Point::default(),
                }),
                WindowHostEvent::Destroyed { runtime_hwnd } => out.push(Msg {
                    hwnd: HWND::from_raw(runtime_hwnd as usize),
                    message: window_message::WM_DESTROY,
                    w_param: 0,
                    l_param: 0,
                    time: 0,
                    pt: Point::default(),
                }),
                WindowHostEvent::Resized {
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
                WindowHostEvent::Moved { runtime_hwnd, x, y } => {
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
                WindowHostEvent::Focused {
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
                WindowHostEvent::RedrawRequested { runtime_hwnd } => out.push(Msg {
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

impl WindowBackend for EmulatedBackend {
    fn create_window(&mut self, hwnd: HWND, state: &WindowState) {
        EmulatedBackend::create_window(self, hwnd, state);
    }

    fn destroy_window(&mut self, hwnd: HWND) {
        EmulatedBackend::destroy_window(self, hwnd);
    }

    fn set_visible(&mut self, hwnd: HWND, visible: bool) {
        EmulatedBackend::set_visible(self, hwnd, visible);
    }

    fn set_title(&mut self, hwnd: HWND, title: &str) {
        EmulatedBackend::set_title(self, hwnd, title);
    }

    fn request_redraw(&mut self, hwnd: HWND) {
        EmulatedBackend::request_redraw(self, hwnd);
    }

    fn set_window_rect(&mut self, hwnd: HWND, x: i32, y: i32, width: i32, height: i32) {
        EmulatedBackend::set_window_rect(self, hwnd, x, y, width, height);
    }

    fn pump_messages(&mut self) -> Vec<Msg> {
        EmulatedBackend::pump_messages(self)
    }
}
