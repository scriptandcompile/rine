//! kernel32 threading: CreateThread, TLS, Wait*, GetCurrentThread, Sleep.

use std::alloc::{Layout, alloc_zeroed};
use std::ptr;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Condvar, Mutex};

use rine_types::errors::WinBool;
use rine_types::handles::{Handle, HandleEntry, INVALID_HANDLE_VALUE, handle_table};
use rine_types::threading::{
    self, INFINITE, STILL_ACTIVE, TLS_OUT_OF_INDEXES, ThreadWaitable, WaitStatus, Waitable,
};
use tracing::{debug, warn};

// ── TEB setup for child threads ──────────────────────────────────

const TEB_SIZE: usize = 0x1000;
const PEB_SIZE: usize = 0x1000;
const TIB_STACK_BASE: usize = 0x08;
const TIB_STACK_LIMIT: usize = 0x10;
const TEB_SELF: usize = 0x30;
const TEB_PEB: usize = 0x60;
const ARCH_SET_GS: i32 = 0x1001;

/// Allocate and install a TEB for the current (child) thread.
///
/// # Safety
/// Must be called exactly once per thread, before any PE code runs.
unsafe fn setup_child_teb() {
    let layout = Layout::from_size_align(TEB_SIZE, 16).unwrap();
    let teb = unsafe { alloc_zeroed(layout) };
    assert!(!teb.is_null(), "failed to allocate child TEB");

    let peb_layout = Layout::from_size_align(PEB_SIZE, 16).unwrap();
    let peb = unsafe { alloc_zeroed(peb_layout) };
    assert!(!peb.is_null(), "failed to allocate child PEB");

    unsafe {
        let stack_base: u64;
        core::arch::asm!("mov {}, rsp", out(reg) stack_base);
        let stack_base = (stack_base + 0x100000) & !0xFFF;
        let stack_limit = stack_base.saturating_sub(0x200000);

        ptr::write(teb.add(TIB_STACK_BASE) as *mut u64, stack_base);
        ptr::write(teb.add(TIB_STACK_LIMIT) as *mut u64, stack_limit);
        ptr::write(teb.add(TEB_SELF) as *mut u64, teb as u64);
        ptr::write(teb.add(TEB_PEB) as *mut u64, peb as u64);
    }

    let ret = unsafe {
        libc::syscall(
            libc::SYS_arch_prctl,
            ARCH_SET_GS as libc::c_ulong,
            teb as u64,
        )
    };
    assert!(ret == 0, "arch_prctl(ARCH_SET_GS) failed in child thread");

    debug!(
        teb = format_args!("{teb:#p}"),
        "child thread TEB initialized"
    );
}

// ── CreateThread ─────────────────────────────────────────────────

/// Windows thread start routine: `DWORD WINAPI ThreadProc(LPVOID)`.
type ThreadStartRoutine = unsafe extern "win64" fn(usize) -> u32;

/// Global counter for synthetic thread IDs.
static NEXT_THREAD_ID: AtomicU32 = AtomicU32::new(1000);

/// CreateThread — create a new thread.
///
/// # Windows signature
/// ```c
/// HANDLE CreateThread(
///     LPSECURITY_ATTRIBUTES  lpThreadAttributes,   // rcx (ignored)
///     SIZE_T                 dwStackSize,           // rdx (ignored)
///     LPTHREAD_START_ROUTINE lpStartAddress,        // r8
///     LPVOID                 lpParameter,           // r9
///     DWORD                  dwCreationFlags,       // stack [rsp+0x28]
///     LPDWORD                lpThreadId             // stack [rsp+0x30]
/// );
/// ```
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn CreateThread(
    _security_attrs: usize,
    _stack_size: usize,
    start_address: usize,
    parameter: usize,
    creation_flags: u32,
    thread_id_out: *mut u32,
) -> isize {
    if start_address == 0 {
        warn!("CreateThread: null start address");
        return INVALID_HANDLE_VALUE.as_raw();
    }

    if creation_flags & 0x04 != 0 {
        warn!("CreateThread: CREATE_SUSPENDED not yet supported; starting immediately");
    }

    let exit_code = Arc::new(AtomicU32::new(STILL_ACTIVE));
    let completed = Arc::new((Mutex::new(false), Condvar::new()));

    let exit_code_child = Arc::clone(&exit_code);
    let completed_child = Arc::clone(&completed);

    let result = std::thread::Builder::new().spawn(move || {
        // TEB setup for this thread.
        unsafe { setup_child_teb() };

        // Call the PE thread function using the Windows x64 calling convention.
        let start_fn: ThreadStartRoutine = unsafe { core::mem::transmute(start_address) };
        let code = unsafe { start_fn(parameter) };

        // Record exit code and wake any waiters.
        exit_code_child.store(code, Ordering::Release);
        let (lock, cvar) = &*completed_child;
        *lock.lock().unwrap() = true;
        cvar.notify_all();

        debug!(exit_code = code, "child thread exited");
    });

    match result {
        Ok(handle) => {
            let tid = NEXT_THREAD_ID.fetch_add(1, Ordering::Relaxed);
            if !thread_id_out.is_null() {
                unsafe { ptr::write(thread_id_out, tid) };
            }

            // The JoinHandle is dropped (detached).  We track the thread
            // exclusively through the Arc-backed waitable state.
            drop(handle);

            let waitable = ThreadWaitable {
                exit_code,
                completed,
            };
            let h = handle_table().insert(HandleEntry::Thread(waitable));
            debug!(?h, tid, "created thread");
            h.as_raw()
        }
        Err(e) => {
            warn!("CreateThread: spawn failed: {e}");
            INVALID_HANDLE_VALUE.as_raw()
        }
    }
}

