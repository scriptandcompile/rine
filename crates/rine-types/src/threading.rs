//! Thread-level state shared by rine DLL implementations.
//!
//! Provides TLS slot management and waitable-object types used by
//! kernel32 threading, synchronization, and the `Wait*` family.

use std::fmt;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use threading_runtime::{
    EventWaitRef, MutexStateLike, MutexWaitRef, ProcessWaitRef, SemaphoreWaitRef, ThreadWaitRef,
    WaitRef, WaitableLike,
};

// ── Windows constants ────────────────────────────────────────────

/// Infinite timeout value for wait functions (milliseconds).
pub const INFINITE: u32 = 0xFFFF_FFFF;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct WaitStatus(pub u32);

impl WaitStatus {
    /// The specified object is a mutex object that was not released by the
    /// thread that owned the mutex object before the owning thread terminated.
    /// Ownership of the mutex object is granted to the calling thread and the
    /// mutex state is set to nonsignaled.
    pub const WAIT_ABANDONED: Self = Self(0x0000_0080);
    /// Wait succeeded, and the object is signaled.
    pub const WAIT_OBJECT_0: Self = Self(0x0000_0000);
    /// The time-out interval elapsed, and the object's state is nonsignaled.
    pub const WAIT_TIMEOUT: Self = Self(0x0000_0102);
    /// The function has failed. To get extended error information, call
    /// `GetLastError()`.
    pub const WAIT_FAILED: Self = Self(0xFFFF_FFFF);
}

pub const STILL_ACTIVE: u32 = 259;
pub const TLS_OUT_OF_INDEXES: u32 = 0xFFFF_FFFF;

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

/// Shared state backing a process handle.
#[derive(Clone)]
pub struct ProcessWaitable {
    /// Exit code.  [`STILL_ACTIVE`] while the process is running.
    pub exit_code: Arc<AtomicU32>,
    /// (`done` flag, condvar) — signalled when the process exits.
    pub completed: Arc<(Mutex<bool>, Condvar)>,
    /// Linux PID of the child process.
    pub pid: u32,
}

impl fmt::Debug for ProcessWaitable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProcessWaitable")
            .field("pid", &self.pid)
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

/// Shared state backing a mutex handle (`CreateMutex`).
#[derive(Clone)]
pub struct MutexWaitable {
    pub inner: Arc<MutexInner>,
}

impl fmt::Debug for MutexWaitable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MutexWaitable").finish_non_exhaustive()
    }
}

/// Interior of a Windows mutex object.
pub struct MutexInner {
    /// `None` = unlocked, `Some(tid)` = locked by that thread.
    pub state: Mutex<MutexState>,
    pub condvar: Condvar,
}

/// Run-time state of a Windows mutex.
pub struct MutexState {
    pub owner: Option<thread::ThreadId>,
    pub count: u32,
}

/// Shared state backing a semaphore handle (`CreateSemaphore`).
#[derive(Clone)]
pub struct SemaphoreWaitable {
    pub inner: Arc<SemaphoreInner>,
}

impl fmt::Debug for SemaphoreWaitable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SemaphoreWaitable")
            .field("max_count", &self.inner.max_count)
            .finish_non_exhaustive()
    }
}

/// Interior of a Windows semaphore object.
pub struct SemaphoreInner {
    pub count: Mutex<i32>,
    pub max_count: i32,
    pub condvar: Condvar,
}

/// A waitable object extracted from the handle table (Arc-cloned so the
/// table lock is not held during the wait).
#[derive(Clone)]
pub enum Waitable {
    Thread(ThreadWaitable),
    Event(EventWaitable),
    Process(ProcessWaitable),
    Mutex(MutexWaitable),
    Semaphore(SemaphoreWaitable),
}

impl MutexStateLike for MutexState {
    fn owner(&self) -> Option<thread::ThreadId> {
        self.owner
    }

    fn set_owner(&mut self, owner: Option<thread::ThreadId>) {
        self.owner = owner;
    }

    fn count(&self) -> u32 {
        self.count
    }

    fn set_count(&mut self, count: u32) {
        self.count = count;
    }
}

impl WaitableLike for Waitable {
    type MutexState = MutexState;

    fn as_wait_ref(&self) -> WaitRef<'_, Self::MutexState> {
        match self {
            Waitable::Thread(thread) => WaitRef::Thread(ThreadWaitRef {
                completed: &thread.completed,
            }),
            Waitable::Process(process) => WaitRef::Process(ProcessWaitRef {
                completed: &process.completed,
            }),
            Waitable::Event(event) => WaitRef::Event(EventWaitRef {
                signaled: &event.inner.signaled,
                condvar: &event.inner.condvar,
                manual_reset: event.inner.manual_reset,
            }),
            Waitable::Mutex(mutex) => WaitRef::Mutex(MutexWaitRef {
                state: &mutex.inner.state,
                condvar: &mutex.inner.condvar,
            }),
            Waitable::Semaphore(semaphore) => WaitRef::Semaphore(SemaphoreWaitRef {
                count: &semaphore.inner.count,
                condvar: &semaphore.inner.condvar,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Constants ────────────────────────────────────────────────

    #[test]
    fn windows_constants_are_correct() {
        assert_eq!(INFINITE, 0xFFFF_FFFF);

        assert_eq!(WaitStatus::WAIT_OBJECT_0.0, 0);
        assert_eq!(WaitStatus::WAIT_TIMEOUT.0, 0x102);
        assert_eq!(WaitStatus::WAIT_FAILED.0, 0xFFFF_FFFF);

        assert_eq!(STILL_ACTIVE, 259);
        assert_eq!(TLS_OUT_OF_INDEXES, 0xFFFF_FFFF);
    }
}
