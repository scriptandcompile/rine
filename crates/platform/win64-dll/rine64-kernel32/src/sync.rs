#![allow(non_snake_case)]
//! kernel32 synchronisation: critical sections (stub/no-op for single-threaded Phase 1).

/// InitializeCriticalSection — no-op stub.
///
/// # Safety
/// `cs` must be a valid pointer to a CRITICAL_SECTION-sized region.
pub unsafe extern "win64" fn InitializeCriticalSection(cs: *mut u8) {
    if !cs.is_null() {
        // Zero out the critical section struct (enough to keep the CRT happy).
        unsafe { core::ptr::write_bytes(cs, 0, 40) }; // sizeof(CRITICAL_SECTION) = 40
    }
}

/// EnterCriticalSection — no-op (single-threaded).
pub unsafe extern "win64" fn EnterCriticalSection(_cs: *mut u8) {}

/// LeaveCriticalSection — no-op (single-threaded).
pub unsafe extern "win64" fn LeaveCriticalSection(_cs: *mut u8) {}

/// DeleteCriticalSection — no-op.
pub unsafe extern "win64" fn DeleteCriticalSection(_cs: *mut u8) {}
