//! MSVCRT C memory functions: malloc, calloc, free, memcpy.
//!
//! Forwards to the host libc. Since these are non-variadic and take simple
//! types, the standard `extern "C"` declaration works correctly.

use core::ffi::c_void;

use rine_common_msvcrt as common;

/// Allocate a block of memory.
///
/// # Arguments
/// * `size` - The size of the memory block to allocate, in bytes.
///
/// # Safety
/// This is unsafe because it returns a raw pointer to a memory block. The caller must ensure
/// that the pointer is properly managed and eventually freed to avoid memory leaks or undefined behavior.
///
/// # Returns
/// A pointer to the allocated memory block, or null if the allocation fails.
pub unsafe extern "C" fn malloc(size: usize) -> *mut c_void {
    unsafe { common::malloc(size) }
}

/// calloc — allocate and zero-initialize an array.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn calloc(count: usize, size: usize) -> *mut c_void {
    let ptr = unsafe { libc::calloc(count, size) };
    if !ptr.is_null() {
        let total = count.saturating_mul(size);
        common::CRT_ALLOCATIONS.record(ptr, total);
        rine_types::dev_notify!(on_memory_allocated(ptr as u64, total as u64, "calloc"));
    }
    ptr
}

/// realloc — resize a memory block.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void {
    let old_size = common::CRT_ALLOCATIONS.forget(ptr);

    let new_ptr = unsafe { libc::realloc(ptr, size) };
    if new_ptr.is_null() {
        if let Some(sz) = old_size {
            common::CRT_ALLOCATIONS.restore(ptr, sz);
        }
        return new_ptr;
    }

    if let Some(sz) = old_size {
        rine_types::dev_notify!(on_memory_freed(ptr as u64, sz as u64, "realloc"));
    }
    common::CRT_ALLOCATIONS.record(new_ptr, size);
    rine_types::dev_notify!(on_memory_allocated(new_ptr as u64, size as u64, "realloc"));
    new_ptr
}

/// free — deallocate and return a memory block.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn free(ptr: *mut c_void) {
    if let Some(sz) = common::CRT_ALLOCATIONS.forget(ptr) {
        rine_types::dev_notify!(on_memory_freed(ptr as u64, sz as u64, "free"));
    }
    unsafe { libc::free(ptr) };
}
