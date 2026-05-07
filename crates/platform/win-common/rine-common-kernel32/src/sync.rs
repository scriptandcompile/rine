use std::ptr;
use std::sync::{Arc, Condvar, Mutex};

use rine_types::errors::BOOL;
use rine_types::handles::{Handle, HandleEntry, handle_table};
use rine_types::sync::{CriticalSection, LPCriticalSection};
use rine_types::threading::{
    EventInner, EventWaitable, MutexInner, MutexState, MutexWaitable, SemaphoreInner,
    SemaphoreWaitable, Waitable,
};

use tracing::warn;

/// Initialization for synchronization primitives like events and mutexes.
///
/// These are implemented using Rust's standard library synchronization types,
/// but wrapped in a way that allows them to be used as Windows synchronization
/// objects with the expected semantics.
///
/// For example, Windows events can be either manual-reset or auto-reset, and
/// this is handled by the `EventWaitable` type.
/// Mutexes track ownership and recursion count to support recursive locking by
/// the owning thread.
///
/// # Safety
///
/// The caller must ensure that the `cs` pointer is valid and points to a memory region
/// that can hold the necessary data for a critical section.
/// The caller must also ensure that the critical section is properly initialized before
/// use, and that it is not used after being deleted.
pub unsafe fn init_critical_section(cs: LPCriticalSection) {
    unsafe {
        *cs = CriticalSection::new();
    }
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
pub unsafe fn enter_critical_section(cs: LPCriticalSection) {
    if cs.is_null() {
        return;
    }

    unsafe {
        let mut mutex = (*cs).get_mutex();

        if mutex.is_null() {
            init_critical_section(cs);
            mutex = (*cs).get_mutex();
        }

        libc::pthread_mutex_lock(mutex);
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
/// Returns `BOOL::TRUE` if the lock was successfully acquired, or `BOOL::FALSE`
/// if the critical section is already owned by another thread or if an error occurred (e.g. invalid pointer).
pub unsafe fn try_enter_critical_section(cs: LPCriticalSection) -> BOOL {
    if cs.is_null() {
        return BOOL::FALSE;
    }

    unsafe {
        let mutex = (*cs).get_mutex();

        if mutex.is_null() {
            return BOOL::FALSE;
        }

        if libc::pthread_mutex_trylock(mutex) == 0 {
            BOOL::TRUE
        } else {
            BOOL::FALSE
        }
    }
}

/// Leave a critical section by unlocking the underlying mutex.
///
/// # Arguments
/// * `cs` - A pointer to the critical section to leave. Must have been initialized with `init_critical_section`.
///
/// # Safety
/// The caller must ensure that `cs` is a valid pointer to a critical section that has been properly initialized.
/// The caller must also ensure that the critical section is not used after being deleted.
/// If `cs` is null, this function does nothing and returns immediately.
///
/// # Returns
/// If the critical section was successfully left, the function returns `BOOL::TRUE`.
/// If the `cs` pointer is null, the function returns `BOOL::FALSE` and does not perform any operation.
///
/// # Notes
/// Missing implementation features:
/// - No Win32-accurate `GetLastError` mapping is provided for invalid-pointer
///   and unlock-error cases.
/// - Error handling does not map pthread failure codes to Win32 behavior.
pub unsafe fn leave_critical_section(cs: LPCriticalSection) {
    if cs.is_null() {
        return;
    }

    unsafe {
        let mutex = (*cs).get_mutex();

        if mutex.is_null() {
            return;
        }

        libc::pthread_mutex_unlock(mutex);
    }
}

/// Destroy and deallocate the mutex.
///
/// # Arguments
/// * `cs` - A pointer to the critical section to delete. Must have been initialized with `init_critical_section`.
///
/// # Safety
/// The caller must ensure that `cs` points to a valid CRITICAL_SECTION structure that has been
/// properly initialized and is not currently in use.
pub unsafe fn delete_critical_section(cs: LPCriticalSection) {
    if cs.is_null() {
        return;
    }

    unsafe {
        let mutex = (*cs).get_mutex();

        if mutex.is_null() {
            return;
        }

        libc::pthread_mutex_destroy(mutex);

        // Mutex was heap-allocated by CriticalSection::new(); free it.
        drop(Box::from_raw(mutex));

        // Clear the stored pointer in the CRITICAL_SECTION structure.
        ptr::write(cs as *mut *mut libc::pthread_mutex_t, core::ptr::null_mut());
    }
}

/// Release a mutex, decrementing its ownership count and potentially unblocking waiters.
///
/// # Arguments
/// * `handle` - A handle to the mutex to release. The caller must have ownership of
///   the mutex (i.e. have previously acquired it and not yet released it).
///
/// # Safety
/// The caller must ensure that `handle` is a valid handle to a mutex object that the caller currently owns.
/// Releasing a mutex that is not owned by the caller, or using an invalid handle, will result in failure and return `BOOL::FALSE`.
///
/// # Returns
/// If the mutex is successfully released, the function returns `BOOL::TRUE` and any waiting threads
/// are unblocked according to the mutex's behavior.
/// If the mutex handle is invalid or the caller does not have ownership of the mutex,
/// the function returns `BOOL::FALSE` and no action is taken.
#[inline]
pub unsafe fn release_mutex(handle: Handle) -> BOOL {
    let waitable = match handle_table().get_waitable(handle) {
        Some(Waitable::Mutex(m)) => m,
        _ => return BOOL::FALSE,
    };

    let thread_id = std::thread::current().id();
    let mut state = waitable.inner.state.lock().unwrap();

    if state.owner != Some(thread_id) {
        return BOOL::FALSE;
    }

    state.count -= 1;
    if state.count == 0 {
        state.owner = None;
        waitable.inner.condvar.notify_one();
    }

    BOOL::TRUE
}

/// Creates a synchronization event.
///
/// # Arguments
/// * `manual_reset` - If true, the event remains signaled until manually reset;
///   if false, it resets automatically after a single wait.
/// * `initial_state` - If true, the event starts in a signaled state;
///   if false, it starts non-signaled.
///
/// # Returns
/// A handle to the newly created event.
///
/// # Examples
/// ```
/// use rine_common_kernel32::sync::create_event;
/// use rine_types::errors::BOOL;
///
/// let manual_reset_event = create_event(BOOL::TRUE, BOOL::FALSE);
/// let auto_reset_event = create_event(BOOL::FALSE, BOOL::TRUE);
/// ```
pub fn create_event(manual_reset: BOOL, initial_state: BOOL) -> Handle {
    let waitable = EventWaitable {
        inner: Arc::new(EventInner {
            signaled: Mutex::new(initial_state.is_true()),
            condvar: Condvar::new(),
            manual_reset: manual_reset.is_true(),
        }),
    };
    handle_table().insert(HandleEntry::Event(waitable))
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
/// Setting an event with an invalid handle will result in failure and return `BOOL::FALSE`.
/// If the event is successfully set to the signaled state, the function returns `BOOL::TRUE`
/// and any waiting threads are released according to the event's reset mode (manual or auto).
pub fn set_event(event_handle: Handle) -> BOOL {
    let waitable = match handle_table().get_waitable(event_handle) {
        Some(Waitable::Event(e)) => e,
        _ => return BOOL::FALSE,
    };

    let mut signaled = waitable.inner.signaled.lock().unwrap();
    *signaled = true;

    if waitable.inner.manual_reset {
        waitable.inner.condvar.notify_all();
    } else {
        waitable.inner.condvar.notify_one();
    }

    BOOL::TRUE
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
/// Resetting an event with an invalid handle will result in failure and return `BOOL::FALSE`.
/// If the event is successfully reset to the non-signaled state, the function returns `BOOL::TRUE`
/// and any threads that wait on it will block until it is set again.
pub fn reset_event(event_handle: Handle) -> BOOL {
    let waitable = match handle_table().get_waitable(event_handle) {
        Some(Waitable::Event(e)) => e,
        _ => return BOOL::FALSE,
    };

    let mut signaled = waitable.inner.signaled.lock().unwrap();
    *signaled = false;

    BOOL::TRUE
}

/// Creates a named or unnamed mutex.
///
/// # Arguments
/// * `initial_owner` - If true, the calling thread becomes the initial owner of the mutex.
/// * `name` - Optional name for the mutex to allow cross-process synchronization.
///
/// # Returns
/// A tuple containing the mutex handle and a descriptive string of the mutex state.
///
/// # Examples
/// ```
/// use rine_common_kernel32::sync::create_mutex;
/// use rine_types::errors::BOOL;
///
/// let (unnamed_mutex, desc) = create_mutex(BOOL::FALSE, None);
/// assert_eq!(desc, "(unnamed)");
///
/// let (named_mutex, desc) = create_mutex(BOOL::TRUE, Some("MyMutex".to_string()));
/// assert_eq!(desc, "MyMutex (initially-owned)");
/// ```
pub fn create_mutex(initial_owner: BOOL, name: Option<String>) -> (Handle, String) {
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

    let detail = match (name.as_deref(), initial_owner.is_true()) {
        (Some(n), true) => format!("{} (initially-owned)", n),
        (Some(n), false) => n.to_owned(),
        (None, true) => "(unnamed, initially-owned)".to_owned(),
        (None, false) => "(unnamed)".to_owned(),
    };

    (h, detail)
}

/// Creates a semaphore with the specified initial and maximum counts.
///
/// # Arguments
/// * `initial_count` - The initial count for the semaphore. Must be non-negative and
///   less than or equal to `maximum_count`.
/// * `maximum_count` - The maximum count for the semaphore. Must be greater than 0.
///
/// # Returns
/// A handle to the newly created semaphore, or `Handle::NULL` if the parameters are
/// invalid.
///
/// # Examples
/// ```
/// use rine_common_kernel32::sync::create_semaphore;
/// use rine_types::handles::Handle;
///
/// let semaphore = create_semaphore(2, 5);
/// assert!(semaphore.is_valid());
///
/// let invalid_semaphore = create_semaphore(-1, 5);
/// assert_eq!(invalid_semaphore, Handle::NULL);
///
/// let invalid_semaphore = create_semaphore(3, 2);
/// assert_eq!(invalid_semaphore, Handle::NULL);
/// ```
pub fn create_semaphore(initial_count: i32, maximum_count: i32) -> Handle {
    if maximum_count <= 0 || initial_count < 0 || initial_count > maximum_count {
        warn!(
            initial_count,
            maximum_count, "CreateSemaphore: invalid parameters"
        );
        return Handle::NULL;
    }

    let waitable = SemaphoreWaitable {
        inner: Arc::new(SemaphoreInner {
            count: Mutex::new(initial_count),
            max_count: maximum_count,
            condvar: Condvar::new(),
        }),
    };

    handle_table().insert(HandleEntry::Semaphore(waitable))
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
/// If the semaphore is successfully released, the function returns `BOOL::TRUE` and any waiting threads
/// are unblocked according to the semaphore's behavior.
/// If the semaphore handle is invalid, the caller does not have appropriate access, or if releasing the
/// semaphore would exceed its maximum count, the function returns `BOOL::FALSE` and no action is taken.
pub unsafe fn release_semaphore(
    handle: Handle,
    release_count: i32,
    previous_count: *mut i32,
) -> BOOL {
    let waitable = match handle_table().get_waitable(handle) {
        Some(Waitable::Semaphore(s)) => s,
        _ => {
            warn!(handle = handle.as_raw(), "ReleaseSemaphore: invalid handle");
            return BOOL::FALSE;
        }
    };

    let mut current_count = waitable.inner.count.lock().unwrap();
    let prev = *current_count;

    if prev + release_count > waitable.inner.max_count {
        warn!(
            prev,
            release_count,
            max = waitable.inner.max_count,
            "ReleaseSemaphore: would exceed maximum count"
        );
        return BOOL::FALSE;
    }

    if !previous_count.is_null() {
        unsafe { ptr::write(previous_count, prev) };
    }

    *current_count = prev + release_count;

    // Wake up to `release_count` waiters.
    for _ in 0..release_count {
        waitable.inner.condvar.notify_one();
    }

    BOOL::TRUE
}

#[cfg(test)]
mod tests {
    use super::*;
    use rine_types::errors::BOOL;

    #[test]
    fn create_event_auto_reset() {
        let h = create_event(BOOL::FALSE, BOOL::FALSE);
        assert!(h.is_valid());
    }

    #[test]
    fn create_event_manual_reset() {
        let h = create_event(BOOL::TRUE, BOOL::FALSE);
        assert!(h.is_valid());
    }

    #[test]
    fn create_event_initial_signaled() {
        let h = create_event(BOOL::FALSE, BOOL::TRUE);
        assert!(h.is_valid());
    }

    #[test]
    fn create_event_initial_not_signaled() {
        let h = create_event(BOOL::FALSE, BOOL::FALSE);
        assert!(h.is_valid());
    }

    #[test]
    fn create_event_different_parameters() {
        let h1 = create_event(BOOL::FALSE, BOOL::FALSE);
        let h2 = create_event(BOOL::FALSE, BOOL::FALSE);
        assert!(h1.is_valid());
        assert!(h2.is_valid());
    }

    #[test]
    fn create_event_manual_vs_auto() {
        let h1 = create_event(BOOL::FALSE, BOOL::FALSE);
        let h2 = create_event(BOOL::TRUE, BOOL::FALSE);
        assert!(h1.is_valid());
        assert!(h2.is_valid());
    }

    #[test]
    fn create_mutex_unnamed_auto() {
        let (h, desc) = create_mutex(BOOL::FALSE, None);
        assert!(h.is_valid());
        assert!(desc.contains("(unnamed)"));
    }

    #[test]
    fn create_mutex_unnamed_initial_owned() {
        let (h, desc) = create_mutex(BOOL::TRUE, None);
        assert!(h.is_valid());
        assert!(desc.contains("(unnamed") && desc.contains("initially-owned"));
    }

    #[test]
    fn create_mutex_named() {
        let (h, desc) = create_mutex(BOOL::FALSE, Some("TestMutex".to_string()));
        assert!(h.is_valid());
        assert_eq!(desc, "TestMutex");
    }

    #[test]
    fn create_mutex_named_initial_owned() {
        let (h, desc) = create_mutex(BOOL::TRUE, Some("OwnedMutex".to_string()));
        assert!(h.is_valid());
        assert_eq!(desc, "OwnedMutex (initially-owned)");
    }

    #[test]
    fn create_mutex_different_parameters() {
        let (h1, desc1) = create_mutex(BOOL::FALSE, None);
        let (h2, desc2) = create_mutex(BOOL::FALSE, Some("Test".to_string()));
        assert!(h1.is_valid());
        assert!(h2.is_valid());
        assert_eq!(desc1, "(unnamed)");
        assert_eq!(desc2, "Test");
    }

    #[test]
    fn create_mutex_initial_state() {
        let (h, desc) = create_mutex(BOOL::TRUE, Some("Test".to_string()));
        assert!(h.is_valid());
        assert!(desc.contains("initially-owned"));

        let (h2, desc2) = create_mutex(BOOL::FALSE, Some("Test2".to_string()));
        assert!(h2.is_valid());
        assert!(!desc2.contains("initially-owned"));
    }

    #[test]
    fn create_mutex_multiple_instances() {
        let mut handles = Vec::new();
        let mut descs = Vec::new();

        for i in 0..5 {
            let (h, desc) = create_mutex(BOOL::FALSE, Some(format!("Mutex{}", i)));
            handles.push(h);
            descs.push(desc);
        }

        for h in &handles {
            assert!(h.is_valid());
        }

        assert_eq!(descs[0], "Mutex0");
        assert_eq!(descs[1], "Mutex1");
        assert_eq!(descs[2], "Mutex2");
        assert_eq!(descs[3], "Mutex3");
        assert_eq!(descs[4], "Mutex4");
    }

    #[test]
    fn create_semaphore_valid() {
        let h = create_semaphore(2, 5);
        assert!(h != Handle::NULL);
    }

    #[test]
    fn create_semaphore_invalid_initial_count() {
        let h = create_semaphore(-1, 5);
        assert_eq!(h, Handle::NULL);
    }

    #[test]
    fn create_semaphore_invalid_maximum_count() {
        let h = create_semaphore(3, 2);
        assert_eq!(h, Handle::NULL);
    }

    #[test]
    fn create_semaphore_zero_initial() {
        let h = create_semaphore(0, 5);
        assert!(h != Handle::NULL);
    }

    #[test]
    fn create_semaphore_initial_equals_maximum() {
        let h = create_semaphore(5, 5);
        assert!(h != Handle::NULL);
    }

    #[test]
    fn create_semaphore_multiple_instances() {
        let h1 = create_semaphore(1, 3);
        let h2 = create_semaphore(2, 4);
        assert!(h1 != Handle::NULL);
        assert!(h2 != Handle::NULL);
    }
}
