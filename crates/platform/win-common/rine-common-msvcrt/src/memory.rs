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
