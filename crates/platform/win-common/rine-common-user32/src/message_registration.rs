use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

const REGISTERED_MESSAGE_MIN: u32 = 0xC000;
const REGISTERED_MESSAGE_MAX: u32 = 0xFFFF;

#[derive(Default)]
struct RegisteredMessageTable {
    by_name: HashMap<String, u32>,
    next_id: u32,
}

impl RegisteredMessageTable {
    fn register(&mut self, message_kind: &str) -> u32 {
        // RegisterWindowMessage names are matched case-insensitively, so normalize
        // to a lowercase key before lookup and insertion.
        let normalized_name = message_kind.to_lowercase();

        if let Some(existing) = self.by_name.get(&normalized_name) {
            return *existing;
        }

        if self.next_id > REGISTERED_MESSAGE_MAX {
            return 0;
        }

        let message_id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        self.by_name.insert(normalized_name, message_id);
        message_id
    }
}

static REGISTERED_MESSAGES: LazyLock<Mutex<RegisteredMessageTable>> = LazyLock::new(|| {
    Mutex::new(RegisteredMessageTable {
        by_name: HashMap::new(),
        next_id: REGISTERED_MESSAGE_MIN,
    })
});

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
/// Missing implementation features:
/// - We do not set `GetLastError` yet on failures.
/// - Process/session scoping nuances are simplified to a process-global table.
/// - Case-insensitive matching uses a normalized lowercase key.
pub fn register_window_message(message_kind: &str) -> u32 {
    if message_kind.is_empty() {
        return 0;
    }

    let mut table = REGISTERED_MESSAGES.lock().unwrap();
    table.register(message_kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_window_message_reuses_existing_id() {
        let id1 = register_window_message("rine.test.message.reused");
        let id2 = register_window_message("rine.test.message.reused");
        assert_eq!(id1, id2);
        assert!((REGISTERED_MESSAGE_MIN..=REGISTERED_MESSAGE_MAX).contains(&id1));
    }

    #[test]
    fn register_window_message_is_case_insensitive() {
        let id1 = register_window_message("Rine.Test.Message.Case");
        let id2 = register_window_message("rine.test.message.case");
        assert_eq!(id1, id2);
    }

    #[test]
    fn register_window_message_rejects_empty_name() {
        assert_eq!(register_window_message(""), 0);
    }
}
