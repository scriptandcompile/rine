/// Pluggable back-end for cursor position queries and moves.
pub trait CursorProvider: Send + Sync {
    /// Returns the current cursor position in screen coordinates,
    /// or `None` if the position could not be determined.
    fn get_cursor_pos(&self) -> Option<(i32, i32)>;

    /// Moves the cursor to `(x, y)` in screen coordinates.
    /// Returns `true` on success, `false` on failure.
    fn set_cursor_pos(&self, x: i32, y: i32) -> bool;
}
