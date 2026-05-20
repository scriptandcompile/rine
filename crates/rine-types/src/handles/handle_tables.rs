//! Global handle table — maps `HANDLE` → `HandleEntry`.
//!
//! The handle table maps Windows `HANDLE` values to Linux resources
//! (file descriptors, find-data iterators, etc.).  Handles are allocated
//! from a monotonically increasing counter so they never collide with
//! `NULL` (0) or `INVALID_HANDLE_VALUE` (−1).
//!
//! The three standard I/O handles (stdin/stdout/stderr) are pre-registered
//! in the table by [`HandleTable::init`].

use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;

use crate::registry::RegistryKeyState;
use crate::threading::{
    EventWaitable, MutexWaitable, ProcessWaitable, SemaphoreWaitable, ThreadWaitable, Waitable,
};
use crate::windows::{Accel, HWND};

use super::core::HANDLE;

// ---------------------------------------------------------------------------
// Heap state (for HeapCreate handles)
// ---------------------------------------------------------------------------

/// State tracked for a heap created via `HeapCreate`.
#[derive(Debug)]
pub struct HeapState {
    pub allocations: Mutex<HashMap<usize, (usize, usize)>>,
    pub flags: u32,
}

// ---------------------------------------------------------------------------
// Handle table — resource tracking
// ---------------------------------------------------------------------------

/// What a handle points to.
#[derive(Debug)]
pub enum HandleEntry {
    /// A Linux file descriptor (from `open`, `socket`, …).
    File(i32),
    /// A `FindFirstFile` directory search iterator.
    FindData(FindDataState),
    /// A thread created by `CreateThread`.
    Thread(ThreadWaitable),
    /// An event object created by `CreateEvent`.
    Event(EventWaitable),
    /// A child process created by `CreateProcess`.
    Process(ProcessWaitable),
    /// A mutex object created by `CreateMutex`.
    Mutex(MutexWaitable),
    /// A semaphore object created by `CreateSemaphore`.
    Semaphore(SemaphoreWaitable),
    /// A heap object created by `HeapCreate`.
    Heap(HeapState),
    /// A registry key opened by `RegOpenKeyEx` / `RegCreateKeyEx`.
    RegistryKey(RegistryKeyState),
    /// A window created by `CreateWindow`.
    Window(HWND),
    /// An accelerator table loaded by `LoadAccelerators`.
    AcceleratorTable(Vec<Accel>),
}

/// State kept for an active `FindFirstFile`/`FindNextFile` session.
#[derive(Debug)]
pub struct FindDataState {
    pub entries: Vec<FindEntry>,
    pub cursor: usize,
}

/// A single directory entry returned by `FindFirstFile`/`FindNextFile`.
#[derive(Debug, Clone)]
pub struct FindEntry {
    pub file_name: String,
    pub file_size: u64,
    pub attributes: u32,
}

/// Global handle table — maps `Handle` → `HandleEntry`.
pub struct HandleTable {
    inner: Mutex<HandleTableInner>,
}

struct HandleTableInner {
    map: HashMap<isize, HandleEntry>,
    next_id: isize,
}

/// The process-wide handle table.
static HANDLE_TABLE: LazyLock<HandleTable> = LazyLock::new(|| HandleTable {
    inner: Mutex::new(HandleTableInner {
        map: HashMap::new(),
        next_id: 0x1000,
    }),
});

impl Default for HandleTable {
    fn default() -> Self {
        Self::new()
    }
}