// ── TLS ──────────────────────────────────────────────────────────

/// TlsAlloc — allocate a TLS index.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn TlsAlloc() -> u32 {
    match threading::tls_alloc() {
        Some(idx) => {
            debug!(index = idx, "TlsAlloc");
            idx
        }
        None => {
            warn!("TlsAlloc: no free TLS slots");
            TLS_OUT_OF_INDEXES
        }
    }
}

/// TlsFree — release a TLS index.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn TlsFree(tls_index: u32) -> WinBool {
    if threading::tls_free(tls_index) {
        WinBool::TRUE
    } else {
        WinBool::FALSE
    }
}

/// TlsGetValue — retrieve the current thread's value for a slot.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn TlsGetValue(tls_index: u32) -> usize {
    threading::tls_get_value(tls_index)
}

/// TlsSetValue — set the current thread's value for a slot.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn TlsSetValue(tls_index: u32, value: usize) -> WinBool {
    if threading::tls_set_value(tls_index, value) {
        WinBool::TRUE
    } else {
        WinBool::FALSE
    }
}

// ── Thread query ─────────────────────────────────────────────────

/// GetCurrentThread — pseudo-handle for the calling thread.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn GetCurrentThread() -> isize {
    -2 // Windows pseudo-handle
}

/// GetCurrentThreadId — Linux tid mapped to a DWORD.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn GetCurrentThreadId() -> u32 {
    unsafe { libc::syscall(libc::SYS_gettid) as u32 }
}

/// GetExitCodeThread — read exit code (STILL_ACTIVE while running).
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn GetExitCodeThread(
    thread_handle: isize,
    exit_code_out: *mut u32,
) -> WinBool {
    if exit_code_out.is_null() {
        return WinBool::FALSE;
    }
    let h = Handle::from_raw(thread_handle);
    match handle_table().get_thread_exit_code(h) {
        Some(code) => {
            unsafe { ptr::write(exit_code_out, code) };
            WinBool::TRUE
        }
        None => WinBool::FALSE,
    }
}

// ── Wait ─────────────────────────────────────────────────────────

/// WaitForSingleObject — block until one handle is signalled or timeout.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn WaitForSingleObject(handle: isize, timeout_ms: u32) -> u32 {
    let h = Handle::from_raw(handle);
    match handle_table().get_waitable(h) {
        Some(waitable) => threading::wait_on(&waitable, timeout_ms),
        None => {
            warn!(handle, "WaitForSingleObject: invalid handle");
            WaitStatus::WAIT_FAILED.0
        }
    }
}

/// WaitForMultipleObjects — block until one or all handles are signalled.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn WaitForMultipleObjects(
    count: u32,
    handles_ptr: *const isize,
    wait_all: WinBool,
    timeout_ms: u32,
) -> u32 {
    if handles_ptr.is_null() || count == 0 || count > 64 {
        return WaitStatus::WAIT_FAILED.0;
    }

    let raw_handles: Vec<isize> = (0..count as usize)
        .map(|i| unsafe { *handles_ptr.add(i) })
        .collect();

    let waitables: Vec<Option<Waitable>> = raw_handles
        .iter()
        .map(|&raw| handle_table().get_waitable(Handle::from_raw(raw)))
        .collect();

    if waitables.iter().any(|w| w.is_none()) {
        warn!("WaitForMultipleObjects: one or more invalid handles");
        return WaitStatus::WAIT_FAILED.0;
    }
    let waitables: Vec<Waitable> = waitables.into_iter().flatten().collect();

    if wait_all.is_true() {
        // Wait for ALL objects — sequentially, adjusting timeout.
        let start = std::time::Instant::now();
        for w in &waitables {
            let remaining = if timeout_ms == INFINITE {
                INFINITE
            } else {
                let elapsed = start.elapsed().as_millis() as u32;
                if elapsed >= timeout_ms {
                    return WaitStatus::WAIT_TIMEOUT.0;
                }
                timeout_ms - elapsed
            };
            let result = threading::wait_on(w, remaining);
            if result != WaitStatus::WAIT_OBJECT_0.0 {
                return result;
            }
        }
        WaitStatus::WAIT_OBJECT_0.0
    } else {
        // Wait for ANY object — poll with short sleeps.
        let start = std::time::Instant::now();
        loop {
            for (i, w) in waitables.iter().enumerate() {
                if threading::wait_on(w, 0) == WaitStatus::WAIT_OBJECT_0.0 {
                    return WaitStatus(WaitStatus::WAIT_OBJECT_0.0 + i as u32).0;
                }
            }
            if timeout_ms != INFINITE {
                let elapsed = start.elapsed().as_millis() as u32;
                if elapsed >= timeout_ms {
                    return WaitStatus::WAIT_TIMEOUT.0;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}

// ── Sleep ────────────────────────────────────────────────────────

/// Sleep — suspend execution for the given number of milliseconds.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn Sleep(milliseconds: u32) {
    let dur = std::time::Duration::from_millis(milliseconds as u64);
    std::thread::sleep(dur);
}
