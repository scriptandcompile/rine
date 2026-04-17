//! kernel32 64bit synchronisation objects: critical sections, events, mutexes, semaphores.
//!
//! This is mostly a thin wrapper around the common implementations in `rine-common-kernel32`,
//! but also includes the Windows API entry points and some handle table integration.

use std::ptr;

use rine_common_kernel32 as common;
use rine_types::errors::WinBool;
use rine_types::handles::{Handle, handle_table};

use tracing::{debug, warn};

/// Initialize a critical section.
///
/// # Arguments
/// * `cs` - pointer to a CRITICAL_SECTION structure to initialize. Must not be null.
///
/// # Safety
/// The caller must ensure that `cs` points to a valid memory location for a CRITICAL_SECTION structure.
/// Initializing a critical section on an invalid pointer may lead to undefined behavior.
/// The caller is responsible for ensuring that the CRITICAL_SECTION is properly deleted with
/// `DeleteCriticalSection` when no longer needed.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn InitializeCriticalSection(cs: *mut u8) {
    if cs.is_null() {
        return;
    }

    unsafe { common::sync::init_critical_section(cs) };
}

/// Initialize a critical section with a spin count.
///
/// # Arguments
/// * `cs` - pointer to a CRITICAL_SECTION structure to initialize. Must not be null.
/// * `_spin_count` - The spin count is currently ignored, as our critical section implementation does not support spinning.
///
/// # Safety
/// The caller must ensure that `cs` points to a valid memory location for a CRITICAL_SECTION structure.
/// Initializing a critical section on an invalid pointer may lead to undefined behavior.
/// The caller is responsible for ensuring that the CRITICAL_SECTION is properly deleted with
/// `DeleteCriticalSection` when no longer needed.
///
/// # Returns
/// If the critical section was successfully initialized, the function returns `WinBool::TRUE`.
/// If the `cs` pointer is null, the function returns `WinBool::FALSE` and does not perform any initialization.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn InitializeCriticalSectionAndSpinCount(
    cs: *mut u8,
    _spin_count: u32,
) -> WinBool {
    if cs.is_null() {
        return WinBool::FALSE;
    }

    unsafe { common::sync::init_critical_section(cs) };

    WinBool::TRUE
}

/// Enter a critical section by locking the underlying mutex.
///
/// # Arguments
/// * `cs` - A pointer to the critical section to enter. Must have been initialized with `init_critical_section`.
///
/// # Safety
/// The caller must ensure that `cs` is a valid pointer to a critical section that has been properly initialized.
/// The caller must also ensure that the critical section is not used after being deleted.
/// If `cs` is null, this function does nothing and returns immediately.
/// Otherwise, it will block until the mutex can be locked.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn EnterCriticalSection(cs: *mut u8) {
    unsafe {
        common::sync::enter_critical_section(cs);
    }
}

/// Try a non-blocking lock attempt.
///
/// # Arguments
/// * `cs` - pointer to the CRITICAL_SECTION structure representing the mutex to attempt to lock. Must not be null.
///
/// # Safety
/// The caller must ensure that `cs` points to a valid CRITICAL_SECTION structure.
/// Passing an invalid pointer or a pointer to an improperly initialized CRITICAL_SECTION may lead to undefined behavior.
/// The caller is responsible for ensuring that the CRITICAL_SECTION is properly initialized before calling this function.
///
/// # Returns
/// Returns `WinBool::TRUE` if the lock was successfully acquired, or `WinBool::FALSE`
/// if the critical section is already owned by another thread or if an error occurred (e.g. invalid pointer).
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn TryEnterCriticalSection(cs: *mut u8) -> WinBool {
    unsafe { common::sync::try_enter_critical_section(cs) }
}

/// Leave a critical section by unlocking the underlying mutex.
///
/// # Arguments
/// * `cs` - A pointer to the critical section to leave. Must have been initialized with `InitializeCriticalSection`.
///
/// # Safety
/// The caller must ensure that `cs` is a valid pointer to a critical section that has been properly initialized.
/// The caller must also ensure that the critical section is not used after being deleted.
/// If `cs` is null, this function does nothing and returns immediately.
///
/// # Returns
/// If the critical section was successfully left, the function returns `WinBool::TRUE`.
/// If the `cs` pointer is null, the function returns `WinBool::FALSE` and does not perform any operation.
///
/// # Notes
/// If the critical section was not owned by the calling thread, the behavior is undefined and may result in an error or deadlock.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn LeaveCriticalSection(cs: *mut u8) {
    unsafe { common::sync::leave_critical_section(cs) };
}

