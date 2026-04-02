//! MSVCRT C memory functions: malloc, calloc, free, memcpy.
//!
//! Forwards to the host libc. Since these are non-variadic and take simple
//! types, the `extern "win64"` declaration lets the compiler handle the
//! ABI translation to SysV calls internally.

use core::ffi::c_void;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

static CRT_ALLOCATIONS: LazyLock<Mutex<HashMap<usize, usize>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// malloc — allocate a block of memory.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "win64" fn malloc(size: usize) -> *mut c_void {
    let ptr = unsafe { libc::malloc(size) };
    if !ptr.is_null() {
        CRT_ALLOCATIONS.lock().unwrap().insert(ptr as usize, size);
        rine_types::dev_notify!(on_memory_allocated(ptr as u64, size as u64, "malloc"));
    }
    ptr
}

/// calloc — allocate and zero-initialize an array.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "win64" fn calloc(count: usize, size: usize) -> *mut c_void {
    let ptr = unsafe { libc::calloc(count, size) };
    if !ptr.is_null() {
        let total = count.saturating_mul(size);
        CRT_ALLOCATIONS.lock().unwrap().insert(ptr as usize, total);
        rine_types::dev_notify!(on_memory_allocated(ptr as u64, total as u64, "calloc"));
    }
    ptr
}

/// realloc — resize a memory block.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "win64" fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void {
    let old_size = if !ptr.is_null() {
        CRT_ALLOCATIONS.lock().unwrap().remove(&(ptr as usize))
    } else {
        None
    };

    let new_ptr = unsafe { libc::realloc(ptr, size) };
    if new_ptr.is_null() {
        if let Some(sz) = old_size {
            CRT_ALLOCATIONS.lock().unwrap().insert(ptr as usize, sz);
        }
        return new_ptr;
    }

    if let Some(sz) = old_size {
        rine_types::dev_notify!(on_memory_freed(ptr as u64, sz as u64, "realloc"));
    }

    CRT_ALLOCATIONS
        .lock()
        .unwrap()
        .insert(new_ptr as usize, size);
    rine_types::dev_notify!(on_memory_allocated(new_ptr as u64, size as u64, "realloc"));
    new_ptr
}

/// free — free a previously allocated memory block.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "win64" fn free(ptr: *mut c_void) {
    if !ptr.is_null()
        && let Some(sz) = CRT_ALLOCATIONS.lock().unwrap().remove(&(ptr as usize))
    {
        rine_types::dev_notify!(on_memory_freed(ptr as u64, sz as u64, "free"));
    }
    unsafe { libc::free(ptr) }
}

/// memcpy — copy n bytes from src to dest.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "win64" fn memcpy(
    dest: *mut c_void,
    src: *const c_void,
    n: usize,
) -> *mut c_void {
    unsafe { libc::memcpy(dest, src, n) }
}

/// memset — fill memory with a byte value.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "win64" fn memset(dest: *mut c_void, c: i32, n: usize) -> *mut c_void {
    unsafe { libc::memset(dest, c, n) }
}
