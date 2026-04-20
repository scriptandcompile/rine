use core::ffi::c_void;
use std::sync::LazyLock;

use rine_types::dev_notify;

use crate::AllocationTracker;

/// CRT memory allocation tracking for debugging purposes.
/// This is used to track allocations made through our custom `malloc`, `calloc`, and `realloc` implementations,
/// so that we can notify the dev tools about memory usage and potential leaks.
pub static CRT_ALLOCATIONS: LazyLock<AllocationTracker> = LazyLock::new(AllocationTracker::new);

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
pub unsafe fn malloc(size: usize) -> *mut c_void {
    let ptr = unsafe { libc::malloc(size) };
    if !ptr.is_null() {
        CRT_ALLOCATIONS.record(ptr, size);
        dev_notify!(on_memory_allocated(ptr as u64, size as u64, "malloc"));
    }
    ptr
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
pub unsafe fn calloc(count: usize, size: usize) -> *mut c_void {
    let ptr = unsafe { libc::calloc(count, size) };
    if !ptr.is_null() {
        let total = count.saturating_mul(size);
        CRT_ALLOCATIONS.record(ptr, total);
        rine_types::dev_notify!(on_memory_allocated(ptr as u64, total as u64, "calloc"));
    }
    ptr
}

/// Resize a memory block to a new size.
///
/// # Arguments
/// * `ptr` - A pointer to the memory block to resize. This must be a pointer returned by a previous call to
///   `malloc`, `calloc`, or `realloc`.
/// * `size` - The new size for the memory block, in bytes.
///
/// # Safety
/// This is unsafe because it returns a raw pointer to a memory block. The caller must ensure
/// that the pointer is properly managed and eventually freed to avoid memory leaks or undefined behavior.
/// Additionally, the caller must ensure that `ptr` is either null or a pointer returned by a previous call
/// to `malloc`, `calloc`, or `realloc`.
///
/// # Returns
/// A pointer to the resized memory block, which may be the same as `ptr` or a new location.
/// If the allocation fails, returns null and the original block is left unchanged.
///
/// # Notes
/// If `ptr` is null, this function behaves like `malloc(size)`.
/// If `size` is zero and `ptr` is not null, the block pointed to by `ptr` is freed and a null pointer is returned.
/// Otherwise, the function attempts to resize the block pointed to by `ptr` to `size` bytes, possibly moving it to a new location.
/// The contents of the block are preserved up to the lesser of the old and new sizes.
pub unsafe fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void {
    let old_size = CRT_ALLOCATIONS.forget(ptr);

    let new_ptr = unsafe { libc::realloc(ptr, size) };
    if new_ptr.is_null() {
        if let Some(sz) = old_size {
            CRT_ALLOCATIONS.restore(ptr, sz);
        }
        return new_ptr;
    }

    if let Some(sz) = old_size {
        rine_types::dev_notify!(on_memory_freed(ptr as u64, sz as u64, "realloc"));
    }

    CRT_ALLOCATIONS.record(new_ptr, size);
    rine_types::dev_notify!(on_memory_allocated(new_ptr as u64, size as u64, "realloc"));
    new_ptr
}
