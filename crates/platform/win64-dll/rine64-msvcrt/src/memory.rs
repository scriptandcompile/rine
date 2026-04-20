//! MSVCRT C memory functions: malloc, calloc, free, memcpy.
//!
//! Forwards to the host libc. Since these are non-variadic and take simple
//! types, the `extern "win64"` declaration lets the compiler handle the
//! ABI translation to SysV calls internally.

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
pub unsafe extern "win64" fn malloc(size: usize) -> *mut c_void {
    unsafe { common::malloc(size) }
}

/// Allocate and zero-initialize an array.
///
/// # Arguments
/// * `count` - The number of elements to allocate.
/// * `size` - The size of each element, in bytes.
///
/// # Safety
/// This is unsafe because it returns a raw pointer to a memory block. The caller must ensure
/// that the pointer is properly managed and eventually freed to avoid memory leaks or undefined behavior.
///
/// # Returns
/// A pointer to the allocated memory block, or null if the allocation fails.
pub unsafe extern "win64" fn calloc(count: usize, size: usize) -> *mut c_void {
    unsafe { common::calloc(count, size) }
}

/// realloc — resize a memory block.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "win64" fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void {
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

/// free — free a previously allocated memory block.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "win64" fn free(ptr: *mut c_void) {
    if let Some(sz) = common::CRT_ALLOCATIONS.forget(ptr) {
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
