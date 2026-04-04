use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rine_channel::{HostWindowCommand, HostWindowEvent, HostWindowReceiver, HostWindowSender};
use rine_types::windows::Rect;
use tracing::warn;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
use winit::platform::run_return::EventLoopExtRunReturn;
#[cfg(target_os = "linux")]
use winit::platform::wayland::EventLoopBuilderExtWayland;
#[cfg(target_os = "linux")]
use winit::platform::x11::EventLoopBuilderExtX11;
use winit::window::{Window, WindowBuilder, WindowId};

pub const WINDOW_HOST_SOCKET_ENV: &str = "RINE_WINDOW_HOST_SOCKET";

pub struct WindowHostSession {
    socket_path: PathBuf,
    transport_thread: JoinHandle<io::Result<()>>,
    host_thread: JoinHandle<()>,
}

impl WindowHostSession {
    pub fn start() -> io::Result<Self> {
        let socket_path = unique_socket_path();
        let (command_tx, command_rx) = mpsc::channel();
        let (event_tx, event_rx) = mpsc::channel();

        let host_thread = thread::spawn(move || run_window_host(command_rx, event_tx));

        let transport_socket_path = socket_path.clone();
        let transport_thread = thread::spawn(move || {
            let mut receiver = HostWindowReceiver::bind(&transport_socket_path)?;
            let mut event_sender = receiver.try_clone_sender()?;
            let writer = thread::spawn(move || {
                while let Ok(event) = event_rx.recv() {
                    if event_sender.send_event(&event).is_err() {
                        break;
                    }
                }
            });

            loop {
                match receiver.recv_command() {
                    Ok(command) => {
                        let should_stop = matches!(command, HostWindowCommand::ShutdownWindowHost);
                        if command_tx.send(command).is_err() {
                            break;
                        }
                        if should_stop {
                            break;
                        }
                    }
                    Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => break,
                    Err(error) => {
                        let _ = writer.join();
                        return Err(error);
                    }
                }
            }

            drop(command_tx);
            let _ = writer.join();
            Ok(())
        });

        Ok(Self {
            socket_path,
            transport_thread,
            host_thread,
        })
    }

    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    pub fn wait(self) {
        self.request_shutdown();

        match self.transport_thread.join() {
            Ok(Ok(())) => {}
            Ok(Err(error)) => warn!("window host transport thread failed: {error}"),
            Err(_) => warn!("window host transport thread panicked"),
        }

        if self.host_thread.join().is_err() {
            warn!("window host thread panicked");
        }
    }

    fn request_shutdown(&self) {
        if let Ok(mut sender) = HostWindowSender::connect(&self.socket_path) {
            let _ = sender.send_command(&HostWindowCommand::ShutdownWindowHost);
        }
    }
}

fn unique_socket_path() -> PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!("rine-window-host-{ts}.sock"))
}

struct WindowHost {
    event_loop: EventLoop<()>,
    windows: HashMap<u64, Window>,
    reverse: HashMap<WindowId, u64>,
}

impl WindowHost {
    fn new() -> Self {
        let mut event_loop_builder = EventLoopBuilder::new();
        #[cfg(target_os = "linux")]
        {
            EventLoopBuilderExtWayland::with_any_thread(&mut event_loop_builder, true);
            EventLoopBuilderExtX11::with_any_thread(&mut event_loop_builder, true);
        }

        Self {
            event_loop: event_loop_builder.build(),
            windows: HashMap::new(),
            reverse: HashMap::new(),
        }
    }

