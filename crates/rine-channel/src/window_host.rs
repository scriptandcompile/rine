use std::io::{self, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

use rine_types::windows::Rect;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum HostWindowCommand {
    CreateWindow {
        runtime_hwnd: u64,
        title: String,
        rect: Rect,
        visible: bool,
        style: u32,
        ex_style: u32,
    },
    DestroyWindow {
        runtime_hwnd: u64,
    },
    SetTitle {
        runtime_hwnd: u64,
        title: String,
    },
    SetVisible {
        runtime_hwnd: u64,
        visible: bool,
    },
    SetRect {
        runtime_hwnd: u64,
        rect: Rect,
    },
    RequestRedraw {
        runtime_hwnd: u64,
    },
    ShutdownWindowHost,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum HostWindowEvent {
    Created {
        runtime_hwnd: u64,
        success: bool,
        error: Option<String>,
    },
    CloseRequested {
        runtime_hwnd: u64,
    },
    Destroyed {
        runtime_hwnd: u64,
    },
    Resized {
        runtime_hwnd: u64,
        width: u32,
        height: u32,
    },
    Moved {
        runtime_hwnd: u64,
        x: i32,
        y: i32,
    },
    Focused {
        runtime_hwnd: u64,
        focused: bool,
    },
    RedrawRequested {
        runtime_hwnd: u64,
    },
}

pub struct HostWindowSender {
    stream: UnixStream,
}

impl HostWindowSender {
    pub fn connect(socket_path: &Path) -> io::Result<Self> {
        let mut delay = Duration::from_millis(50);
        let max_total = Duration::from_secs(2);
        let start = std::time::Instant::now();

        loop {
            match UnixStream::connect(socket_path) {
                Ok(stream) => return Ok(Self { stream }),
                Err(e) if start.elapsed() < max_total => {
                    std::thread::sleep(delay);
                    delay = (delay * 2).min(Duration::from_millis(500));
                    if start.elapsed() >= max_total {
                        return Err(e);
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }

    pub fn send_command(&mut self, command: &HostWindowCommand) -> io::Result<()> {
        write_message(&mut self.stream, command)
    }

    pub fn send_event(&mut self, event: &HostWindowEvent) -> io::Result<()> {
        write_message(&mut self.stream, event)
    }

    pub fn recv_event(&mut self) -> io::Result<HostWindowEvent> {
        read_message(&mut self.stream)
    }

    pub fn try_clone(&self) -> io::Result<Self> {
        Ok(Self {
            stream: self.stream.try_clone()?,
        })
    }

    pub fn shutdown(&mut self) {
        let _ = self.stream.shutdown(std::net::Shutdown::Write);
    }
}

pub struct HostWindowReceiver {
    stream: UnixStream,
    _listener: UnixListener,
    _socket_path: PathBuf,
}

impl HostWindowReceiver {
    pub fn bind(socket_path: &Path) -> io::Result<Self> {
        let _ = std::fs::remove_file(socket_path);
        let listener = UnixListener::bind(socket_path)?;
        let (stream, _addr) = listener.accept()?;
        Ok(Self {
            stream,
            _listener: listener,
            _socket_path: socket_path.to_path_buf(),
        })
    }

    pub fn recv_command(&mut self) -> io::Result<HostWindowCommand> {
        read_message(&mut self.stream)
    }

    pub fn send_event(&mut self, event: &HostWindowEvent) -> io::Result<()> {
        write_message(&mut self.stream, event)
    }

    pub fn try_clone_sender(&self) -> io::Result<HostWindowSender> {
        Ok(HostWindowSender {
            stream: self.stream.try_clone()?,
        })
    }
}

impl Drop for HostWindowReceiver {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self._socket_path);
    }
}

fn write_message<T: Serialize>(stream: &mut UnixStream, value: &T) -> io::Result<()> {
    let json =
        serde_json::to_vec(value).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let len = (json.len() as u32).to_le_bytes();
    stream.write_all(&len)?;
    stream.write_all(&json)?;
    stream.flush()
}

fn read_message<T: for<'de> Deserialize<'de>>(stream: &mut UnixStream) -> io::Result<T> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;

    let len = u32::from_le_bytes(len_buf) as usize;
    if len > 16 * 1024 * 1024 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("message too large: {len} bytes"),
        ));
    }

    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf)?;

    serde_json::from_slice(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_socket_path() -> PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!("rine-window-host-{ts}.sock"))
    }

    #[test]
    fn round_trips_host_window_messages() {
        let socket_path = unique_socket_path();
        let server_path = socket_path.clone();

        let server = std::thread::spawn(move || {
            let mut receiver = HostWindowReceiver::bind(&server_path).expect("bind should work");
            let command = receiver.recv_command().expect("recv_command should work");
            assert_eq!(
                command,
                HostWindowCommand::SetVisible {
                    runtime_hwnd: 0x1234,
                    visible: true,
                }
            );
            receiver
                .send_event(&HostWindowEvent::Focused {
                    runtime_hwnd: 0x1234,
                    focused: true,
                })
                .expect("send_event should work");
        });

        let mut sender = HostWindowSender::connect(&socket_path).expect("connect should work");
        sender
            .send_command(&HostWindowCommand::SetVisible {
                runtime_hwnd: 0x1234,
                visible: true,
            })
            .expect("send_command should work");

        let event = sender.recv_event().expect("recv_event should work");

        assert_eq!(
            event,
            HostWindowEvent::Focused {
                runtime_hwnd: 0x1234,
                focused: true,
            }
        );
        server.join().expect("server thread should succeed");
    }
}