/// DeleteCriticalSection — destroy and deallocate the mutex.
#[allow(non_snake_case, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn DeleteCriticalSection(cs: *mut u8) {
    if cs.is_null() {
        return;
    }

    unsafe {
        let mutex = common::sync::get_mutex(cs);

        if mutex.is_null() {
            return;
        }

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
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn CreateEventA(
    _security_attrs: usize,
    manual_reset: WinBool,
    initial_state: WinBool,
    _name: *const u8,
) -> isize {
    let handle = common::sync::create_event(manual_reset, initial_state);

    debug!(?handle, "CreateEventA");
    rine_types::dev_notify!(on_handle_created(
        handle.as_raw() as i64,
        "Event",
        if manual_reset.is_true() {
            "manual-reset"
        } else {
            "auto-reset"
        }
    ));

    handle.as_raw()
}

/// CreateEventW — wide-string variant (name ignored).
#[allow(non_snake_case, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn CreateEventW(
    _security_attrs: usize,
    manual_reset: WinBool,
    initial_state: WinBool,
    _name: *const u16,
) -> isize {
    let handle = common::sync::create_event(manual_reset, initial_state);

    debug!(?handle, "CreateEventW");
    rine_types::dev_notify!(on_handle_created(
        handle.as_raw() as i64,
        "Event",
        if manual_reset.is_true() {
            "manual-reset"
        } else {
            "auto-reset"
        }
    ));

    handle.as_raw()
}

/// SetEvent — signal the event and wake waiters.
#[allow(non_snake_case, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn SetEvent(event_handle: isize) -> WinBool {
    let handle = Handle::from_raw(event_handle);

    let waitable = match handle_table().get_waitable(handle) {
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
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn ResetEvent(event_handle: isize) -> WinBool {
    let handle = Handle::from_raw(event_handle);

    let waitable = match handle_table().get_waitable(handle) {
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

/// Create a mutex object, optionally initially owned and with an optional (ANSI) name.
///
/// # Arguments
/// * `_security_attrs` - Currently ignored, as we do not implement any access control features.
/// * `initial_owner` - If TRUE, the creating thread takes initial ownership of the mutex,
///   meaning it must release it before another thread can acquire it. If FALSE, the mutex
///   is created in an unowned state.
/// * `name` - Currently ignored, as named mutexes are not implemented, but it is still read
///   and logged for dev notification purposes.
///
/// Returns a handle to the created mutex, or 0 on failure (e.g. invalid parameters).
///
/// # Safety
/// The caller must ensure that `name` points to a valid null-terminated ANSI string if it is not null.
/// The caller is responsible for managing the returned mutex handle, including closing it when no longer needed.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn CreateMutexA(
    _security_attrs: usize,
    initial_owner: WinBool,
    name: *const u8,
) -> isize {
    let name_str = unsafe { rine_types::strings::read_cstr(name) };
    let (handle, detail) = common::sync::create_mutex(initial_owner, name_str.clone());

    debug!(?handle, name = ?name_str, "CreateMutexA");
    rine_types::dev_notify!(on_handle_created(handle.as_raw() as i64, "Mutex", &detail));

    handle.as_raw()
}

/// Create a mutex object, optionally initially owned and with an optional (UTF-16) name.
///
/// # Arguments
/// * `_security_attrs` - Currently ignored, as we do not implement any access control features.
/// * `initial_owner` - If TRUE, the creating thread takes initial ownership of the mutex,
///   meaning it must release it before another thread can acquire it. If FALSE, the mutex
///   is created in an unowned state.
/// * `name` - Currently ignored, as named mutexes are not implemented, but it is still read
///   and logged for dev notification purposes.
///
/// Returns a handle to the created mutex, or 0 on failure (e.g. invalid parameters).
///
/// # Safety
///
/// The caller must ensure that `name` points to a valid null-terminated UTF-16 string if it is not null.
/// The caller is responsible for managing the returned mutex handle, including closing it when no longer needed.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn CreateMutexW(
    _security_attrs: usize,
    initial_owner: WinBool,
    name: *const u16,
) -> isize {
    let name_str = unsafe { rine_types::strings::read_wstr(name) };
    let (handle, detail) = common::sync::create_mutex(initial_owner, name_str.clone());

    debug!(?handle, name = ?name_str, "CreateMutexW");
    rine_types::dev_notify!(on_handle_created(handle.as_raw() as i64, "Mutex", &detail));

    handle.as_raw()
}

/// Release a mutex, decrementing its ownership count and potentially unblocking waiters.
///
/// Returns TRUE on success, FALSE on failure (e.g. invalid handle, not a mutex, or not owned by the caller).
///
/// # Arguments
///
/// * `mutex_handle` - A handle to the mutex to release. The caller must have ownership of
///   the mutex (i.e. have previously acquired it and not yet released it).
///
/// # Safety
///
/// The caller must ensure that `mutex_handle` is a valid handle to a mutex object that the caller currently owns.
/// Releasing a mutex that is not owned by the caller, or using an invalid handle, will result in failure and return FALSE.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn ReleaseMutex(mutex_handle: isize) -> WinBool {
    unsafe { common::sync::release_mutex(mutex_handle) }
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
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn CreateSemaphoreA(
    _security_attrs: usize,
    initial_count: i32,
    maximum_count: i32,
    _name: *const u8,
) -> isize {
    let handle = common::sync::create_semaphore(initial_count, maximum_count);

    debug!(?handle, "CreateSemaphoreA");
    rine_types::dev_notify!(on_handle_created(
        handle as i64,
        "SemaphoreA",
        &format!("initial={initial_count}, max={maximum_count}")
    ));

    handle
}

/// CreateSemaphoreW — wide-string variant (name ignored).
#[allow(non_snake_case, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn CreateSemaphoreW(
    _security_attrs: usize,
    initial_count: i32,
    maximum_count: i32,
    _name: *const u16,
) -> isize {
    let handle = common::sync::create_semaphore(initial_count, maximum_count);

    debug!(?handle, "CreateSemaphoreW");
    rine_types::dev_notify!(on_handle_created(
        handle as i64,
        "SemaphoreW",
        &format!("initial={initial_count}, max={maximum_count}")
    ));

    handle
}

