use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use rine_types::errors::WinBool;
use rine_types::handles::{Handle, HandleEntry, handle_table};
use rine_types::threading::{
    self, INFINITE, STILL_ACTIVE, TLS_OUT_OF_INDEXES, ThreadWaitable, WaitStatus, Waitable,
};
use tracing::{debug, warn};

const CREATE_SUSPENDED: u32 = 0x04;
static NEXT_THREAD_ID: AtomicU32 = AtomicU32::new(1000);

/// Create and start a new thread.
///
/// # Arguments
/// * `start_address`: Function pointer to the thread entry point.
///   calling convention and return a `DWORD` exit code.
/// * `parameter`: Argument passed to the thread entry point.
/// * `creation_flags`: Creation flags, currently ignored except for `CREATE_SUSPENDED` which is not supported but
///   will prevent a warning.
/// * `thread_id_out`: Optional pointer to receive the new thread's ID.
/// * `run_start`: Closure that runs the thread entry point and returns the exit code.
///   This is where we set up the child thread's TEB before calling the entry point.
///
/// # Safety
/// The caller must ensure that `start_address` is a valid function pointer with the correct signature and calling convention.
/// The caller must not pass invalid or non-zero values for `_security_attrs` and `_stack_size`.
/// The caller must ensure that `thread_id_out` is either null or a valid pointer to a `u32` variable.
/// The caller is responsible for closing the returned thread handle when it is no longer needed.
///
/// # Returns
/// A handle to the new thread, or `INVALID_HANDLE_VALUE` on failure.
/// The caller can use this handle with other synchronization functions and must close it with `CloseHandle` when done.
///
/// # Notes
/// Missing implementation features:
/// - `CREATE_SUSPENDED` is not supported; the thread starts immediately.
/// - `_security_attrs` and `_stack_size` semantics are ignored.
/// - Most creation flags beyond basic launch are not implemented.
/// - No Win32-accurate `GetLastError` mapping is provided for failure paths.
pub fn create_thread<F>(
    start_address: usize,
    parameter: usize,
    creation_flags: u32,
    thread_id_out: Option<&mut u32>,
    run_start: F,
) -> Handle
where
    F: FnOnce(usize, usize) -> u32 + Send + 'static,
{
    if start_address == 0 {
        warn!("CreateThread: null start address");
        return Handle::NULL;
    }

    if creation_flags & CREATE_SUSPENDED != 0 {
        warn!("CreateThread: CREATE_SUSPENDED not yet supported; starting immediately");
    }

    let exit_code = Arc::new(AtomicU32::new(STILL_ACTIVE));
    let completed = Arc::new((Mutex::new(false), Condvar::new()));
    let tid = NEXT_THREAD_ID.fetch_add(1, Ordering::Relaxed);

    let waitable = ThreadWaitable {
        exit_code: Arc::clone(&exit_code),
        completed: Arc::clone(&completed),
    };
    let h = handle_table().insert(HandleEntry::Thread(waitable));

    debug!(?h, tid, "created thread");
    rine_types::dev_notify!(on_handle_created(
        h.as_raw() as i64,
        "Thread",
        &format!("tid={tid}")
    ));
    rine_types::dev_notify!(on_thread_created(
        h.as_raw() as i64,
        tid,
        start_address as u64
    ));

    if let Some(thread_id_out) = thread_id_out {
        *thread_id_out = tid;
    }

    let result = std::thread::Builder::new().spawn(move || {
        let code = run_start(start_address, parameter);
        exit_code.store(code, Ordering::Release);

        // Notify before waking waiters so the event is not lost on process exit.
        rine_types::dev_notify!(on_thread_exited(tid, code));

        let (lock, cvar) = &*completed;
        *lock.lock().unwrap() = true;
        cvar.notify_all();

        debug!(exit_code = code, "child thread exited");
    });

    match result {
        Ok(join_handle) => {
            drop(join_handle);
            h
        }
        Err(e) => {
            warn!("CreateThread: spawn failed: {e}");
            handle_table().remove(h);
            Handle::NULL
        }
    }
}

