use std::sync::{Arc, Condvar, Mutex};

use rine_types::errors::WinBool;
use rine_types::handles::{Handle, HandleEntry, handle_table};
use rine_types::threading::{EventInner, EventWaitable, MutexInner, MutexState, MutexWaitable};

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

pub fn create_mutex(initial_owner: WinBool, name: Option<String>) -> (Handle, String) {
    let (owner, count) = if initial_owner.is_true() {
        (Some(std::thread::current().id()), 1)
    } else {
        (None, 0)
    };

    let waitable = MutexWaitable {
        inner: Arc::new(MutexInner {
            state: Mutex::new(MutexState { owner, count }),
            condvar: Condvar::new(),
        }),
    };
    let h = handle_table().insert(HandleEntry::Mutex(waitable));

    let detail = match (name.as_deref(), initial_owner.is_true()) {
        (Some(n), true) => format!("{} (initially-owned)", n),
        (Some(n), false) => n.to_owned(),
        (None, true) => "(unnamed, initially-owned)".to_owned(),
        (None, false) => "(unnamed)".to_owned(),
    };

    (h, detail)
}