/// Release a semaphore, incrementing its count by `release_count` and potentially unblocking waiters.
///
/// Returns TRUE on success, FALSE on failure (e.g. invalid handle, not a semaphore, or
/// release would exceed max count).
///
/// # Arguments
/// * `semaphore_handle` - A handle to the semaphore to release. The caller must have
///   appropriate access to the semaphore.
/// * `release_count` - The amount by which to increment the semaphore's count.
///   Must be greater than 0 and such that the resulting count does not exceed the semaphore's maximum count.
/// * `previous_count` - An optional pointer to receive the previous count of the
///   semaphore before the release. Can be null if the caller does not need this information.
///
/// # Safety
/// The caller must ensure that `semaphore_handle` is a valid handle to a semaphore object and
/// that the caller has appropriate access rights to release it. The caller must also ensure
/// that `release_count` is greater than 0 and that releasing the semaphore by this amount
/// will not cause its count to exceed the semaphore's maximum count.
///
/// If `previous_count` is not null, the caller must ensure that it points to a valid writable
/// memory location where an i32 can be stored. Releasing a semaphore with an invalid handle,
/// or with parameters that would exceed the maximum count, will result in failure and return FALSE.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn ReleaseSemaphore(
    semaphore_handle: isize,
    release_count: i32,
    previous_count: *mut i32,
) -> WinBool {
    unsafe { common::sync::release_semaphore(semaphore_handle, release_count, previous_count) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rine_types::threading::{WaitStatus, wait_on};

    use std::ptr;
    use std::sync::Arc;

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
            let w = handle_table().get_waitable(Handle::from_raw(h)).unwrap();
            assert_eq!(wait_on(&w, 0), WaitStatus::WAIT_OBJECT_0.0);

            assert!(ResetEvent(h).is_true());
            // Event is now unsignaled.
            let w = handle_table().get_waitable(Handle::from_raw(h)).unwrap();
            assert_eq!(wait_on(&w, 0), WaitStatus::WAIT_TIMEOUT.0);
        }
    }

    #[test]
    fn create_event_w_initially_signaled() {
        unsafe {
            let h = CreateEventW(0, WinBool::FALSE, WinBool::TRUE, ptr::null());
            assert_ne!(h, 0);
            let w = handle_table().get_waitable(Handle::from_raw(h)).unwrap();
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

            let w = handle_table().get_waitable(Handle::from_raw(h)).unwrap();
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
            let w = handle_table().get_waitable(Handle::from_raw(h)).unwrap();
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
            let w = handle_table().get_waitable(Handle::from_raw(h)).unwrap();
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
                let w = handle_table().get_waitable(Handle::from_raw(h)).unwrap();
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
            let w = handle_table().get_waitable(Handle::from_raw(h)).unwrap();

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
                let w = handle_table().get_waitable(Handle::from_raw(h)).unwrap();
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