/// Allocate a TLS index.
///
/// # Safety
/// The caller is responsible for calling `TlsFree` to release the index when it is no longer needed.
///
/// # Returns
/// A TLS index, or `TLS_OUT_OF_INDEXES` (0xFFFFFFFF) on failure.
pub fn tls_alloc() -> u32 {
    match threading::tls_alloc() {
        Some(idx) => {
            debug!(index = idx, "TlsAlloc");
            rine_types::dev_notify!(on_tls_allocated(idx));
            idx
        }
        None => {
            warn!("TlsAlloc: no free TLS slots");
            TLS_OUT_OF_INDEXES
        }
    }
}

/// Free a TLS index.
///
/// # Arguments
/// * `tls_index`: The TLS index to free, which must have been previously allocated by `TlsAlloc` and not already freed.
///
/// # Safety
/// The caller must ensure that `tls_index` is a valid index previously allocated by `TlsAlloc` and not already freed.
/// The caller is responsible for ensuring that no threads are currently using the TLS index before freeing it.
///
/// # Returns
/// `WinBool::TRUE` on success, `WinBool::FALSE` on failure (e.g., invalid index).
pub fn tls_free(tls_index: u32) -> WinBool {
    if threading::tls_free(tls_index) {
        rine_types::dev_notify!(on_tls_freed(tls_index));
        WinBool::TRUE
    } else {
        WinBool::FALSE
    }
}

/// Get the current thread's value for a TLS slot.
///
/// # Arguments
/// * `tls_index`: The TLS index to query, which must have been previously allocated by `TlsAlloc` and not already freed.
///
/// # Safety
/// The caller must ensure that `tls_index` is a valid index previously allocated by `TlsAlloc` and not already freed.
/// The caller is responsible for ensuring that the returned value is interpreted correctly based on how it was set with `TlsSetValue`.
///
/// # Returns
/// The value associated with the TLS index for the current thread, or 0 if the index is invalid or has not been set.
///
/// # Note
/// A 0 can also be a valid value set by `TlsSetValue`, so the caller should use `GetLastError` to distinguish between an
/// error and a valid 0 value if needed.
/// Currently, we do not set `GetLastError` to provide more information.
pub fn tls_get_value(tls_index: u32) -> usize {
    threading::tls_get_value(tls_index)
}

/// Set the current thread's value for a slot.
///
/// # Arguments
/// * `tls_index`: The TLS index to set, which must have been previously allocated by `TlsAlloc` and not already freed.
/// * `value`: The value to associate with the TLS index for the current thread. This can be any `usize` value, including 0.
///
/// # Safety
/// The caller must ensure that `tls_index` is a valid index previously allocated by `TlsAlloc` and not already freed.
/// The caller is responsible for ensuring that the value is interpreted correctly based on how it will be used when
/// retrieved with `TlsGetValue`.
/// The caller should also ensure that any necessary synchronization is performed when setting and getting TLS values
/// across threads, as appropriate for the application's logic.
///
/// # Returns
/// `WinBool::TRUE` on success, `WinBool::FALSE` on failure (e.g., invalid index).
///
/// # Notes
/// Missing implementation features:
/// - No Win32-accurate `GetLastError` mapping is provided for invalid TLS index failures.
pub fn tls_set_value(tls_index: u32, value: usize) -> WinBool {
    if threading::tls_set_value(tls_index, value) {
        WinBool::TRUE
    } else {
        WinBool::FALSE
    }
}

/// Cause the current thread to sleep for the specified duration.
///
/// # Arguments
/// * `duration`: The duration to sleep for.
///
/// # Safety
/// The caller is responsible for ensuring that sleeping is appropriate in the current context
/// (e.g., not holding locks that would cause deadlocks).
/// The caller should also be aware that sleeping does not guarantee precise timing and may be affected by system scheduling and load.
pub fn sleep(duration: Duration) {
    std::thread::sleep(duration);
}

/// Get a pseudo-handle for the calling thread.
///
/// # Safety
/// This is not a real handle and cannot be used with all handle functions, but it can be used with certain functions that
/// specifically support it (e.g., `GetExitCodeThread`).
/// The caller should not attempt to close this handle or use it in contexts that require a real handle.
///
/// # Returns
/// A pseudo-handle value that represents the current thread.
/// This value is not a real handle and should only be used with functions that explicitly support it.
///
/// # Notes
/// Missing implementation features:
/// - The pseudo-handle is not currently mapped to a concrete thread entry in
///   the internal handle table.
/// - APIs expecting a queryable thread handle may still reject this pseudo-
///   handle instead of treating it as `GetCurrentThread()`.
pub fn current_thread() -> isize {
    -2
}

