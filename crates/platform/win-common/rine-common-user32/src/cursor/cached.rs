use std::sync::Mutex;

use super::provider::CursorProvider;

/// In-memory cursor position cache.
///
/// Stores the last position written by [`set_cursor_pos`] and returns it
/// from [`get_cursor_pos`].  The position is initially unknown (`None`)
/// until explicitly set.
///
/// This is the default provider in non-windowed environments and in tests.
pub struct CachedCursorProvider {
    position: Mutex<Option<(i32, i32)>>,
}

impl Default for CachedCursorProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl CachedCursorProvider {
    /// Create a new cache with no known position.
    pub fn new() -> Self {
        Self {
            position: Mutex::new(None),
        }
    }
}

impl CursorProvider for CachedCursorProvider {
    fn get_cursor_pos(&self) -> Option<(i32, i32)> {
        *self.position.lock().unwrap()
    }

    fn set_cursor_pos(&self, x: i32, y: i32) -> bool {
        *self.position.lock().unwrap() = Some((x, y));
        true
    }
}
