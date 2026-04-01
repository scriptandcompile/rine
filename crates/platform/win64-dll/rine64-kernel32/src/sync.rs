#![allow(non_snake_case)]
//! kernel32 synchronisation: critical sections backed by `pthread_mutex`.
//!
//! A Windows `CRITICAL_SECTION` is 40 bytes on x86-64.  We heap-allocate a
//! recursive `pthread_mutex_t` and store its pointer in the first 8 bytes
//! of the caller-supplied struct.  The remaining bytes are zeroed.

use std::ptr;

use rine_types::errors::WinBool;
use rine_types::handles::{Handle, HandleEntry, handle_table};
use rine_types::threading::{
    EventInner, EventWaitable, MutexInner, MutexState, MutexWaitable, SemaphoreInner,
    SemaphoreWaitable, Waitable,
};
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
        return WinBool::FALSE;
    }
    unsafe { init_cs(cs) };
    WinBool::TRUE
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
        return WinBool::FALSE;
    }
    let mutex = unsafe { get_mutex(cs) };
    if mutex.is_null() {
        return WinBool::FALSE;
    }
    if unsafe { libc::pthread_mutex_trylock(mutex) } == 0 {
        WinBool::TRUE
    } else {
        WinBool::FALSE
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
            signaled: Mutex::new(initial_state.is_true()),
            condvar: Condvar::new(),
            manual_reset: manual_reset.is_true(),
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
            signaled: Mutex::new(initial_state.is_true()),
            condvar: Condvar::new(),
            manual_reset: manual_reset.is_true(),
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
            return WinBool::FALSE;
        }
    };
    let mut signaled = waitable.inner.signaled.lock().unwrap();
    *signaled = true;
    if waitable.inner.manual_reset {
        waitable.inner.condvar.notify_all();
    } else {
        waitable.inner.condvar.notify_one();
    }
    WinBool::TRUE
}

/// ResetEvent — clear the signalled state.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn ResetEvent(event_handle: isize) -> WinBool {
    let h = Handle::from_raw(event_handle);
    let waitable = match handle_table().get_waitable(h) {
        Some(rine_types::threading::Waitable::Event(e)) => e,
        _ => {
            warn!(handle = event_handle, "ResetEvent: invalid handle");
            return WinBool::FALSE;
        }
    };
    let mut signaled = waitable.inner.signaled.lock().unwrap();
    *signaled = false;
    WinBool::TRUE
}

// ── Mutexes ──────────────────────────────────────────────────────

/// CreateMutexA — create a named or unnamed mutex object (name ignored).
///
/// ```c
/// HANDLE CreateMutexA(
///     LPSECURITY_ATTRIBUTES lpMutexAttributes,  // rcx (ignored)
///     BOOL  bInitialOwner,                       // rdx
///     LPCSTR lpName                              // r8 (ignored)
/// );
/// ```
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn CreateMutexA(
    _security_attrs: usize,
    initial_owner: WinBool,
    _name: *const u8,
) -> isize {
    create_mutex_impl(initial_owner, "CreateMutexA")
}

/// CreateMutexW — wide-string variant (name ignored).
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn CreateMutexW(
    _security_attrs: usize,
    initial_owner: WinBool,
    _name: *const u16,
) -> isize {
    create_mutex_impl(initial_owner, "CreateMutexW")
}

fn create_mutex_impl(initial_owner: WinBool, tag: &str) -> isize {
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
    debug!(?h, tag);
    h.as_raw()
}

/// ReleaseMutex — release ownership of the mutex.
///
/// The calling thread must own the mutex.  Returns FALSE on error.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn ReleaseMutex(mutex_handle: isize) -> WinBool {
    let h = Handle::from_raw(mutex_handle);
    let waitable = match handle_table().get_waitable(h) {
        Some(Waitable::Mutex(m)) => m,
        _ => {
            warn!(handle = mutex_handle, "ReleaseMutex: invalid handle");
            return WinBool::FALSE;
        }
    };
    let tid = std::thread::current().id();
    let mut state = waitable.inner.state.lock().unwrap();
    if state.owner != Some(tid) {
        warn!(
            handle = mutex_handle,
            "ReleaseMutex: caller does not own mutex"
        );
        return WinBool::FALSE;
    }
    state.count -= 1;
    if state.count == 0 {
        state.owner = None;
        waitable.inner.condvar.notify_one();
    }
    WinBool::TRUE
}

// ── Semaphores ───────────────────────────────────────────────────