/// Get the current thread's ID.
///
/// # Safety
/// This is a unique identifier for the thread that can be used with certain APIs and for debugging purposes.
/// It is not a handle and cannot be used with handle-based APIs.
/// The thread ID is assigned by our implementation and is not guaranteed to match any OS-level thread ID,
/// but it is unique within the process and can be used to identify threads in logs and debugging tools.
///
/// # Returns
/// The current thread's ID as a `u32`.
pub fn current_thread_id() -> u32 {
    // Linux tid mapped to DWORD.
    unsafe { libc::syscall(libc::SYS_gettid) as u32 }
}

/// Get the exit code of a thread.
///
/// # Arguments
/// * `handle`: A handle to the thread, which can be a real thread handle or the pseudo-handle returned by `current_thread()`.
/// * `exit_code_out`: Optional pointer to receive the thread's exit code.
///   If the thread is still active, this will be set to `STILL_ACTIVE` (259).
///
/// # Safety
/// The caller must ensure that `thread_handle` is a valid thread handle or the pseudo-handle for the current thread.
/// The caller must also ensure that `exit_code_out` is either null or a valid pointer to a `u32` variable.
/// The caller is responsible for interpreting the returned exit code correctly, especially if the thread is still
/// active (in which case it will be set to `STILL_ACTIVE`).
///
/// # Returns
/// `WinBool::TRUE` on success, with the thread's exit code written to `exit_code_out` if it is not null.
/// `WinBool::FALSE` on failure (e.g., invalid handle).
///
/// # Notes
/// Missing implementation features:
/// - No Win32-accurate `GetLastError` mapping is provided for invalid-handle
///   or null-output-pointer failures.
/// - No explicit access-right checks are enforced against per-handle granted
///   permissions.
/// - Pseudo-handle semantics (`GetCurrentThread`) are not normalized here.
pub fn get_exit_code_thread(handle: Handle, exit_code_out: Option<&mut u32>) -> WinBool {
    let Some(exit_code_out) = exit_code_out else {
        return WinBool::FALSE;
    };

    match handle_table().get_thread_exit_code(handle) {
        Some(code) => {
            *exit_code_out = code;
            WinBool::TRUE
        }
        None => WinBool::FALSE,
    }
}

/// Block the current thread until the specified handle is signalled or the timeout elapses.
///
/// # Arguments
/// * `handle`: A handle to wait on, which can be a thread handle, process handle, or synchronization object handle.
/// * `duration`: The timeout duration to wait, or `INFINITE` (0xFFFFFFFF) to wait indefinitely.
///
/// # Safety
/// The caller should be aware that the actual wait time may be affected by system scheduling and load, and that the
/// function may return before the timeout elapses if the handle is signalled.
///
/// # Returns
/// `WAIT_OBJECT_0` if the handle was signalled, `WAIT_TIMEOUT` if the timeout elapsed, or `WAIT_FAILED` on error.
pub fn wait_for_single_object(handle: Handle, duration: Duration) -> u32 {
    match handle_table().get_waitable(handle) {
        Some(waitable) => {
            let timeout_ms = if duration == Duration::from_millis(INFINITE as u64) {
                INFINITE
            } else {
                duration.as_millis() as u32
            };
            threading::wait_on(&waitable, timeout_ms)
        }
        None => {
            warn!("{:?} WaitForSingleObject: invalid handle", handle.as_raw());
            WaitStatus::WAIT_FAILED.0
        }
    }
}

