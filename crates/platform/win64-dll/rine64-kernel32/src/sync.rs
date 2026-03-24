#![allow(non_snake_case)]
//! kernel32 synchronisation: critical sections backed by `pthread_mutex`.
//!
//! A Windows `CRITICAL_SECTION` is 40 bytes on x86-64.  We heap-allocate a
//! recursive `pthread_mutex_t` and store its pointer in the first 8 bytes
//! of the caller-supplied struct.  The remaining bytes are zeroed.

use std::ptr;

use rine_types::errors::WinBool;
use rine_types::handles::{Handle, HandleEntry, handle_table};
use rine_types::threading::{EventInner, EventWaitable};
use std::sync::{Arc, Condvar, Mutex};
use tracing::{debug, warn};

// ── Critical Section ─────────────────────────────────────────────

/// Shared init: allocate a recursive `pthread_mutex_t` and store its
/// pointer in the first 8 bytes of the CRITICAL_SECTION.
unsafe fn init_cs(cs: *mut u8) {
    unsafe { ptr::write_bytes(cs, 0, 40) };

    let mutex = Box::into_raw(Box::new(unsafe {
        core::mem::zeroed::<libc::pthread_mutex_t>()
    }));

    unsafe {
        let mut attr: libc::pthread_mutexattr_t = core::mem::zeroed();
        libc::pthread_mutexattr_init(&mut attr);
        libc::pthread_mutexattr_settype(&mut attr, libc::PTHREAD_MUTEX_RECURSIVE);
        libc::pthread_mutex_init(mutex, &attr);
        libc::pthread_mutexattr_destroy(&mut attr);

        ptr::write(cs as *mut *mut libc::pthread_mutex_t, mutex);
    }
}

/// Read the mutex pointer from a CRITICAL_SECTION.
#[inline]
unsafe fn get_mutex(cs: *const u8) -> *mut libc::pthread_mutex_t {
    unsafe { ptr::read(cs as *const *mut libc::pthread_mutex_t) }
}

/// InitializeCriticalSection
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn InitializeCriticalSection(cs: *mut u8) {
    if cs.is_null() {
        return;
    }
    unsafe { init_cs(cs) };
}

/// InitializeCriticalSectionAndSpinCount — spin count is ignored (always 0).
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn InitializeCriticalSectionAndSpinCount(
    cs: *mut u8,
    _spin_count: u32,
) -> WinBool {
    if cs.is_null() {
        return rine_types::errors::FALSE;
    }
    unsafe { init_cs(cs) };
    rine_types::errors::TRUE
}

/// EnterCriticalSection — lock the recursive mutex.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn EnterCriticalSection(cs: *mut u8) {
    if cs.is_null() {
        return;
    }
    let mutex = unsafe { get_mutex(cs) };
    if mutex.is_null() {
        // Lazy init for zero-initialised CRITICAL_SECTIONs.
        unsafe { init_cs(cs) };
        let mutex = unsafe { get_mutex(cs) };
        unsafe { libc::pthread_mutex_lock(mutex) };
        return;
    }
    unsafe { libc::pthread_mutex_lock(mutex) };
}

/// TryEnterCriticalSection — non-blocking lock attempt.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn TryEnterCriticalSection(cs: *mut u8) -> WinBool {
    if cs.is_null() {
        return rine_types::errors::FALSE;
    }
    let mutex = unsafe { get_mutex(cs) };
    if mutex.is_null() {
        return rine_types::errors::FALSE;
    }
    if unsafe { libc::pthread_mutex_trylock(mutex) } == 0 {
        rine_types::errors::TRUE
    } else {
        rine_types::errors::FALSE
    }
}

/// LeaveCriticalSection — unlock the recursive mutex.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn LeaveCriticalSection(cs: *mut u8) {
    if cs.is_null() {
        return;
    }
    let mutex = unsafe { get_mutex(cs) };
    if mutex.is_null() {
        return;
    }
    unsafe { libc::pthread_mutex_unlock(mutex) };
}

/// DeleteCriticalSection — destroy and deallocate the mutex.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn DeleteCriticalSection(cs: *mut u8) {
    if cs.is_null() {
        return;
    }
    let mutex = unsafe { get_mutex(cs) };
    if mutex.is_null() {
        return;
    }
    unsafe {
        libc::pthread_mutex_destroy(mutex);
        drop(Box::from_raw(mutex));
        ptr::write(cs as *mut *mut libc::pthread_mutex_t, ptr::null_mut());
    }
}

// ── Events ───────────────────────────────────────────────────────

/// CreateEventA — create an event object (name ignored).
///
/// ```c
/// HANDLE CreateEventA(
///     LPSECURITY_ATTRIBUTES lpEventAttributes,  // rcx (ignored)
///     BOOL  bManualReset,                        // rdx
///     BOOL  bInitialState,                       // r8
///     LPCSTR lpName                              // r9 (ignored)
/// );
/// ```
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn CreateEventA(
    _security_attrs: usize,
    manual_reset: WinBool,
    initial_state: WinBool,
    _name: *const u8,
) -> isize {
    let waitable = EventWaitable {
        inner: Arc::new(EventInner {
            signaled: Mutex::new(initial_state != 0),
            condvar: Condvar::new(),
            manual_reset: manual_reset != 0,
        }),
    };
    let h = handle_table().insert(HandleEntry::Event(waitable));
    debug!(?h, "CreateEventA");
    h.as_raw()
}

/// CreateEventW — wide-string variant (name ignored).
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn CreateEventW(
    _security_attrs: usize,
    manual_reset: WinBool,
    initial_state: WinBool,
    _name: *const u16,
) -> isize {
    let waitable = EventWaitable {
        inner: Arc::new(EventInner {
            signaled: Mutex::new(initial_state != 0),
            condvar: Condvar::new(),
            manual_reset: manual_reset != 0,
        }),
    };
    let h = handle_table().insert(HandleEntry::Event(waitable));
    debug!(?h, "CreateEventW");
    h.as_raw()
}

/// SetEvent — signal the event and wake waiters.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn SetEvent(event_handle: isize) -> WinBool {
    let h = Handle::from_raw(event_handle);
    let waitable = match handle_table().get_waitable(h) {
        Some(rine_types::threading::Waitable::Event(e)) => e,
        _ => {
            warn!(handle = event_handle, "SetEvent: invalid handle");
            return rine_types::errors::FALSE;
        }
    };
    let mut signaled = waitable.inner.signaled.lock().unwrap();
    *signaled = true;
    if waitable.inner.manual_reset {
        waitable.inner.condvar.notify_all();
    } else {
        waitable.inner.condvar.notify_one();
    }
    rine_types::errors::TRUE
}

/// ResetEvent — clear the signalled state.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn ResetEvent(event_handle: isize) -> WinBool {
    let h = Handle::from_raw(event_handle);
    let waitable = match handle_table().get_waitable(h) {
        Some(rine_types::threading::Waitable::Event(e)) => e,
        _ => {
            warn!(handle = event_handle, "ResetEvent: invalid handle");
            return rine_types::errors::FALSE;
        }
    };
    let mut signaled = waitable.inner.signaled.lock().unwrap();
    *signaled = false;
    rine_types::errors::TRUE
}
