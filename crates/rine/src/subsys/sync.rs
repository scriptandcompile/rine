//! Synchronization primitives subsystem.
//!
//! This module ties together the sync-object types from [`rine_types::threading`]
//! (events, mutexes, semaphores) with the kernel32 API surface implemented in
//! the `rine64-kernel32` crate.
//!
//! ## Mapping
//!
//! | Windows API                     | Backing mechanism            |
//! |---------------------------------|------------------------------|
//! | `CreateEvent` / `SetEvent` …    | `Mutex<bool>` + `Condvar`    |
//! | `CreateMutex` / `ReleaseMutex`  | `Mutex<MutexState>` + `Condvar` (recursive, thread-owned) |
//! | `CreateSemaphore` / `ReleaseSemaphore` | `Mutex<i32>` + `Condvar` |
//! | `InitializeCriticalSection` …   | `pthread_mutex_t` (recursive) |
//!
//! All waitable objects (events, mutexes, semaphores) integrate with
//! `WaitForSingleObject` / `WaitForMultipleObjects` through the
//! [`Waitable`](rine_types::threading::Waitable) enum.
