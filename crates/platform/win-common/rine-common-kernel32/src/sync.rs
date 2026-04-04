use std::sync::{Arc, Condvar, Mutex};

use rine_types::errors::WinBool;
use rine_types::handles::{Handle, HandleEntry, handle_table};
use rine_types::threading::{EventInner, EventWaitable};

pub fn create_event(manual_reset: WinBool, initial_state: WinBool) -> Handle {
    let waitable = EventWaitable {
        inner: Arc::new(EventInner {
            signaled: Mutex::new(initial_state.is_true()),
            condvar: Condvar::new(),
            manual_reset: manual_reset.is_true(),
        }),
    };
    handle_table().insert(HandleEntry::Event(waitable))
}
