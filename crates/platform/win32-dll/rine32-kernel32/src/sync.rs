//! kernel32 32bit synchronisation objects: critical sections, events, mutexes, semaphores.
//!
//! This is mostly a thin wrapper around the common implementations in `rine-common-kernel32`,
//! but also includes the Windows API entry points and some handle table integration.

use std::ptr;

use rine_common_kernel32 as common;
use rine_types::errors::WinBool;
use rine_types::handles::{Handle, handle_table};
use rine_types::threading;
use tracing::debug;

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn InitializeCriticalSection(cs: *mut u8) {
    if cs.is_null() {
        return;
    }
    unsafe { common::sync::init_critical_section(cs) };
}

/// InitializeCriticalSectionAndSpinCount — spin count is ignored (always 0).
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn InitializeCriticalSectionAndSpinCount(
    cs: *mut u8,
    _spin_count: u32,
) -> WinBool {
    if cs.is_null() {
        return WinBool::FALSE;
    }

    unsafe { common::sync::init_critical_section(cs) };

    WinBool::TRUE
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn EnterCriticalSection(cs: *mut u8) {
    if cs.is_null() {
        return;
    }

    unsafe {
        let mut mutex = common::sync::get_mutex(cs);

        if mutex.is_null() {
            common::sync::init_critical_section(cs);
            mutex = common::sync::get_mutex(cs);
        }

        libc::pthread_mutex_lock(mutex);
    }
}

/// TryEnterCriticalSection — non-blocking lock attempt.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn TryEnterCriticalSection(cs: *mut u8) -> WinBool {
    if cs.is_null() {
        return WinBool::FALSE;
    }

    unsafe {
        let mutex = common::sync::get_mutex(cs);

        if mutex.is_null() {
            return WinBool::FALSE;
        }

        if libc::pthread_mutex_trylock(mutex) == 0 {
            WinBool::TRUE
        } else {
            WinBool::FALSE
        }
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn LeaveCriticalSection(cs: *mut u8) {
    if cs.is_null() {
        return;
    }

    unsafe {
        let mutex = common::sync::get_mutex(cs);

        if mutex.is_null() {
            return;
        }

        libc::pthread_mutex_unlock(mutex);
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn DeleteCriticalSection(cs: *mut u8) {
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

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn CreateEventA(
    _security_attrs: usize,
    manual_reset: WinBool,
    initial_state: WinBool,
    _name: *const u8,
) -> isize {
    let h = common::sync::create_event(manual_reset, initial_state);

    debug!(?h, "CreateEventA");
    rine_types::dev_notify!(on_handle_created(
        h.as_raw() as i64,
        "Event",
        if manual_reset.is_true() {
            "manual-reset"
        } else {
            "auto-reset"
        }
    ));

    h.as_raw()
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn CreateEventW(
    _security_attrs: usize,
    manual_reset: WinBool,
    initial_state: WinBool,
    _name: *const u16,
) -> isize {
    let h = common::sync::create_event(manual_reset, initial_state);

    debug!(?h, "CreateEventW");
    rine_types::dev_notify!(on_handle_created(
        h.as_raw() as i64,
        "Event",
        if manual_reset.is_true() {
            "manual-reset"
        } else {
            "auto-reset"
        }
    ));

    h.as_raw()
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn SetEvent(event_handle: isize) -> WinBool {
    let handle = Handle::from_raw(event_handle);
    let waitable = match handle_table().get_waitable(handle) {
        Some(threading::Waitable::Event(e)) => e,
        _ => return WinBool::FALSE,
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

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn ResetEvent(event_handle: isize) -> WinBool {
    let handle = Handle::from_raw(event_handle);
    let waitable = match handle_table().get_waitable(handle) {
        Some(threading::Waitable::Event(e)) => e,
        _ => return WinBool::FALSE,
    };

    let mut signaled = waitable.inner.signaled.lock().unwrap();
    *signaled = false;

    WinBool::TRUE
}

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
///
/// The caller must ensure that `name` points to a valid null-terminated ANSI string if it is not null.
/// The caller is responsible for managing the returned mutex handle, including closing it when no longer needed.
#[allow(non_snake_case)]
pub unsafe extern "stdcall" fn CreateMutexA(
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
pub unsafe extern "stdcall" fn CreateMutexW(
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
pub unsafe extern "stdcall" fn ReleaseMutex(mutex_handle: isize) -> WinBool {
    unsafe { common::sync::release_mutex(mutex_handle) }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn CreateSemaphoreA(
    _security_attrs: usize,
    initial_count: i32,
    maximum_count: i32,
    _name: *const u8,
) -> isize {
    if maximum_count <= 0 || initial_count < 0 || initial_count > maximum_count {
        return 0;
    }

    let handle = common::sync::create_semaphore(initial_count, maximum_count);

    debug!(?handle, "CreateSemaphoreA");
    rine_types::dev_notify!(on_handle_created(
        handle as i64,
        "SemaphoreA",
        &format!("initial={initial_count}, max={maximum_count}")
    ));

    handle as isize
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn CreateSemaphoreW(
    _security_attrs: usize,
    initial_count: i32,
    maximum_count: i32,
    _name: *const u16,
) -> isize {
    if maximum_count <= 0 || initial_count < 0 || initial_count > maximum_count {
        return 0;
    }

    let handle = common::sync::create_semaphore(initial_count, maximum_count);

    debug!(?handle, "CreateSemaphoreW");
    rine_types::dev_notify!(on_handle_created(
        handle as i64,
        "SemaphoreW",
        &format!("initial={initial_count}, max={maximum_count}")
    ));

    handle as isize
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn ReleaseSemaphore(
    semaphore_handle: isize,
    release_count: i32,
    previous_count: *mut i32,
) -> WinBool {
    if release_count <= 0 {
        return WinBool::FALSE;
    }

    let handle = Handle::from_raw(semaphore_handle);
    let waitable = match handle_table().get_waitable(handle) {
        Some(threading::Waitable::Semaphore(s)) => s,
        _ => return WinBool::FALSE,
    };

    let mut count = waitable.inner.count.lock().unwrap();
    let prev = *count;

    if prev + release_count > waitable.inner.max_count {
        return WinBool::FALSE;
    }

    if !previous_count.is_null() {
        unsafe { ptr::write(previous_count, prev) };
    }

    *count = prev + release_count;
    for _ in 0..release_count {
        waitable.inner.condvar.notify_one();
    }

    WinBool::TRUE
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