/// CreateSemaphoreA — create a named or unnamed semaphore (name ignored).
///
/// ```c
/// HANDLE CreateSemaphoreA(
///     LPSECURITY_ATTRIBUTES lpSemaphoreAttributes,  // rcx (ignored)
///     LONG  lInitialCount,                           // rdx
///     LONG  lMaximumCount,                           // r8
///     LPCSTR lpName                                  // r9 (ignored)
/// );
/// ```
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn CreateSemaphoreA(
    _security_attrs: usize,
    initial_count: i32,
    maximum_count: i32,
    _name: *const u8,
) -> isize {
    create_semaphore_impl(initial_count, maximum_count, "CreateSemaphoreA")
}

/// CreateSemaphoreW — wide-string variant (name ignored).
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn CreateSemaphoreW(
    _security_attrs: usize,
    initial_count: i32,
    maximum_count: i32,
    _name: *const u16,
) -> isize {
    create_semaphore_impl(initial_count, maximum_count, "CreateSemaphoreW")
}

fn create_semaphore_impl(initial_count: i32, maximum_count: i32, tag: &str) -> isize {
    if maximum_count <= 0 || initial_count < 0 || initial_count > maximum_count {
        warn!(
            initial_count,
            maximum_count, "CreateSemaphore: invalid parameters"
        );
        return 0; // NULL handle
    }

    let waitable = SemaphoreWaitable {
        inner: Arc::new(SemaphoreInner {
            count: Mutex::new(initial_count),
            max_count: maximum_count,
            condvar: Condvar::new(),
        }),
    };
    let h = handle_table().insert(HandleEntry::Semaphore(waitable));
    debug!(?h, tag);
    h.as_raw()
}

/// ReleaseSemaphore — increment the semaphore count.
///
/// ```c
/// BOOL ReleaseSemaphore(
///     HANDLE hSemaphore,        // rcx
///     LONG   lReleaseCount,     // rdx
///     LPLONG lpPreviousCount    // r8  (optional, may be NULL)
/// );
/// ```
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn ReleaseSemaphore(
    semaphore_handle: isize,
    release_count: i32,
    previous_count: *mut i32,
) -> WinBool {
    if release_count <= 0 {
        warn!(release_count, "ReleaseSemaphore: release_count must be > 0");
        return WinBool::FALSE;
    }

    let h = Handle::from_raw(semaphore_handle);
    let waitable = match handle_table().get_waitable(h) {
        Some(Waitable::Semaphore(s)) => s,
        _ => {
            warn!(
                handle = semaphore_handle,
                "ReleaseSemaphore: invalid handle"
            );
            return WinBool::FALSE;
        }
    };

    let mut count = waitable.inner.count.lock().unwrap();
    let prev = *count;

    if prev + release_count > waitable.inner.max_count {
        warn!(
            prev,
            release_count,
            max = waitable.inner.max_count,
            "ReleaseSemaphore: would exceed maximum count"
        );
        return WinBool::FALSE;
    }

    if !previous_count.is_null() {
        unsafe { ptr::write(previous_count, prev) };
    }

    *count = prev + release_count;

    // Wake up to `release_count` waiters.
    for _ in 0..release_count {
        waitable.inner.condvar.notify_one();
    }

    WinBool::TRUE
}

#[cfg(test)]
mod tests {
    use super::*;
    use rine_types::threading::{wait_on, WaitStatus};
    use std::ptr;

    // ── Critical Section tests ───────────────────────────────────

    #[test]
    fn critical_section_init_enter_leave_delete() {
        let mut cs = [0u8; 40];
        unsafe {
            InitializeCriticalSection(cs.as_mut_ptr());
            EnterCriticalSection(cs.as_mut_ptr());
            LeaveCriticalSection(cs.as_mut_ptr());
            DeleteCriticalSection(cs.as_mut_ptr());
        }
    }

    #[test]
    fn critical_section_recursive_entry() {
        let mut cs = [0u8; 40];
        unsafe {
            InitializeCriticalSection(cs.as_mut_ptr());
            // Recursive lock on same thread should not deadlock.
            EnterCriticalSection(cs.as_mut_ptr());
            EnterCriticalSection(cs.as_mut_ptr());
            LeaveCriticalSection(cs.as_mut_ptr());
            LeaveCriticalSection(cs.as_mut_ptr());
            DeleteCriticalSection(cs.as_mut_ptr());
        }
    }

    #[test]
    fn critical_section_and_spin_count() {
        let mut cs = [0u8; 40];
        unsafe {
            let result = InitializeCriticalSectionAndSpinCount(cs.as_mut_ptr(), 4000);
            assert!(result.is_true());
            EnterCriticalSection(cs.as_mut_ptr());
            LeaveCriticalSection(cs.as_mut_ptr());
            DeleteCriticalSection(cs.as_mut_ptr());
        }
    }