impl HandleTable {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HandleTableInner {
                map: HashMap::new(),
                next_id: 0x1000,
            }),
        }
    }

    pub fn init(&self) {
        let mut inner = self.inner.lock().unwrap();
        for fd in 0..3 {
            let id = inner.next_id;
            inner.next_id += 1;
            inner.map.insert(id, HandleEntry::File(fd));
        }
    }

    pub fn insert(&self, entry: HandleEntry) -> HANDLE {
        let mut inner = self.inner.lock().unwrap();
        let id = inner.next_id;
        inner.next_id += 1;
        inner.map.insert(id, entry);
        HANDLE::from_raw(id)
    }

    pub fn remove(&self, h: HANDLE) -> Option<HandleEntry> {
        let mut inner = self.inner.lock().unwrap();
        inner.map.remove(&h.as_raw())
    }

    pub fn get_fd(&self, h: HANDLE) -> Option<i32> {
        let inner = self.inner.lock().unwrap();
        match inner.map.get(&h.as_raw()) {
            Some(HandleEntry::File(fd)) => Some(*fd),
            _ => None,
        }
    }

    pub fn with_find_data<F, R>(&self, h: HANDLE, f: F) -> Option<R>
    where
        F: FnOnce(&mut FindDataState) -> R,
    {
        let mut inner = self.inner.lock().unwrap();
        match inner.map.get_mut(&h.as_raw()) {
            Some(HandleEntry::FindData(state)) => Some(f(state)),
            _ => None,
        }
    }

    pub fn with_heap<F, R>(&self, h: HANDLE, f: F) -> Option<R>
    where
        F: FnOnce(&HeapState) -> R,
    {
        let inner = self.inner.lock().unwrap();
        match inner.map.get(&h.as_raw()) {
            Some(HandleEntry::Heap(state)) => Some(f(state)),
            _ => None,
        }
    }

    pub fn with_registry_key<F, R>(&self, h: HANDLE, f: F) -> Option<R>
    where
        F: FnOnce(&RegistryKeyState) -> R,
    {
        let inner = self.inner.lock().unwrap();
        match inner.map.get(&h.as_raw()) {
            Some(HandleEntry::RegistryKey(state)) => Some(f(state)),
            _ => None,
        }
    }

    pub fn get_hwnd(&self, h: HANDLE) -> Option<HWND> {
        let inner = self.inner.lock().unwrap();
        match inner.map.get(&h.as_raw()) {
            Some(HandleEntry::Window(hwnd)) => Some(*hwnd),
            _ => None,
        }
    }

    pub fn get_waitable(&self, h: HANDLE) -> Option<Waitable> {
        let inner = self.inner.lock().unwrap();
        match inner.map.get(&h.as_raw()) {
            Some(HandleEntry::Thread(t)) => Some(Waitable::Thread(t.clone())),
            Some(HandleEntry::Event(e)) => Some(Waitable::Event(e.clone())),
            Some(HandleEntry::Process(p)) => Some(Waitable::Process(p.clone())),
            Some(HandleEntry::Mutex(m)) => Some(Waitable::Mutex(m.clone())),
            Some(HandleEntry::Semaphore(s)) => Some(Waitable::Semaphore(s.clone())),
            _ => None,
        }
    }

    pub fn with_accelerator_table<F, R>(&self, h: HANDLE, f: F) -> Option<R>
    where
        F: FnOnce(&[Accel]) -> R,
    {
        let inner = self.inner.lock().unwrap();
        match inner.map.get(&h.as_raw()) {
            Some(HandleEntry::AcceleratorTable(entries)) => Some(f(entries)),
            _ => None,
        }
    }

    pub fn get_thread_exit_code(&self, h: HANDLE) -> Option<u32> {
        let inner = self.inner.lock().unwrap();
        match inner.map.get(&h.as_raw()) {
            Some(HandleEntry::Thread(t)) => {
                Some(t.exit_code.load(std::sync::atomic::Ordering::Acquire))
            }
            _ => None,
        }
    }
}

/// Access the process-wide handle table.
pub fn handle_table() -> &'static HandleTable {
    &HANDLE_TABLE
}

// ---------------------------------------------------------------------------
// Standard-handle helpers
// ---------------------------------------------------------------------------

/// Well-known pseudo-handle returned by `GetStdHandle`.
pub const STD_INPUT_HANDLE: u32 = 0xFFFF_FFF6;
pub const STD_OUTPUT_HANDLE: u32 = 0xFFFF_FFF5;
pub const STD_ERROR_HANDLE: u32 = 0xFFFF_FFF4;

