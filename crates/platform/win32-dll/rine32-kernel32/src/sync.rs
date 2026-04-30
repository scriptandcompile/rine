//! kernel32 32bit synchronisation objects: critical sections, events, mutexes, semaphores.
//!
//! This is mostly a thin wrapper around the common implementations in `rine-common-kernel32`,
//! but also includes the Windows API entry points and some handle table integration.

use rine_common_kernel32 as common;
use rine_types::errors::WinBool;
use rine_types::handles::Handle;
use rine_types::strings::{LPCSTR, LPCWSTR};
use rine_types::sync::LPCriticalSection;
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
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn InitializeCriticalSection(cs: LPCriticalSection) {
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
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn InitializeCriticalSectionAndSpinCount(
    cs: LPCriticalSection,
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
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn EnterCriticalSection(cs: LPCriticalSection) {
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
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn TryEnterCriticalSection(cs: LPCriticalSection) -> WinBool {
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
/// Missing implementation features:
/// - No Win32-accurate `GetLastError` mapping is provided for invalid-pointer
///   and unlock-error cases.
/// - Error handling does not map pthread failure codes to Win32 behavior.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn LeaveCriticalSection(cs: LPCriticalSection) {
    unsafe { common::sync::leave_critical_section(cs) };
}

/// Destroy and deallocate the mutex.
///
/// # Arguments
/// * `cs` - A pointer to the critical section to delete. Must have been initialized with `InitializeCriticalSection`.
///
/// # Safety
/// The caller must ensure that `cs` points to a valid CRITICAL_SECTION structure that has been
/// properly initialized and is not currently in use.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn DeleteCriticalSection(cs: LPCriticalSection) {
    unsafe {
        common::sync::delete_critical_section(cs);
    }
}

/// Create an event object, which can be in a signaled or non-signaled state and can be waited on by threads.
///
/// # Arguments
/// * `_security_attrs` - Currently ignored, as we do not implement any access control features.
/// * `manual_reset` - If `WinBool::TRUE`, the event is a manual-reset event that remains signaled until explicitly reset.
///   If `WinBool::FALSE`, it is an auto-reset event that automatically resets to non-signaled after releasing a single waiting thread.
/// * `initial_state` - If `WinBool::TRUE`, the event is initially signaled; if `WinBool::FALSE`, it is initially non-signaled.
/// * `_name` - Currently ignored, as named events are not implemented, but it is still read for dev notification purposes.
///
/// # Safety
/// The caller must ensure that if `_name` is not null, it points to a valid null-terminated string
/// (ANSI for CreateEventA, UTF-16 for CreateEventW).
/// The caller is responsible for managing the returned event handle, including closing it when no longer needed.
///
/// # Returns
/// Returns a handle to the created event, or 0 on failure (e.g. invalid parameters).
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn CreateEventA(
    _security_attrs: usize,
    manual_reset: WinBool,
    initial_state: WinBool,
    _name: LPCSTR,
) -> Handle {
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

    h
}

/// Create an event object, which can be in a signaled or non-signaled state and can be waited on by threads.
///
/// # Arguments
/// * `_security_attrs` - Currently ignored, as we do not implement any access control features.
/// * `manual_reset` - If `WinBool::TRUE`, the event is a manual-reset event that remains signaled until explicitly reset.
///   If `WinBool::FALSE`, it is an auto-reset event that automatically resets to non-signaled after releasing a single waiting thread.
/// * `initial_state` - If `WinBool::TRUE`, the event is initially signaled; if `WinBool::FALSE`, it is initially non-signaled.
/// * `_name` - Currently ignored, as named events are not implemented, but it is still read for dev notification purposes.
///
/// # Safety
/// The caller must ensure that if `_name` is not null, it points to a valid null-terminated string
/// (ANSI for CreateEventA, UTF-16 for CreateEventW).
/// The caller is responsible for managing the returned event handle, including closing it when no longer needed.
///
/// # Returns
/// Returns a handle to the created event, or 0 on failure (e.g. invalid parameters).
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn CreateEventW(
    _security_attrs: usize,
    manual_reset: WinBool,
    initial_state: WinBool,
    _name: LPCWSTR,
) -> Handle {
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

    h
}

/// Set an event to the signaled state, releasing any waiting threads.
///
/// # Arguments
/// * `event_handle` - A handle to the event to set. The caller must have appropriate access rights to the event.
///
/// # Safety
/// The caller must ensure that `event_handle` is a valid handle to an event object and that the
/// caller has appropriate access rights to set it.
///
/// # Returns
/// Setting an event with an invalid handle will result in failure and return `WinBool::FALSE`.
/// If the event is successfully set to the signaled state, the function returns `WinBool::TRUE`
/// and any waiting threads are released according to the event's reset mode (manual or auto).
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn SetEvent(event_handle: Handle) -> WinBool {
    common::sync::set_event(event_handle)
}

/// Reset an event to the non-signaled state, causing threads that wait on it to block until it is set again.
///
/// # Arguments
/// * `event_handle` - A handle to the event to reset. The caller must have appropriate access rights to the event.
///
/// # Safety
/// The caller must ensure that `event_handle` is a valid handle to an event object and that the caller has
/// appropriate access rights to reset it.
///
/// # Returns
/// Resetting an event with an invalid handle will result in failure and return `WinBool::FALSE`.
/// If the event is successfully reset to the non-signaled state, the function returns `WinBool::TRUE`
/// and any threads that wait on it will block until it is set again.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn ResetEvent(event_handle: Handle) -> WinBool {
    common::sync::reset_event(event_handle)
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
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn CreateMutexA(
    _security_attrs: usize,
    initial_owner: WinBool,
    _name: LPCSTR,
) -> Handle {
    let name_str = unsafe { _name.read_string() };
    let (handle, detail) = common::sync::create_mutex(initial_owner, name_str.clone());

    debug!(?handle, name = ?name_str, "CreateMutexA");
    rine_types::dev_notify!(on_handle_created(handle.as_raw() as i64, "Mutex", &detail));

    handle
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
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn CreateMutexW(
    _security_attrs: usize,
    initial_owner: WinBool,
    _name: LPCWSTR,
) -> Handle {
    let name_str = unsafe { _name.read_string() };
    let (handle, detail) = common::sync::create_mutex(initial_owner, name_str.clone());

    debug!(?handle, name = ?name_str, "CreateMutexW");
    rine_types::dev_notify!(on_handle_created(handle.as_raw() as i64, "Mutex", &detail));

    handle
}

/// Release a mutex, decrementing its ownership count and potentially unblocking waiters.
///
/// # Arguments
/// * `mutex_handle` - A handle to the mutex to release. The caller must have ownership of
///   the mutex (i.e. have previously acquired it and not yet released it).
///
/// # Safety
/// The caller must ensure that `mutex_handle` is a valid handle to a mutex object that the caller currently owns.
/// Releasing a mutex that is not owned by the caller, or using an invalid handle, will result in failure and return `WinBool::FALSE`.
///
/// # Returns
/// If the mutex is successfully released, the function returns `WinBool::TRUE` and any waiting threads
/// are unblocked according to the mutex's behavior.
/// If the mutex handle is invalid or the caller does not have ownership of the mutex,
/// the function returns `WinBool::FALSE` and no action is taken.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn ReleaseMutex(mutex_handle: Handle) -> WinBool {
    unsafe { common::sync::release_mutex(mutex_handle) }
}

/// Create a semaphore object with the specified initial and maximum count, and with an optional (ANSI) name.
///
/// # Arguments
/// * `_security_attrs` - Currently ignored, as we do not implement any access control features.
/// * `initial_count` - The initial count for the semaphore. Must be non-negative and less than or equal to `maximum_count`.
/// * `maximum_count` - The maximum count for the semaphore. Must be greater than 0.
/// * `name` - Currently ignored, as named semaphores are not implemented, but it is still read and logged for dev notification purposes.
///
/// Returns a handle to the created semaphore, or 0 on failure (e.g. invalid parameters).
///
/// # Safety
/// The caller must ensure that `name` points to a valid null-terminated ANSI string if it is not null.
/// The caller is responsible for managing the returned semaphore handle, including closing it when no longer needed.
///
/// # Returns
/// If the parameters are valid and the semaphore is successfully created, the function returns a handle to the semaphore.
/// If `initial_count` is negative, greater than `maximum_count`, or if `maximum_count` is not greater than 0,
/// the function returns 0 (NULL) to indicate failure.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn CreateSemaphoreA(
    _security_attrs: usize,
    initial_count: i32,
    maximum_count: i32,
    _name: LPCSTR,
) -> Handle {
    if maximum_count <= 0 || initial_count < 0 || initial_count > maximum_count {
        return Handle::NULL;
    }

    let handle = common::sync::create_semaphore(initial_count, maximum_count);

    debug!(?handle, "CreateSemaphoreA");
    rine_types::dev_notify!(on_handle_created(
        handle.as_raw() as i64,
        "SemaphoreA",
        &format!("initial={initial_count}, max={maximum_count}")
    ));

    handle
}

/// Create a semaphore object with the specified initial and maximum count, and with an optional (UTF-16LE) name.
///
/// # Arguments
/// * `_security_attrs` - Currently ignored, as we do not implement any access control features.
/// * `initial_count` - The initial count for the semaphore. Must be non-negative and less than or equal to `maximum_count`.
/// * `maximum_count` - The maximum count for the semaphore. Must be greater than 0.
/// * `name` - Currently ignored, as named semaphores are not implemented, but it is still read and logged for dev notification purposes.
///
/// Returns a handle to the created semaphore, or 0 on failure (e.g. invalid parameters).
///
/// # Safety
/// The caller must ensure that `name` points to a valid null-terminated UTF-16LE string if it is not null.
/// The caller is responsible for managing the returned semaphore handle, including closing it when no longer needed.
///
/// # Returns
/// If the parameters are valid and the semaphore is successfully created, the function returns a handle to the semaphore.
/// If `initial_count` is negative, greater than `maximum_count`, or if `maximum_count` is not greater than 0,
/// the function returns 0 (NULL) to indicate failure.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn CreateSemaphoreW(
    _security_attrs: usize,
    initial_count: i32,
    maximum_count: i32,
    _name: LPCWSTR,
) -> Handle {
    if maximum_count <= 0 || initial_count < 0 || initial_count > maximum_count {
        return Handle::NULL;
    }

    let handle = common::sync::create_semaphore(initial_count, maximum_count);

    debug!(?handle, "CreateSemaphoreW");
    rine_types::dev_notify!(on_handle_created(
        handle.as_raw() as i64,
        "SemaphoreW",
        &format!("initial={initial_count}, max={maximum_count}")
    ));

    handle
}