    #[test]
    fn try_enter_critical_section_succeeds_when_free() {
        let mut cs = [0u8; 40];
        unsafe {
            InitializeCriticalSection(cs.as_mut_ptr());
            let result = TryEnterCriticalSection(cs.as_mut_ptr());
            assert!(result.is_true());
            LeaveCriticalSection(cs.as_mut_ptr());
            DeleteCriticalSection(cs.as_mut_ptr());
        }
    }

    #[test]
    fn critical_section_null_is_noop() {
        unsafe {
            InitializeCriticalSection(ptr::null_mut());
            EnterCriticalSection(ptr::null_mut());
            LeaveCriticalSection(ptr::null_mut());
            DeleteCriticalSection(ptr::null_mut());
        }
    }

    // ── Event tests ──────────────────────────────────────────────

    #[test]
    fn create_event_and_set_reset() {
        unsafe {
            let h = CreateEventA(0, WinBool::TRUE, WinBool::FALSE, ptr::null());
            assert_ne!(h, 0);

            assert!(SetEvent(h).is_true());
            // Event is signaled, wait should succeed.
            let w = handle_table()
                .get_waitable(Handle::from_raw(h))
                .unwrap();
            assert_eq!(wait_on(&w, 0), WaitStatus::WAIT_OBJECT_0.0);

            assert!(ResetEvent(h).is_true());
            // Event is now unsignaled.
            let w = handle_table()
                .get_waitable(Handle::from_raw(h))
                .unwrap();
            assert_eq!(wait_on(&w, 0), WaitStatus::WAIT_TIMEOUT.0);
        }
    }

    #[test]
    fn create_event_w_initially_signaled() {
        unsafe {
            let h = CreateEventW(0, WinBool::FALSE, WinBool::TRUE, ptr::null());
            assert_ne!(h, 0);
            let w = handle_table()
                .get_waitable(Handle::from_raw(h))
                .unwrap();
            // Auto-reset, initially signaled — first wait succeeds, second times out.
            assert_eq!(wait_on(&w, 0), WaitStatus::WAIT_OBJECT_0.0);
            assert_eq!(wait_on(&w, 0), WaitStatus::WAIT_TIMEOUT.0);
        }
    }

    #[test]
    fn set_event_invalid_handle_returns_false() {
        unsafe {
            assert!(!SetEvent(0xDEAD).is_true());
            assert!(!ResetEvent(0xDEAD).is_true());
        }
    }

    // ── Mutex tests ──────────────────────────────────────────────

    #[test]
    fn create_mutex_unowned_and_wait() {
        unsafe {
            let h = CreateMutexA(0, WinBool::FALSE, ptr::null());
            assert_ne!(h, 0);

            let w = handle_table()
                .get_waitable(Handle::from_raw(h))
                .unwrap();
            // Unowned mutex should be immediately acquirable.
            assert_eq!(wait_on(&w, 0), WaitStatus::WAIT_OBJECT_0.0);
        }
    }

    #[test]
    fn create_mutex_initially_owned() {
        unsafe {
            let h = CreateMutexA(0, WinBool::TRUE, ptr::null());
            assert_ne!(h, 0);

            // Same thread can recursively acquire.
            let w = handle_table()
                .get_waitable(Handle::from_raw(h))
                .unwrap();
            assert_eq!(wait_on(&w, 0), WaitStatus::WAIT_OBJECT_0.0);
        }
    }

    #[test]
    fn create_mutex_w_variant_works() {
        unsafe {
            let h = CreateMutexW(0, WinBool::FALSE, ptr::null());
            assert_ne!(h, 0);
        }
    }

    #[test]
    fn release_mutex_by_owner_succeeds() {
        unsafe {
            let h = CreateMutexA(0, WinBool::TRUE, ptr::null());
            assert!(ReleaseMutex(h).is_true());
        }
    }

    #[test]
    fn release_mutex_not_owned_fails() {
        unsafe {
            // Create unowned mutex.
            let h = CreateMutexA(0, WinBool::FALSE, ptr::null());
            // Nobody owns it, releasing should fail.
            assert!(!ReleaseMutex(h).is_true());
        }
    }

    #[test]
    fn release_mutex_invalid_handle_fails() {
        unsafe {
            assert!(!ReleaseMutex(0xDEAD).is_true());
        }
    }

    #[test]
    fn mutex_recursive_release() {
        unsafe {
            let h = CreateMutexA(0, WinBool::TRUE, ptr::null());
            // Recursive acquire.
            let w = handle_table()
                .get_waitable(Handle::from_raw(h))
                .unwrap();
            assert_eq!(wait_on(&w, 0), WaitStatus::WAIT_OBJECT_0.0); // count = 2

            // First release (count → 1): still owned.
            assert!(ReleaseMutex(h).is_true());
            // Second release (count → 0): now unowned.
            assert!(ReleaseMutex(h).is_true());
            // Third release: not owned anymore → fail.
            assert!(!ReleaseMutex(h).is_true());
        }
    }

