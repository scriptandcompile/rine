use std::io::{self, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

use crate::DevEvent;

/// Sends DevEvents to the rine-dev dashboard over a Unix domain socket.
pub struct DevSender {
    stream: UnixStream,
}

impl DevSender {
    /// Connect to a listening rine-dev socket.
    ///
    /// Retries with exponential backoff for up to ~2s to give
    /// rine-dev time to bind the socket after being spawned.
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

    /// Send a single event (length-prefixed JSON).
    pub fn send(&mut self, event: &DevEvent) -> io::Result<()> {
        let json =
            serde_json::to_vec(event).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let len = (json.len() as u32).to_le_bytes();
        self.stream.write_all(&len)?;
        self.stream.write_all(&json)?;
        self.stream.flush()
    }

    /// Shut down the write half of the socket so the receiver sees a
    /// clean EOF after reading all buffered data.
    pub fn shutdown(&mut self) {
        let _ = self.stream.shutdown(std::net::Shutdown::Write);
    }
}
