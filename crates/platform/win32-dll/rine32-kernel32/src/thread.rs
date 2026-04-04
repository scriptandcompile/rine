use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use rine_types::errors::WinBool;
use rine_types::handles::{Handle, HandleEntry, INVALID_HANDLE_VALUE, handle_table};
use rine_types::threading::{self, TLS_OUT_OF_INDEXES};

static NEXT_THREAD_ID: AtomicU32 = AtomicU32::new(1000);

type ThreadStartRoutine = unsafe extern "stdcall" fn(usize) -> u32;

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn TlsAlloc() -> u32 {
    threading::tls_alloc().unwrap_or(TLS_OUT_OF_INDEXES)
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn TlsFree(tls_index: u32) -> WinBool {
    if threading::tls_free(tls_index) {
        WinBool::TRUE
    } else {
        WinBool::FALSE
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn TlsGetValue(tls_index: u32) -> usize {
    threading::tls_get_value(tls_index)
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn TlsSetValue(tls_index: u32, value: usize) -> WinBool {
    if threading::tls_set_value(tls_index, value) {
        WinBool::TRUE
    } else {
        WinBool::FALSE
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn Sleep(milliseconds: u32) {
    std::thread::sleep(Duration::from_millis(milliseconds as u64));
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn CreateThread(
    _security_attrs: usize,
    _stack_size: usize,
    start_address: usize,
    parameter: usize,
    _creation_flags: u32,
    thread_id_out: *mut u32,
) -> isize {
    if start_address == 0 {
        return INVALID_HANDLE_VALUE.as_raw();
    }

    let exit_code = Arc::new(AtomicU32::new(threading::STILL_ACTIVE));
    let completed = Arc::new((Mutex::new(false), Condvar::new()));
    let tid = NEXT_THREAD_ID.fetch_add(1, Ordering::Relaxed);

    let waitable = threading::ThreadWaitable {
        exit_code: Arc::clone(&exit_code),
        completed: Arc::clone(&completed),
    };
    let h = handle_table().insert(HandleEntry::Thread(waitable));

    if !thread_id_out.is_null() {
        unsafe { std::ptr::write(thread_id_out, tid) };
    }

    let result = std::thread::Builder::new().spawn(move || {
        let start_fn: ThreadStartRoutine = unsafe { core::mem::transmute(start_address) };
        let code = unsafe { start_fn(parameter) };
        exit_code.store(code, Ordering::Release);
        let (lock, cvar) = &*completed;
        *lock.lock().unwrap() = true;
        cvar.notify_all();
    });

    match result {
        Ok(join_handle) => {
            drop(join_handle);
            h.as_raw()
        }
        Err(_) => {
            handle_table().remove(h);
            INVALID_HANDLE_VALUE.as_raw()
        }
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn GetCurrentThread() -> isize {
    -2
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn GetCurrentThreadId() -> u32 {
    unsafe { libc::syscall(libc::SYS_gettid) as u32 }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn GetExitCodeThread(
    thread_handle: isize,
    exit_code_out: *mut u32,
) -> WinBool {
    if exit_code_out.is_null() {
        return WinBool::FALSE;
    }
    let h = Handle::from_raw(thread_handle);
    match handle_table().get_thread_exit_code(h) {
        Some(code) => {
            unsafe { std::ptr::write(exit_code_out, code) };
            WinBool::TRUE
        }
        None => WinBool::FALSE,
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn WaitForSingleObject(handle: isize, timeout_ms: u32) -> u32 {
    let h = Handle::from_raw(handle);
    match handle_table().get_waitable(h) {
        Some(waitable) => threading::wait_on(&waitable, timeout_ms),
        None => threading::WaitStatus::WAIT_FAILED.0,
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn WaitForMultipleObjects(
    count: u32,
    handles_ptr: *const isize,
    wait_all: WinBool,
    timeout_ms: u32,
) -> u32 {
    if handles_ptr.is_null() || count == 0 || count > 64 {
        return threading::WaitStatus::WAIT_FAILED.0;
    }

    let raw_handles: Vec<isize> = (0..count as usize)
        .map(|i| unsafe { *handles_ptr.add(i) })
        .collect();

    let waitables: Vec<Option<threading::Waitable>> = raw_handles
        .iter()
        .map(|&raw| handle_table().get_waitable(Handle::from_raw(raw)))
        .collect();

    if waitables.iter().any(|w| w.is_none()) {
        return threading::WaitStatus::WAIT_FAILED.0;
    }
    let waitables: Vec<threading::Waitable> = waitables.into_iter().flatten().collect();

    if wait_all.is_true() {
        let start = std::time::Instant::now();
        for w in &waitables {
            let remaining = if timeout_ms == threading::INFINITE {
                threading::INFINITE
            } else {
                let elapsed = start.elapsed().as_millis() as u32;
                if elapsed >= timeout_ms {
                    return threading::WaitStatus::WAIT_TIMEOUT.0;
                }
                timeout_ms - elapsed
            };
            let result = threading::wait_on(w, remaining);
            if result != threading::WaitStatus::WAIT_OBJECT_0.0 {
                return result;
            }
        }
        threading::WaitStatus::WAIT_OBJECT_0.0
    } else {
        let start = std::time::Instant::now();
        loop {
            for (i, w) in waitables.iter().enumerate() {
                if threading::wait_on(w, 0) == threading::WaitStatus::WAIT_OBJECT_0.0 {
                    return threading::WaitStatus(
                        threading::WaitStatus::WAIT_OBJECT_0.0 + i as u32,
                    )
                    .0;
                }
            }
            if timeout_ms != threading::INFINITE {
                let elapsed = start.elapsed().as_millis() as u32;
                if elapsed >= timeout_ms {
                    return threading::WaitStatus::WAIT_TIMEOUT.0;
                }
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}