    #[test]
    fn mutex_cross_thread_contention() {
        use std::sync::atomic::{AtomicBool, Ordering};

        unsafe {
            let h = CreateMutexA(0, WinBool::TRUE, ptr::null());
            let released = Arc::new(AtomicBool::new(false));
            let released2 = Arc::clone(&released);

            // Spawn a thread that tries to acquire (should block/timeout).
            let child = std::thread::spawn(move || {
                let w = handle_table()
                    .get_waitable(Handle::from_raw(h))
                    .unwrap();
                let result = wait_on(&w, 10);
                // Should timeout because parent holds it.
                assert_eq!(result, WaitStatus::WAIT_TIMEOUT.0);

                // Wait for parent to release.
                while !released2.load(Ordering::Acquire) {
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }

                // Now it should be acquirable.
                let result = wait_on(&w, 1000);
                assert_eq!(result, WaitStatus::WAIT_OBJECT_0.0);
            });

            // Give child time to try and timeout.
            std::thread::sleep(std::time::Duration::from_millis(50));
            ReleaseMutex(h);
            released.store(true, Ordering::Release);

            child.join().unwrap();
        }
    }

    // ── Semaphore tests ──────────────────────────────────────────

    #[test]
    fn create_semaphore_valid_params() {
        unsafe {
            let h = CreateSemaphoreA(0, 2, 5, ptr::null());
            assert_ne!(h, 0);
        }
    }

    #[test]
    fn create_semaphore_w_variant_works() {
        unsafe {
            let h = CreateSemaphoreW(0, 1, 10, ptr::null());
            assert_ne!(h, 0);
        }
    }

    #[test]
    fn create_semaphore_invalid_params_returns_null() {
        unsafe {
            // max_count <= 0
            assert_eq!(CreateSemaphoreA(0, 0, 0, ptr::null()), 0);
            // initial_count < 0
            assert_eq!(CreateSemaphoreA(0, -1, 5, ptr::null()), 0);
            // initial_count > max_count
            assert_eq!(CreateSemaphoreA(0, 6, 5, ptr::null()), 0);
        }
    }

    #[test]
    fn semaphore_wait_and_release() {
        unsafe {
            let h = CreateSemaphoreA(0, 1, 5, ptr::null());
            let w = handle_table()
                .get_waitable(Handle::from_raw(h))
                .unwrap();

            // Count is 1, first wait succeeds.
            assert_eq!(wait_on(&w, 0), WaitStatus::WAIT_OBJECT_0.0);
            // Count is now 0, second wait times out.
            assert_eq!(wait_on(&w, 0), WaitStatus::WAIT_TIMEOUT.0);

            // Release one.
            let mut prev: i32 = -1;
            assert!(ReleaseSemaphore(h, 1, &mut prev).is_true());
            assert_eq!(prev, 0);

            // Now acquirable again.
            assert_eq!(wait_on(&w, 0), WaitStatus::WAIT_OBJECT_0.0);
        }
    }

    #[test]
    fn release_semaphore_exceeding_max_fails() {
        unsafe {
            let h = CreateSemaphoreA(0, 3, 5, ptr::null());
            // Try to release 3, which would bring count to 6 > max 5.
            assert!(!ReleaseSemaphore(h, 3, ptr::null_mut()).is_true());
        }
    }

    #[test]
    fn release_semaphore_null_previous_count() {
        unsafe {
            let h = CreateSemaphoreA(0, 0, 5, ptr::null());
            assert!(ReleaseSemaphore(h, 1, ptr::null_mut()).is_true());
        }
    }

    #[test]
    fn release_semaphore_zero_count_fails() {
        unsafe {
            let h = CreateSemaphoreA(0, 1, 5, ptr::null());
            assert!(!ReleaseSemaphore(h, 0, ptr::null_mut()).is_true());
        }
    }

    #[test]
    fn release_semaphore_invalid_handle_fails() {
        unsafe {
            assert!(!ReleaseSemaphore(0xDEAD, 1, ptr::null_mut()).is_true());
        }
    }

    #[test]
    fn semaphore_cross_thread_release_wakes_waiter() {
        unsafe {
            let h = CreateSemaphoreA(0, 0, 5, ptr::null());

            let child = std::thread::spawn(move || {
                let w = handle_table()
                    .get_waitable(Handle::from_raw(h))
                    .unwrap();
                wait_on(&w, 2000)
            });

            // Give child time to start waiting.
            std::thread::sleep(std::time::Duration::from_millis(30));

            // Release from main thread.
            assert!(ReleaseSemaphore(h, 1, ptr::null_mut()).is_true());

            assert_eq!(child.join().unwrap(), WaitStatus::WAIT_OBJECT_0.0);
        }
    }
}
