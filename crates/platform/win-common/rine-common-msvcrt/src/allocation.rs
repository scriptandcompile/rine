use core::ffi::c_void;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Default)]
pub struct AllocationTracker {
    allocations: Mutex<HashMap<usize, usize>>,
}

impl AllocationTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&self, ptr: *mut c_void, size: usize) {
        if ptr.is_null() {
            return;
        }

        self.allocations.lock().unwrap().insert(ptr as usize, size);
    }

    pub fn forget(&self, ptr: *mut c_void) -> Option<usize> {
        if ptr.is_null() {
            return None;
        }

        self.allocations.lock().unwrap().remove(&(ptr as usize))
    }

    pub fn restore(&self, ptr: *mut c_void, size: usize) {
        if ptr.is_null() {
            return;
        }

        self.allocations.lock().unwrap().insert(ptr as usize, size);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::ffi::c_void;

    #[test]
    fn new_tracker_is_empty() {
        let tracker = AllocationTracker::new();
        let map = tracker.allocations.lock().unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn record_null_does_nothing() {
        let tracker = AllocationTracker::new();
        tracker.record(std::ptr::null_mut(), 42);
        let map = tracker.allocations.lock().unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn record_and_forget() {
        let tracker = AllocationTracker::new();
        let p = 0xDEAD_BEEF as *mut c_void;
        tracker.record(p, 128);
        assert_eq!(tracker.forget(p), Some(128));
        let map = tracker.allocations.lock().unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn forget_null_returns_none() {
        let tracker = AllocationTracker::new();
        assert_eq!(tracker.forget(std::ptr::null_mut()), None);
    }

    #[test]
    fn forget_missing_returns_none() {
        let tracker = AllocationTracker::new();
        let p = 0xCAFE_CAFE as *mut c_void;
        assert_eq!(tracker.forget(p), None);
    }

    #[test]
    fn forget_removes_entry() {
        let tracker = AllocationTracker::new();
        let p = 0x1234_5678 as *mut c_void;
        tracker.record(p, 64);
        assert_eq!(tracker.forget(p), Some(64));
        assert_eq!(tracker.forget(p), None);
    }

    #[test]
    fn restore_updates_size() {
        let tracker = AllocationTracker::new();
        let p = 0xBEEF_CAFE as *mut c_void;
        tracker.record(p, 32);
        tracker.restore(p, 256);
        assert_eq!(tracker.forget(p), Some(256));
    }

    #[test]
    fn restore_null_does_nothing() {
        let tracker = AllocationTracker::new();
        tracker.restore(std::ptr::null_mut(), 99);
        let map = tracker.allocations.lock().unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn record_multiple_entries() {
        let tracker = AllocationTracker::new();
        let p1 = 0x1111 as *mut c_void;
        let p2 = 0x2222 as *mut c_void;
        let p3 = 0x3333 as *mut c_void;
        tracker.record(p1, 10);
        tracker.record(p2, 20);
        tracker.record(p3, 30);
        assert_eq!(tracker.forget(p1), Some(10));
        assert_eq!(tracker.forget(p2), Some(20));
        assert_eq!(tracker.forget(p3), Some(30));
    }

    #[test]
    fn forget_returns_size() {
        let tracker = AllocationTracker::new();
        let p = 0xABCD as *mut c_void;
        tracker.record(p, 4096);
        assert_eq!(tracker.forget(p), Some(4096));
    }
}
