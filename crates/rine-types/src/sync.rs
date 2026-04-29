//! FFI-friendly synchronization types shared across crates.

use core::mem;
use core::ptr;

/// A platform abstraction for a Windows-like `CRITICAL_SECTION`.
///
/// On Unix we store a `pthread_mutex_t` followed by padding so the
/// total storage remains 40 bytes (matching existing consumers that treat
/// the CS area as 40 bytes of storage).
#[repr(C)]
pub struct CriticalSection {
    pub mutex: *mut libc::pthread_mutex_t,
    _pad: [u8; 40 - mem::size_of::<*mut libc::pthread_mutex_t>()],
}

/// Pointer to a mutable `CriticalSection` (LPCRITICAL_SECTION equivalent).
pub type LPCriticalSection = *mut CriticalSection;

/// Pointer to an immutable `CriticalSection` (LPCCRITICAL_SECTION equivalent).
pub type LPCCriticalSection = *const CriticalSection;

impl CriticalSection {
    /// Creates a new `CriticalSection` with default attributes.
    pub fn new() -> Self {
        unsafe {
            let mut attr: libc::pthread_mutexattr_t = core::mem::zeroed();
            let mutex = Box::into_raw(Box::new(core::mem::zeroed::<libc::pthread_mutex_t>()));

            libc::pthread_mutexattr_init(&mut attr);
            libc::pthread_mutexattr_settype(&mut attr, libc::PTHREAD_MUTEX_RECURSIVE);
            libc::pthread_mutex_init(mutex, &attr);
            libc::pthread_mutexattr_destroy(&mut attr);

            CriticalSection {
                mutex,
                _pad: [0; 40 - mem::size_of::<*mut libc::pthread_mutex_t>()],
            }
        }
    }

    /// Retrieves a pointer to the underlying `pthread_mutex_t` from the `CriticalSection`.
    ///
    /// # Safety
    /// The caller must ensure that the `CriticalSection` has been properly initialized and not dropped.
    /// The returned pointer is valid as long as the `CriticalSection` is alive and has not been moved or modified.
    /// The caller is responsible for ensuring that the mutex is used in a thread-safe manner according to the expected semantics of a critical section.
    ///
    /// # Returns
    /// A pointer to the underlying `pthread_mutex_t` that can be used for locking and unlocking operations.
    pub fn get_mutex(&self) -> *mut libc::pthread_mutex_t {
        self.mutex
    }

    /// Retrieves a mutable pointer to the underlying `pthread_mutex_t` from the `CriticalSection`.
    ///
    /// # Safety
    /// The caller must ensure that the `CriticalSection` has been properly initialized and not dropped.
    /// The returned pointer is valid as long as the `CriticalSection` is alive and has not been moved or modified.
    /// The caller is responsible for ensuring that the mutex is used in a thread-safe manner according to the expected semantics of a critical section.
    ///
    /// # Returns
    /// A mutable pointer to the underlying `pthread_mutex_t` that can be used for locking and unlocking operations.
    pub fn as_mut_ptr(&mut self) -> LPCriticalSection {
        self as LPCriticalSection
    }
}

impl Default for CriticalSection {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for CriticalSection {
    fn drop(&mut self) {
        unsafe {
            let mutex = self.get_mutex();

            if !mutex.is_null() {
                libc::pthread_mutex_destroy(mutex);
                drop(Box::from_raw(mutex));
                // mark pointer cleared
                ptr::write(&mut self.mutex, core::ptr::null_mut());
            }
        }
    }
}