    fn apply_command(
        &mut self,
        command: HostWindowCommand,
        event_tx: &Sender<HostWindowEvent>,
    ) -> bool {
        match command {
            HostWindowCommand::CreateWindow {
                runtime_hwnd,
                title,
                rect,
                visible,
                ..
            } => {
                let result = self.create_window(runtime_hwnd, &title, rect, visible);
                let _ = event_tx.send(HostWindowEvent::Created {
                    runtime_hwnd,
                    success: result.is_ok(),
                    error: result.err(),
                });
            }
            HostWindowCommand::DestroyWindow { runtime_hwnd } => {
                self.destroy_window(runtime_hwnd);
                let _ = event_tx.send(HostWindowEvent::Destroyed { runtime_hwnd });
            }
            HostWindowCommand::SetTitle {
                runtime_hwnd,
                title,
            } => {
                if let Some(window) = self.windows.get(&runtime_hwnd) {
                    window.set_title(&title);
                }
            }
            HostWindowCommand::SetVisible {
                runtime_hwnd,
                visible,
            } => {
                if let Some(window) = self.windows.get(&runtime_hwnd) {
                    window.set_visible(visible);
                }
            }
            HostWindowCommand::SetRect { runtime_hwnd, rect } => {
                if let Some(window) = self.windows.get(&runtime_hwnd) {
                    window.set_outer_position(LogicalPosition::new(
                        rect.left as f64,
                        rect.top as f64,
                    ));
                    let width = rect.right.saturating_sub(rect.left).max(1) as f64;
                    let height = rect.bottom.saturating_sub(rect.top).max(1) as f64;
                    window.set_inner_size(LogicalSize::new(
                        width,
                        height,
                    ));
                }
            }
            HostWindowCommand::RequestRedraw { runtime_hwnd } => {
                if let Some(window) = self.windows.get(&runtime_hwnd) {
                    window.request_redraw();
                }
            }
            HostWindowCommand::ShutdownWindowHost => return false,
        }

        true
    }

    fn create_window(
        &mut self,
        runtime_hwnd: u64,
        title: &str,
        rect: Rect,
        visible: bool,
    ) -> Result<(), String> {
        let width = rect.right.saturating_sub(rect.left).max(1) as f64;
        let height = rect.bottom.saturating_sub(rect.top).max(1) as f64;
        let window = WindowBuilder::new()
            .with_title(title)
            .with_visible(visible)
            .with_inner_size(LogicalSize::new(width, height))
            .with_position(LogicalPosition::new(rect.left as f64, rect.top as f64))
            .build(&self.event_loop)
            .map_err(|error| format!("failed to create host window: {error}"))?;

        let window_id = window.id();
        self.reverse.insert(window_id, runtime_hwnd);
        self.windows.insert(runtime_hwnd, window);
        Ok(())
    }

    fn destroy_window(&mut self, runtime_hwnd: u64) {
        if let Some(window) = self.windows.remove(&runtime_hwnd) {
            self.reverse.remove(&window.id());
        }
    }

    fn pump_events(&mut self, event_tx: &Sender<HostWindowEvent>) {
        let reverse = &self.reverse;
        self.event_loop.run_return(|event, _target, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent { window_id, event } => {
                    let Some(runtime_hwnd) = reverse.get(&window_id).copied() else {
                        return;
                    };

                    let host_event = match event {
                        WindowEvent::CloseRequested => {
                            Some(HostWindowEvent::CloseRequested { runtime_hwnd })
                        }
                        WindowEvent::Destroyed => Some(HostWindowEvent::Destroyed { runtime_hwnd }),
                        WindowEvent::Resized(size) => Some(HostWindowEvent::Resized {
                            runtime_hwnd,
                            width: size.width,
                            height: size.height,
                        }),
                        WindowEvent::Moved(position) => Some(HostWindowEvent::Moved {
                            runtime_hwnd,
                            x: position.x,
                            y: position.y,
                        }),
                        WindowEvent::Focused(focused) => Some(HostWindowEvent::Focused {
                            runtime_hwnd,
                            focused,
                        }),
                        _ => None,
                    };

                    if let Some(host_event) = host_event {
                        let _ = event_tx.send(host_event);
                    }
                }
                Event::RedrawRequested(window_id) => {
                    if let Some(runtime_hwnd) = reverse.get(&window_id).copied() {
                        let _ = event_tx.send(HostWindowEvent::RedrawRequested { runtime_hwnd });
                    }
                }
                Event::MainEventsCleared | Event::RedrawEventsCleared => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            }
        });
    }
}

fn run_window_host(command_rx: Receiver<HostWindowCommand>, event_tx: Sender<HostWindowEvent>) {
    let mut host = WindowHost::new();

    loop {
        let mut processed_command = false;
        loop {
            match command_rx.try_recv() {
                Ok(command) => {
                    processed_command = true;
                    if !host.apply_command(command, &event_tx) {
                        return;
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return,
            }
        }

        host.pump_events(&event_tx);

        if !processed_command {
            thread::sleep(Duration::from_millis(10));
        }
    }
}
