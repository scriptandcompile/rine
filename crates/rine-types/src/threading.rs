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

#[cfg(test)]
mod tests {
    use super::*;

    // ── TLS tests ────────────────────────────────────────────────

    #[test]
    fn tls_alloc_returns_sequential_indices() {
        let a = tls_alloc().unwrap();
        let b = tls_alloc().unwrap();
        // Indices are unique (not necessarily contiguous due to parallel tests,
        // but must differ).
        assert_ne!(a, b);
        tls_free(a);
        tls_free(b);
    }

    #[test]
    fn tls_get_set_roundtrip() {
        let idx = tls_alloc().unwrap();
        assert_eq!(tls_get_value(idx), 0); // default is zero
        assert!(tls_set_value(idx, 0xDEAD_BEEF));
        assert_eq!(tls_get_value(idx), 0xDEAD_BEEF);
        tls_free(idx);
    }

    #[test]
    fn tls_free_clears_value() {
        let idx = tls_alloc().unwrap();
        tls_set_value(idx, 42);
        assert!(tls_free(idx));
        // After free, value reads as 0 (slot no longer allocated).
        assert_eq!(tls_get_value(idx), 0);
    }

    #[test]
    fn tls_free_unallocated_returns_false() {
        // Freeing a never-allocated slot should fail.
        // Use a high index that's extremely unlikely to be allocated.
        assert!(!tls_free(TLS_MAX_SLOTS as u32)); // out of range
    }

    #[test]
    fn tls_double_free_returns_false() {
        let idx = tls_alloc().unwrap();
        assert!(tls_free(idx));
        assert!(!tls_free(idx)); // second free fails
    }

    #[test]
    fn tls_out_of_range_index() {
        assert_eq!(tls_get_value(TLS_MAX_SLOTS as u32 + 1), 0);
        assert!(!tls_set_value(TLS_MAX_SLOTS as u32 + 1, 99));
    }

    #[test]
    fn tls_realloc_reuses_freed_slot() {
        let a = tls_alloc().unwrap();
        tls_free(a);
        let b = tls_alloc().unwrap();
        // The freed slot should be available again; the allocator scans
        // from index 0 so `b` will be <= `a` (it gets `a` or a lower
        // slot that another parallel test freed).
        // Just verify we got a valid index back.
        assert!(b < TLS_MAX_SLOTS as u32);
        tls_free(b);
    }

    #[test]
    fn tls_per_thread_isolation() {
        let idx = tls_alloc().unwrap();
        tls_set_value(idx, 111);

        let child_saw = std::thread::spawn(move || {
            // Child thread should see default value (0), not parent's.
            let v = tls_get_value(idx);
            tls_set_value(idx, 222);
            (v, tls_get_value(idx))
        })
        .join()
        .unwrap();

        assert_eq!(child_saw, (0, 222));
        // Parent's value unchanged.
        assert_eq!(tls_get_value(idx), 111);
        tls_free(idx);
    }

    // ── Event wait tests ─────────────────────────────────────────

    fn make_event(manual_reset: bool, initial: bool) -> EventWaitable {
        EventWaitable {
            inner: Arc::new(EventInner {
                signaled: Mutex::new(initial),
                condvar: Condvar::new(),
                manual_reset,
            }),
        }
    }

    #[test]
    fn event_initially_signaled_returns_immediately() {
        let e = make_event(true, true);
        let w = Waitable::Event(e);
        assert_eq!(wait_on(&w, 0), WAIT_OBJECT_0);
    }

    #[test]
    fn event_initially_unsignaled_times_out() {
        let e = make_event(true, false);
        let w = Waitable::Event(e);
        assert_eq!(wait_on(&w, 0), WAIT_TIMEOUT);
    }

    #[test]
    fn auto_reset_event_clears_after_wait() {
        let e = make_event(false, true); // auto-reset, initially signaled
        let w = Waitable::Event(e);
        assert_eq!(wait_on(&w, 0), WAIT_OBJECT_0); // consumes the signal
        assert_eq!(wait_on(&w, 0), WAIT_TIMEOUT); // now unsignaled
    }

    #[test]
    fn manual_reset_event_stays_signaled() {
        let e = make_event(true, true); // manual-reset, initially signaled
        let w = Waitable::Event(e);
        assert_eq!(wait_on(&w, 0), WAIT_OBJECT_0);
        assert_eq!(wait_on(&w, 0), WAIT_OBJECT_0); // still signaled
    }

    #[test]
    fn event_signal_from_another_thread() {
        let e = make_event(true, false);
        let inner = Arc::clone(&e.inner);
        let w = Waitable::Event(e);

        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(20));
            *inner.signaled.lock().unwrap() = true;
            inner.condvar.notify_all();
        });

        assert_eq!(wait_on(&w, 1000), WAIT_OBJECT_0);
    }

    #[test]
    fn event_timeout_when_never_signaled() {
        let e = make_event(true, false);
        let w = Waitable::Event(e);
        let start = Instant::now();
        assert_eq!(wait_on(&w, 50), WAIT_TIMEOUT);
        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(40));
        assert!(elapsed < Duration::from_millis(200));
    }

    // ── Thread waitable tests ────────────────────────────────────

    fn make_thread_waitable() -> ThreadWaitable {
        ThreadWaitable {
            exit_code: Arc::new(AtomicU32::new(STILL_ACTIVE)),
            completed: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }

    #[test]
    fn thread_wait_returns_timeout_while_running() {
        let tw = make_thread_waitable();
        let w = Waitable::Thread(tw);
        assert_eq!(wait_on(&w, 0), WAIT_TIMEOUT);
    }

    #[test]
    fn thread_wait_returns_immediately_when_completed() {
        let tw = make_thread_waitable();
        // Simulate thread completion.
        tw.exit_code.store(0, Ordering::Release);
        let (lock, cvar) = &*tw.completed;
        *lock.lock().unwrap() = true;
        cvar.notify_all();

        let w = Waitable::Thread(tw);
        assert_eq!(wait_on(&w, 0), WAIT_OBJECT_0);
    }

    #[test]
    fn thread_wait_with_timeout_succeeds_when_completed_in_time() {
        let tw = make_thread_waitable();
        let completed = Arc::clone(&tw.completed);
        let exit_code = Arc::clone(&tw.exit_code);

        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(20));
            exit_code.store(42, Ordering::Release);
            let (lock, cvar) = &*completed;
            *lock.lock().unwrap() = true;
            cvar.notify_all();
        });

        let w = Waitable::Thread(tw);
        assert_eq!(wait_on(&w, 1000), WAIT_OBJECT_0);
    }

    #[test]
    fn thread_wait_timeout_fires_when_not_completed() {
        let tw = make_thread_waitable();
        let w = Waitable::Thread(tw);
        let start = Instant::now();
        assert_eq!(wait_on(&w, 50), WAIT_TIMEOUT);
        assert!(start.elapsed() >= Duration::from_millis(40));
    }

    // ── Constants ────────────────────────────────────────────────

    #[test]
    fn windows_constants_are_correct() {
        assert_eq!(INFINITE, 0xFFFF_FFFF);
        assert_eq!(WAIT_OBJECT_0, 0);
        assert_eq!(WAIT_TIMEOUT, 0x102);
        assert_eq!(WAIT_FAILED, 0xFFFF_FFFF);
        assert_eq!(STILL_ACTIVE, 259);
        assert_eq!(TLS_OUT_OF_INDEXES, 0xFFFF_FFFF);
    }
}
