//! Thread-level state shared by rine DLL implementations.
//!
//! Provides TLS slot management and waitable-object types used by
//! kernel32 threading, synchronization, and the `Wait*` family.

use std::cell::RefCell;
use std::fmt;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Condvar, LazyLock, Mutex};
use std::time::{Duration, Instant};

// ── Windows constants ────────────────────────────────────────────

pub const INFINITE: u32 = 0xFFFF_FFFF;
pub const WAIT_OBJECT_0: u32 = 0x0000_0000;
pub const WAIT_ABANDONED_0: u32 = 0x0000_0080;
pub const WAIT_TIMEOUT: u32 = 0x0000_0102;
pub const WAIT_FAILED: u32 = 0xFFFF_FFFF;
pub const STILL_ACTIVE: u32 = 259;
pub const TLS_OUT_OF_INDEXES: u32 = 0xFFFF_FFFF;

// ── TLS slot management ─────────────────────────────────────────

const TLS_MIN_AVAILABLE: usize = 64;
const TLS_EXPANSION: usize = 1024;
const TLS_MAX_SLOTS: usize = TLS_MIN_AVAILABLE + TLS_EXPANSION;

/// Global bitmap tracking which TLS indices are allocated.
static TLS_BITMAP: LazyLock<Mutex<[bool; TLS_MAX_SLOTS]>> =
    LazyLock::new(|| Mutex::new([false; TLS_MAX_SLOTS]));

thread_local! {
    /// Per-thread TLS values.  Index corresponds to the global slot index.
    static TLS_VALUES: RefCell<Vec<usize>> = RefCell::new(vec![0; TLS_MAX_SLOTS]);
}

/// Allocate a TLS index.  Returns `None` if all slots are exhausted.
pub fn tls_alloc() -> Option<u32> {
    let mut bitmap = TLS_BITMAP.lock().unwrap();
    for (i, slot) in bitmap.iter_mut().enumerate() {
        if !*slot {
            *slot = true;
            return Some(i as u32);
        }
    }
    None
}

/// Free a previously allocated TLS index.
pub fn tls_free(index: u32) -> bool {
    let idx = index as usize;
    if idx >= TLS_MAX_SLOTS {
        return false;
    }
    let mut bitmap = TLS_BITMAP.lock().unwrap();
    if !bitmap[idx] {
        return false;
    }
    bitmap[idx] = false;
    // Clear value in the current thread (matches Windows behaviour).
    TLS_VALUES.with(|v| {
        let mut v = v.borrow_mut();
        if idx < v.len() {
            v[idx] = 0;
        }
    });
    true
}

/// Get the value for a TLS index in the current thread.
pub fn tls_get_value(index: u32) -> usize {
    let idx = index as usize;
    if idx >= TLS_MAX_SLOTS {
        return 0;
    }
    TLS_VALUES.with(|v| {
        let v = v.borrow();
        if idx < v.len() { v[idx] } else { 0 }
    })
}

/// Set the value for a TLS index in the current thread.
pub fn tls_set_value(index: u32, value: usize) -> bool {
    let idx = index as usize;
    if idx >= TLS_MAX_SLOTS {
        return false;
    }
    TLS_VALUES.with(|v| {
        let mut v = v.borrow_mut();
        if idx < v.len() {
            v[idx] = value;
        }
    });
    true
}

// ── Waitable object types ───────────────────────────────────────

/// Shared state backing a thread handle.
#[derive(Clone)]
pub struct ThreadWaitable {
    /// Exit code.  [`STILL_ACTIVE`] while the thread is running.
    pub exit_code: Arc<AtomicU32>,
    /// (`done` flag, condvar) — signalled when the thread exits.
    pub completed: Arc<(Mutex<bool>, Condvar)>,
}

impl fmt::Debug for ThreadWaitable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ThreadWaitable")
            .field("exit_code", &self.exit_code.load(Ordering::Relaxed))
            .finish()
    }
}

/// Shared state backing an event handle.
#[derive(Clone)]
pub struct EventWaitable {
    pub inner: Arc<EventInner>,
}

impl fmt::Debug for EventWaitable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventWaitable")
            .field("manual_reset", &self.inner.manual_reset)
            .finish()
    }
}

/// Interior of an event object.
pub struct EventInner {
    pub signaled: Mutex<bool>,
    pub condvar: Condvar,
    pub manual_reset: bool,
}

/// A waitable object extracted from the handle table (Arc-cloned so the
/// table lock is not held during the wait).
#[derive(Clone)]
pub enum Waitable {
    Thread(ThreadWaitable),
    Event(EventWaitable),
}

// ── Wait helpers ─────────────────────────────────────────────────

/// Wait on a single waitable object with the given timeout (milliseconds).
/// Returns one of `WAIT_OBJECT_0`, `WAIT_TIMEOUT`, or `WAIT_FAILED`.
pub fn wait_on(waitable: &Waitable, timeout_ms: u32) -> u32 {
    match waitable {
        Waitable::Thread(t) => wait_thread(t, timeout_ms),
        Waitable::Event(e) => wait_event(e, timeout_ms),
    }
}

fn wait_thread(t: &ThreadWaitable, timeout_ms: u32) -> u32 {
    let (lock, cvar) = &*t.completed;
    let mut done = lock.lock().unwrap();
    if *done {
        return WAIT_OBJECT_0;
    }
    if timeout_ms == 0 {
        return WAIT_TIMEOUT;
    }
    if timeout_ms == INFINITE {
        while !*done {
            done = cvar.wait(done).unwrap();
        }
        return WAIT_OBJECT_0;
    }
    let deadline = Instant::now() + Duration::from_millis(timeout_ms as u64);
    while !*done {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return WAIT_TIMEOUT;
        }
        let result = cvar.wait_timeout(done, remaining).unwrap();
        done = result.0;
    }
    WAIT_OBJECT_0
}

fn wait_event(e: &EventWaitable, timeout_ms: u32) -> u32 {
    let mut signaled = e.inner.signaled.lock().unwrap();
    if *signaled {
        if !e.inner.manual_reset {
            *signaled = false; // auto-reset
        }
        return WAIT_OBJECT_0;
    }
    if timeout_ms == 0 {
        return WAIT_TIMEOUT;
    }
    if timeout_ms == INFINITE {
        while !*signaled {
            signaled = e.inner.condvar.wait(signaled).unwrap();
        }
        if !e.inner.manual_reset {
            *signaled = false;
        }
        return WAIT_OBJECT_0;
    }
    let deadline = Instant::now() + Duration::from_millis(timeout_ms as u64);
    while !*signaled {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return WAIT_TIMEOUT;
        }
        let result = e.inner.condvar.wait_timeout(signaled, remaining).unwrap();
        signaled = result.0;
    }
    if !e.inner.manual_reset {
        *signaled = false;
    }
    WAIT_OBJECT_0
}
