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
