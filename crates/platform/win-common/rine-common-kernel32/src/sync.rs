use std::sync::{Arc, Condvar, Mutex};

use rine_types::errors::WinBool;
use rine_types::handles::{Handle, HandleEntry, handle_table};
use rine_types::threading::{EventInner, EventWaitable, MutexInner, MutexState, MutexWaitable};

/// Creates a synchronization event.
///
/// # Arguments
/// * `manual_reset` - If true, the event remains signaled until manually reset; if false, it resets automatically after a single wait.
/// * `initial_state` - If true, the event starts in a signaled state; if false, it starts non-signaled.
///
/// # Returns
/// A handle to the newly created event.
///
/// # Examples
/// ```
/// use rine_common_kernel32::sync::create_event;
/// use rine_types::errors::WinBool;
///
/// let manual_reset_event = create_event(WinBool::TRUE, WinBool::FALSE);
/// let auto_reset_event = create_event(WinBool::FALSE, WinBool::TRUE);
/// ```
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

/// Creates a named or unnamed mutex.
///
/// # Arguments
/// * `initial_owner` - If true, the calling thread becomes the initial owner of the mutex.
/// * `name` - Optional name for the mutex to allow cross-process synchronization.
///
/// # Returns
/// A tuple containing the mutex handle and a descriptive string of the mutex state.
///
/// # Examples
/// ```
/// use rine_common_kernel32::sync::create_mutex;
/// use rine_types::errors::WinBool;
///
/// let (unnamed_mutex, desc) = create_mutex(WinBool::FALSE, None);
/// assert_eq!(desc, "(unnamed)");
///
/// let (named_mutex, desc) = create_mutex(WinBool::TRUE, Some("MyMutex".to_string()));
/// assert_eq!(desc, "MyMutex (initially-owned)");
/// ```
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

#[cfg(test)]
mod tests {
    use super::*;
    use rine_types::errors::WinBool;

    #[test]
    fn test_create_event_auto_reset() {
        let h = create_event(WinBool::FALSE, WinBool::FALSE);
        assert!(h.is_valid());
    }

    #[test]
    fn test_create_event_manual_reset() {
        let h = create_event(WinBool::TRUE, WinBool::FALSE);
        assert!(h.is_valid());
    }

    #[test]
    fn test_create_event_initial_signaled() {
        let h = create_event(WinBool::FALSE, WinBool::TRUE);
        assert!(h.is_valid());
    }

    #[test]
    fn test_create_event_initial_not_signaled() {
        let h = create_event(WinBool::FALSE, WinBool::FALSE);
        assert!(h.is_valid());
    }

    #[test]
    fn test_create_event_different_parameters() {
        let h1 = create_event(WinBool::FALSE, WinBool::FALSE);
        let h2 = create_event(WinBool::FALSE, WinBool::FALSE);
        assert!(h1.is_valid());
        assert!(h2.is_valid());
    }

    #[test]
    fn test_create_event_manual_vs_auto() {
        let h1 = create_event(WinBool::FALSE, WinBool::FALSE);
        let h2 = create_event(WinBool::TRUE, WinBool::FALSE);
        assert!(h1.is_valid());
        assert!(h2.is_valid());
    }

    #[test]
    fn test_create_mutex_unnamed_auto() {
        let (h, desc) = create_mutex(WinBool::FALSE, None);
        assert!(h.is_valid());
        assert!(desc.contains("(unnamed)"));
    }

    #[test]
    fn test_create_mutex_unnamed_initial_owned() {
        let (h, desc) = create_mutex(WinBool::TRUE, None);
        assert!(h.is_valid());
        assert!(desc.contains("(unnamed") && desc.contains("initially-owned"));
    }

    #[test]
    fn test_create_mutex_named() {
        let (h, desc) = create_mutex(WinBool::FALSE, Some("TestMutex".to_string()));
        assert!(h.is_valid());
        assert_eq!(desc, "TestMutex");
    }

    #[test]
    fn test_create_mutex_named_initial_owned() {
        let (h, desc) = create_mutex(WinBool::TRUE, Some("OwnedMutex".to_string()));
        assert!(h.is_valid());
        assert_eq!(desc, "OwnedMutex (initially-owned)");
    }

    #[test]
    fn test_create_mutex_different_parameters() {
        let (h1, desc1) = create_mutex(WinBool::FALSE, None);
        let (h2, desc2) = create_mutex(WinBool::FALSE, Some("Test".to_string()));
        assert!(h1.is_valid());
        assert!(h2.is_valid());
        assert_eq!(desc1, "(unnamed)");
        assert_eq!(desc2, "Test");
    }

    #[test]
    fn test_create_mutex_initial_state() {
        let (h, desc) = create_mutex(WinBool::TRUE, Some("Test".to_string()));
        assert!(h.is_valid());
        assert!(desc.contains("initially-owned"));

        let (h2, desc2) = create_mutex(WinBool::FALSE, Some("Test2".to_string()));
        assert!(h2.is_valid());
        assert!(!desc2.contains("initially-owned"));
    }

    #[test]
    fn test_create_mutex_multiple_instances() {
        let mut handles = Vec::new();
        let mut descs = Vec::new();

        for i in 0..5 {
            let (h, desc) = create_mutex(WinBool::FALSE, Some(format!("Mutex{}", i)));
            handles.push(h);
            descs.push(desc);
        }

        for h in &handles {
            assert!(h.is_valid());
        }

        assert_eq!(descs[0], "Mutex0");
        assert_eq!(descs[1], "Mutex1");
        assert_eq!(descs[2], "Mutex2");
        assert_eq!(descs[3], "Mutex3");
        assert_eq!(descs[4], "Mutex4");
    }
}
