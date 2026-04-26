use tracing::warn;

/// Registers a new window message with the system and returns its message ID.
///
/// # Arguments
/// - `message_kind`: A string that specifies the message to be registered.
///
/// # Returns
/// If the function succeeds, the return value is a message identifier in the range 0xC000 through 0xFFFF.
/// This message identifier can be used as the message parameter when sending or posting messages.
/// If the function fails, the return value is zero. To get extended error information
/// call GetLastError.
///
/// # Notes
/// Currently, this is a stub implementation that always returns 0.
/// We do not set `GetLastError` yet.
pub fn register_window_message(_message_kind: &str) -> u32 {
    warn!("register_window_message is currently a stub implementation that always returns 0");
    0
}
