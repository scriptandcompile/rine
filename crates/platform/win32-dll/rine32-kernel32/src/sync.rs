use std::ptr;
use std::sync::{Arc, Condvar, Mutex};

use rine_common_kernel32 as common;
use rine_types::errors::WinBool;
use rine_types::handles::{Handle, HandleEntry, handle_table};
use rine_types::threading;
use tracing::debug;

unsafe fn init_cs(cs: *mut u8) {
    unsafe { ptr::write_bytes(cs, 0, 24) };

    let mutex = Box::into_raw(Box::new(unsafe {
        core::mem::zeroed::<libc::pthread_mutex_t>()
    }));
    let mut attr: libc::pthread_mutexattr_t = unsafe { core::mem::zeroed() };
    unsafe {
        libc::pthread_mutexattr_init(&mut attr);
        libc::pthread_mutexattr_settype(&mut attr, libc::PTHREAD_MUTEX_RECURSIVE);
        libc::pthread_mutex_init(mutex, &attr);
        libc::pthread_mutexattr_destroy(&mut attr);
    }

    unsafe { ptr::write(cs as *mut *mut libc::pthread_mutex_t, mutex) };
}

unsafe fn get_mutex(cs: *const u8) -> *mut libc::pthread_mutex_t {
    unsafe { ptr::read(cs as *const *mut libc::pthread_mutex_t) }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn InitializeCriticalSection(cs: *mut u8) {
    if cs.is_null() {
        return;
    }
    unsafe { init_cs(cs) };
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn EnterCriticalSection(cs: *mut u8) {
    if cs.is_null() {
        return;
    }
    let mut mutex = unsafe { get_mutex(cs) };
    if mutex.is_null() {
        unsafe { init_cs(cs) };
        mutex = unsafe { get_mutex(cs) };
    }
    unsafe { libc::pthread_mutex_lock(mutex) };
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn LeaveCriticalSection(cs: *mut u8) {
    if cs.is_null() {
        return;
    }
    let mutex = unsafe { get_mutex(cs) };
    if mutex.is_null() {
        return;
    }
    unsafe { libc::pthread_mutex_unlock(mutex) };
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn DeleteCriticalSection(cs: *mut u8) {
    if cs.is_null() {
        return;
    }
    let mutex = unsafe { get_mutex(cs) };
    if mutex.is_null() {
        return;
    }
    unsafe {
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
pub unsafe extern "stdcall" fn SetEvent(event_handle: isize) -> WinBool {
    let h = Handle::from_raw(event_handle);
    let waitable = match handle_table().get_waitable(h) {
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
    let h = Handle::from_raw(event_handle);
    let waitable = match handle_table().get_waitable(h) {
        Some(threading::Waitable::Event(e)) => e,
        _ => return WinBool::FALSE,
    };
    let mut signaled = waitable.inner.signaled.lock().unwrap();
    *signaled = false;
    WinBool::TRUE
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn CreateMutexA(
    _security_attrs: usize,
    initial_owner: WinBool,
    _name: *const u8,
) -> isize {
    let (owner, count) = if initial_owner.is_true() {
        (Some(std::thread::current().id()), 1)
    } else {
        (None, 0)
    };

    let waitable = threading::MutexWaitable {
        inner: Arc::new(threading::MutexInner {
            state: Mutex::new(threading::MutexState { owner, count }),
            condvar: Condvar::new(),
        }),
    };
    handle_table().insert(HandleEntry::Mutex(waitable)).as_raw()
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn ReleaseMutex(mutex_handle: isize) -> WinBool {
    let h = Handle::from_raw(mutex_handle);
    let waitable = match handle_table().get_waitable(h) {
        Some(threading::Waitable::Mutex(m)) => m,
        _ => return WinBool::FALSE,
    };
    let tid = std::thread::current().id();
    let mut state = waitable.inner.state.lock().unwrap();
    if state.owner != Some(tid) {
        return WinBool::FALSE;
    }
    state.count -= 1;
    if state.count == 0 {
        state.owner = None;
        waitable.inner.condvar.notify_one();
    }
    WinBool::TRUE
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

    let waitable = threading::SemaphoreWaitable {
        inner: Arc::new(threading::SemaphoreInner {
            count: Mutex::new(initial_count),
            max_count: maximum_count,
            condvar: Condvar::new(),
        }),
    };
    handle_table()
        .insert(HandleEntry::Semaphore(waitable))
        .as_raw()
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

    let h = Handle::from_raw(semaphore_handle);
    let waitable = match handle_table().get_waitable(h) {
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
