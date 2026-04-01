use std::io::{self, Read};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};

use crate::DevEvent;

/// Receives DevEvents from the rine process over a Unix domain socket.
pub struct DevReceiver {
    stream: UnixStream,
    _listener: UnixListener,
    _socket_path: PathBuf,
}

impl DevReceiver {
    /// Bind a listening socket and wait for the rine process to connect.
    pub fn bind(socket_path: &Path) -> io::Result<Self> {
        // Remove stale socket if it exists.
        let _ = std::fs::remove_file(socket_path);

        let listener = UnixListener::bind(socket_path)?;
        let (stream, _addr) = listener.accept()?;

        Ok(Self {
            stream,
            _listener: listener,
            _socket_path: socket_path.to_path_buf(),
        })
    }

    /// Read a single event (length-prefixed JSON).
    pub fn recv(&mut self) -> io::Result<DevEvent> {
        let mut len_buf = [0u8; 4];
        self.stream.read_exact(&mut len_buf)?;
        let len = u32::from_le_bytes(len_buf) as usize;

        if len > 16 * 1024 * 1024 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("message too large: {len} bytes"),
            ));
        }

        let mut buf = vec![0u8; len];
        self.stream.read_exact(&mut buf)?;

        serde_json::from_slice(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    /// Consume self into an iterator of events, ending when the connection drops.
    pub fn into_stream(mut self) -> impl Iterator<Item = io::Result<DevEvent>> {
        std::iter::from_fn(move || match self.recv() {
            Ok(ev) => Some(Ok(ev)),
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => None,
            Err(e) => Some(Err(e)),
        })
    }
}

impl Drop for DevReceiver {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self._socket_path);
    }
}