/// Map a Windows standard-handle constant to a Linux file descriptor.
pub fn std_handle_to_fd(nstd_handle: u32) -> Option<i32> {
    match nstd_handle {
        STD_INPUT_HANDLE => Some(0),
        STD_OUTPUT_HANDLE => Some(1),
        STD_ERROR_HANDLE => Some(2),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Compatibility shims
// ---------------------------------------------------------------------------

pub const HANDLE_FD_BASE: isize = 0x1000;

/// Encode a Linux file descriptor as a Windows HANDLE.
pub fn fd_to_handle(fd: i32) -> HANDLE {
    HANDLE::from_raw(fd as isize + HANDLE_FD_BASE)
}

/// Decode a HANDLE back to a Linux fd.
pub fn handle_to_fd(h: HANDLE) -> Option<i32> {
    if let Some(fd) = handle_table().get_fd(h) {
        return Some(fd);
    }
    let raw = h.as_raw();
    if raw >= HANDLE_FD_BASE {
        Some((raw - HANDLE_FD_BASE) as i32)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Unit Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::handles::FILE_ATTRIBUTE_ARCHIVE;
    use crate::threading::Waitable;
    use crate::threading::{EventInner, EventWaitable, STILL_ACTIVE, ThreadWaitable};

    use super::*;

    #[test]
    fn std_handle_mapping() {
        assert_eq!(std_handle_to_fd(STD_INPUT_HANDLE), Some(0));
        assert_eq!(std_handle_to_fd(STD_OUTPUT_HANDLE), Some(1));
        assert_eq!(std_handle_to_fd(STD_ERROR_HANDLE), Some(2));
        assert_eq!(std_handle_to_fd(0), None);
    }

    #[test]
    fn fd_handle_roundtrip() {
        for fd in 0..10 {
            let h = fd_to_handle(fd);
            assert_eq!(handle_to_fd(h), Some(fd));
        }
    }

    #[test]
    fn handle_table_insert_and_get_fd() {
        let table = HandleTable::new();
        let h = table.insert(HandleEntry::File(42));
        assert_eq!(table.get_fd(h), Some(42));
    }

    #[test]
    fn handle_table_remove() {
        let table = HandleTable::new();
        let h = table.insert(HandleEntry::File(7));
        assert!(table.get_fd(h).is_some());
        let removed = table.remove(h);
        assert!(matches!(removed, Some(HandleEntry::File(7))));
        assert!(table.get_fd(h).is_none());
    }

    #[test]
    fn handle_table_find_data() {
        let table = HandleTable::new();
        let h = table.insert(HandleEntry::FindData(FindDataState {
            entries: vec![FindEntry {
                file_name: "test.txt".into(),
                file_size: 100,
                attributes: FILE_ATTRIBUTE_ARCHIVE,
            }],
            cursor: 0,
        }));
        assert!(table.get_fd(h).is_none());
        let name = table.with_find_data(h, |state| state.entries[state.cursor].file_name.clone());
        assert_eq!(name, Some("test.txt".into()));
    }

    #[test]
    fn handle_table_thread_entry() {
        use std::sync::atomic::AtomicU32;
        use std::sync::{Arc, Condvar, Mutex};

        let table = HandleTable::new();
        let tw = ThreadWaitable {
            exit_code: Arc::new(AtomicU32::new(STILL_ACTIVE)),
            completed: Arc::new((Mutex::new(false), Condvar::new())),
        };
        let h = table.insert(HandleEntry::Thread(tw));

        assert!(table.get_fd(h).is_none());
        assert_eq!(table.get_thread_exit_code(h), Some(STILL_ACTIVE));
        assert!(matches!(table.get_waitable(h), Some(Waitable::Thread(_))));
    }

    #[test]
    fn handle_table_event_entry() {
        use std::sync::{Arc, Condvar, Mutex};

        let table = HandleTable::new();
        let e = EventWaitable {
            inner: Arc::new(EventInner {
                signaled: Mutex::new(false),
                condvar: Condvar::new(),
                manual_reset: true,
            }),
        };
        let h = table.insert(HandleEntry::Event(e));

        assert!(table.get_fd(h).is_none());
        assert!(table.get_thread_exit_code(h).is_none());
        assert!(matches!(table.get_waitable(h), Some(Waitable::Event(_))));
    }

    #[test]
    fn handle_table_get_waitable_returns_none_for_file() {
        let table = HandleTable::new();
        let h = table.insert(HandleEntry::File(99));
        assert!(table.get_waitable(h).is_none());
    }

    #[test]
    fn handle_table_remove_thread() {
        use std::sync::atomic::AtomicU32;
        use std::sync::{Arc, Condvar, Mutex};

        let table = HandleTable::new();
        let tw = ThreadWaitable {
            exit_code: Arc::new(AtomicU32::new(STILL_ACTIVE)),
            completed: Arc::new((Mutex::new(false), Condvar::new())),
        };
        let h = table.insert(HandleEntry::Thread(tw));
        assert!(matches!(table.remove(h), Some(HandleEntry::Thread(_))));
        assert!(table.get_waitable(h).is_none());
    }
}
