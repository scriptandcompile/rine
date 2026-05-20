// ── X11 cursor provider ──────────────────────────────────────────────
//
// Queries and moves the OS cursor through the X11 protocol via x11rb
// (a pure-Rust X11 client — no C libraries required).
// When running under Wayland the XWayland compat layer handles this
// transparently if available; otherwise we fall back to the cached
// provider.

use tracing::debug;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::ConnectionExt;
use x11rb::rust_connection::RustConnection;

use crate::cursor::provider::CursorProvider;

/// Cursor provider that reads/writes the real OS cursor through X11.
pub struct X11CursorProvider {
    conn: RustConnection,
    root: u32,
}

impl X11CursorProvider {
    /// Try to connect to the X11 server and return a provider if successful.
    pub fn try_new() -> Option<Box<dyn CursorProvider + Send + Sync>> {
        let (conn, screen_num) = RustConnection::connect(None).ok()?;
        let root = conn.setup().roots[screen_num].root;
        debug!("X11 cursor provider initialized (x11rb)");
        Some(Box::new(Self { conn, root }))
    }
}

impl CursorProvider for X11CursorProvider {
    fn get_cursor_pos(&self) -> Option<(i32, i32)> {
        let reply = self.conn.query_pointer(self.root).ok()?.reply().ok()?;
        Some((reply.root_x as i32, reply.root_y as i32))
    }

    fn set_cursor_pos(&self, x: i32, y: i32) -> bool {
        let result = self
            .conn
            .warp_pointer(0u32, self.root, 0, 0, 0, 0, x as i16, y as i16);
        match result {
            Ok(_) => {
                let _ = self.conn.flush();
                true
            }
            Err(_) => false,
        }
    }
}