/// Block the current thread until one or all of the specified handles are signalled, or the timeout elapses.
///
/// # Arguments
/// * `handles`: A slice of handles to wait on, which can be thread handles, process handles,
///   or synchronization object handles.
/// * `wait_all`: If `WinBool::TRUE`, the function returns when all handles are signalled;
///   if `WinBool::FALSE`, it returns when any one handle is signalled.
/// * `duration`: The timeout duration to wait, or `INFINITE` (0xFFFFFFFF) to wait indefinitely.
///
/// # Safety
/// The caller must ensure that `handles_ptr` points to a valid array of `count` handles, and that each handle
/// is valid and of a type that can be waited on.
/// The caller should also be aware that the actual wait time may be affected by system scheduling and load,
/// and that the function may return before the timeout elapses if the specified condition is met (e.g., a handle is signalled).
///
/// # Returns
/// If `wait_all` is `WinBool::FALSE`, returns `WAIT_OBJECT_0 + i` if the handle at index `i` is signalled,
/// `WAIT_TIMEOUT` if the timeout elapsed, or `WAIT_FAILED` on error.
/// If `wait_all` is `WinBool::TRUE`, returns `WAIT_OBJECT_0` if all handles are signalled,
/// `WAIT_TIMEOUT` if the timeout elapsed, or `WAIT_FAILED` on error.
pub fn wait_for_multiple_objects(handles: &[Handle], wait_all: bool, duration: Duration) -> u32 {
    if handles.is_empty() || handles.len() > 64 {
        return WaitStatus::WAIT_FAILED.0;
    }

    let waitables: Vec<Option<Waitable>> = handles
        .iter()
        .map(|&handle| handle_table().get_waitable(handle))
        .collect();

    if waitables.iter().any(|w| w.is_none()) {
        warn!("WaitForMultipleObjects: one or more invalid handles");
        return WaitStatus::WAIT_FAILED.0;
    }
    let waitables: Vec<Waitable> = waitables.into_iter().flatten().collect();

    if wait_all {
        wait_for_all(&waitables, duration)
    } else {
        wait_for_any(&waitables, duration)
    }
}

/// Wait for all handles to be signalled or the timeout to elapse.
///
/// # Arguments
/// * `waitables`: A slice of waitable objects to wait on.
/// * `duration`: The timeout duration to wait, or `INFINITE` (0xFFFFFFFF) to wait indefinitely.
///
/// # Safety
/// The caller should be aware that the actual wait time may be affected by system scheduling and load,
/// and that the function may return before the timeout elapses if all handles are signalled.
/// The caller should also ensure that the waitables are not modified concurrently while waiting.
///
/// # Returns
/// `WAIT_OBJECT_0` if all waitables were signalled, `WAIT_TIMEOUT` if the timeout elapsed, or `WAIT_FAILED` on error.
fn wait_for_all(waitables: &[Waitable], duration: Duration) -> u32 {
    let start = Instant::now();
    for w in waitables {
        let remaining = if duration == Duration::from_millis(INFINITE as u64) {
            INFINITE
        } else {
            let elapsed = start.elapsed().as_millis() as u32;
            if elapsed >= duration.as_millis() as u32 {
                return WaitStatus::WAIT_TIMEOUT.0;
            }
            duration.as_millis() as u32 - elapsed
        };

        let result = threading::wait_on(w, remaining);
        if result != WaitStatus::WAIT_OBJECT_0.0 {
            return result;
        }
    }

    WaitStatus::WAIT_OBJECT_0.0
}

/// Wait for any handle to be signalled or the timeout to elapse.
///
/// # Arguments
/// * `waitables`: A slice of waitable objects to wait on.
/// * `duration`: The timeout duration to wait, or `INFINITE` (0xFFFFFFFF) to wait indefinitely.
///
/// # Safety
/// The caller should be aware that the actual wait time may be affected by system scheduling and load,
/// and that the function may return before the timeout elapses if any handle is signalled.
/// The caller should also ensure that the waitables are not modified concurrently while waiting.
///
/// # Returns
/// Returns `WAIT_OBJECT_0 + i` if the handle at index `i` is signalled, `WAIT_TIMEOUT` if the timeout elapsed, or `WAIT_FAILED` on error.
fn wait_for_any(waitables: &[Waitable], duration: Duration) -> u32 {
    let start = Instant::now();
    loop {
        for (i, w) in waitables.iter().enumerate() {
            if threading::wait_on(w, 0) == WaitStatus::WAIT_OBJECT_0.0 {
                return WaitStatus(WaitStatus::WAIT_OBJECT_0.0 + i as u32).0;
            }
        }

        if duration != Duration::from_millis(INFINITE as u64) {
            let elapsed = start.elapsed().as_millis() as u32;
            if elapsed >= duration.as_millis() as u32 {
                return WaitStatus::WAIT_TIMEOUT.0;
            }
        }

        std::thread::sleep(Duration::from_millis(1));
    }
}