/// Release a semaphore, incrementing its count by `release_count` and potentially unblocking waiters.
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
/// If `previous_count` is not null, the caller must ensure that it points to a valid writable
/// memory location where an i32 can be stored. Releasing a semaphore with an invalid handle,
/// or with parameters that would exceed the maximum count, will result in failure and return FALSE.
///
/// # Returns
/// If the semaphore is successfully released, the function returns `WinBool::TRUE` and any waiting threads
/// are unblocked according to the semaphore's behavior.
/// If the semaphore handle is invalid, the caller does not have appropriate access, or if releasing the
/// semaphore would exceed its maximum count, the function returns `WinBool::FALSE` and no action is taken.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn ReleaseSemaphore(
    semaphore_handle: Handle,
    release_count: i32,
    previous_count: *mut i32,
) -> WinBool {
    if release_count <= 0 {
        warn!(release_count, "ReleaseSemaphore: release_count must be > 0");
        return WinBool::FALSE;
    }

    unsafe { common::sync::release_semaphore(semaphore_handle, release_count, previous_count) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rine_types::handles::handle_table;
    use rine_types::sync::CriticalSection;
    use rine_types::threading::{WaitStatus, wait_on};

    use std::ptr;
    use std::sync::Arc;

    // ── Critical Section tests ───────────────────────────────────

    #[test]
    fn critical_section_init_enter_leave_delete() {
        let mut cs = CriticalSection::new();
        unsafe {
            InitializeCriticalSection(cs.as_mut_ptr());
            EnterCriticalSection(cs.as_mut_ptr());
            LeaveCriticalSection(cs.as_mut_ptr());
            DeleteCriticalSection(cs.as_mut_ptr());
        }
    }

    #[test]
    fn critical_section_recursive_entry() {
        let mut cs = CriticalSection::new();
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
        let mut cs = CriticalSection::new();
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
        let mut cs = CriticalSection::new();
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
            let h = CreateEventA(0, WinBool::TRUE, WinBool::FALSE, LPCSTR::NULL);
            assert_ne!(h, Handle::NULL);

            assert!(SetEvent(h).is_true());
            // Event is signaled, wait should succeed.
            let w = handle_table().get_waitable(h).unwrap();
            assert_eq!(wait_on(&w, 0), WaitStatus::WAIT_OBJECT_0.0);

            assert!(ResetEvent(h).is_true());
            // Event is now unsignaled.
            let w = handle_table().get_waitable(h).unwrap();
            assert_eq!(wait_on(&w, 0), WaitStatus::WAIT_TIMEOUT.0);
        }
    }

    #[test]
    fn create_event_w_initially_signaled() {
        unsafe {
            let h = CreateEventW(0, WinBool::FALSE, WinBool::TRUE, LPCWSTR::NULL);
            assert_ne!(h, Handle::NULL);
            let w = handle_table().get_waitable(h).unwrap();
            // Auto-reset, initially signaled — first wait succeeds, second times out.
            assert_eq!(wait_on(&w, 0), WaitStatus::WAIT_OBJECT_0.0);
            assert_eq!(wait_on(&w, 0), WaitStatus::WAIT_TIMEOUT.0);
        }
    }

    #[test]
    fn set_event_invalid_handle_returns_false() {
        unsafe {
            assert!(!SetEvent(Handle::from_raw(0xDEAD)).is_true());
            assert!(!ResetEvent(Handle::from_raw(0xDEAD)).is_true());
        }
    }

    // ── Mutex tests ──────────────────────────────────────────────

    #[test]
    fn create_mutex_unowned_and_wait() {
        unsafe {
            let h = CreateMutexA(0, WinBool::FALSE, LPCSTR::NULL);
            assert_ne!(h, Handle::NULL);

            let w = handle_table().get_waitable(h).unwrap();
            // Unowned mutex should be immediately acquirable.
            assert_eq!(wait_on(&w, 0), WaitStatus::WAIT_OBJECT_0.0);
        }
    }

    #[test]
    fn create_mutex_initially_owned() {
        unsafe {
            let h = CreateMutexA(0, WinBool::TRUE, LPCSTR::NULL);
            assert_ne!(h, Handle::NULL);

            // Same thread can recursively acquire.
            let w = handle_table().get_waitable(h).unwrap();
            assert_eq!(wait_on(&w, 0), WaitStatus::WAIT_OBJECT_0.0);
        }
    }

    #[test]
    fn create_mutex_w_variant_works() {
        unsafe {
            let h = CreateMutexW(0, WinBool::FALSE, LPCWSTR::NULL);
            assert_ne!(h, Handle::NULL);
        }
    }

    #[test]
    fn release_mutex_by_owner_succeeds() {
        unsafe {
            let h = CreateMutexA(0, WinBool::TRUE, LPCSTR::NULL);
            assert!(ReleaseMutex(h).is_true());
        }
    }

    #[test]
    fn release_mutex_not_owned_fails() {
        unsafe {
            // Create unowned mutex.
            let h = CreateMutexA(0, WinBool::FALSE, LPCSTR::NULL);
            // Nobody owns it, releasing should fail.
            assert!(!ReleaseMutex(h).is_true());
        }
    }

    #[test]
    fn release_mutex_invalid_handle_fails() {
        unsafe {
            assert!(!ReleaseMutex(Handle::from_raw(0xDEAD)).is_true());
        }
    }

    #[test]
    fn mutex_recursive_release() {
        unsafe {
            let h = CreateMutexA(0, WinBool::TRUE, LPCSTR::NULL);
            // Recursive acquire.
            let w = handle_table().get_waitable(h).unwrap();
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
            let h = CreateMutexA(0, WinBool::TRUE, LPCSTR::NULL);
            let released = Arc::new(AtomicBool::new(false));
            let released2 = Arc::clone(&released);

            // Spawn a thread that tries to acquire (should block/timeout).
            let child = std::thread::spawn(move || {
                let w = handle_table().get_waitable(h).unwrap();
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
            let h = CreateSemaphoreA(0, 2, 5, LPCSTR::NULL);
            assert_ne!(h, Handle::NULL);
        }
    }

    #[test]
    fn create_semaphore_w_variant_works() {
        unsafe {
            let h = CreateSemaphoreW(0, 1, 10, LPCWSTR::NULL);
            assert_ne!(h, Handle::NULL);
        }
    }

    #[test]
    fn create_semaphore_invalid_params_returns_null() {
        unsafe {
            // max_count <= 0
            assert_eq!(CreateSemaphoreA(0, 0, 0, LPCSTR::NULL), Handle::NULL);
            // initial_count < 0
            assert_eq!(CreateSemaphoreA(0, -1, 5, LPCSTR::NULL), Handle::NULL);
            // initial_count > max_count
            assert_eq!(CreateSemaphoreA(0, 6, 5, LPCSTR::NULL), Handle::NULL);
        }
    }

    #[test]
    fn semaphore_wait_and_release() {
        unsafe {
            let h = CreateSemaphoreA(0, 1, 5, LPCSTR::NULL);
            let w = handle_table().get_waitable(h).unwrap();

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
            let h = CreateSemaphoreA(0, 3, 5, LPCSTR::NULL);
            // Try to release 3, which would bring count to 6 > max 5.
            assert!(!ReleaseSemaphore(h, 3, ptr::null_mut()).is_true());
        }
    }

    #[test]
    fn release_semaphore_null_previous_count() {
        unsafe {
            let h = CreateSemaphoreA(0, 0, 5, LPCSTR::NULL);
            assert!(ReleaseSemaphore(h, 1, ptr::null_mut()).is_true());
        }
    }

    #[test]
    fn release_semaphore_zero_count_fails() {
        unsafe {
            let h = CreateSemaphoreA(0, 1, 5, LPCSTR::NULL);
            assert!(!ReleaseSemaphore(h, 0, ptr::null_mut()).is_true());
        }
    }

    #[test]
    fn release_semaphore_invalid_handle_fails() {
        unsafe {
            assert!(!ReleaseSemaphore(Handle::from_raw(0xDEAD), 1, ptr::null_mut()).is_true());
        }
    }

    #[test]
    fn semaphore_cross_thread_release_wakes_waiter() {
        unsafe {
            let h = CreateSemaphoreA(0, 0, 5, LPCSTR::NULL);

            let child = std::thread::spawn(move || {
                let w = handle_table().get_waitable(h).unwrap();
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
