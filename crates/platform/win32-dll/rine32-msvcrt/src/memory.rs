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
pub unsafe extern "C" fn calloc(count: usize, size: usize) -> *mut c_void {
    unsafe { common::calloc(count, size) }
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
pub unsafe extern "C" fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void {
    unsafe { common::realloc(ptr, size) }
}

/// Free a previously allocated memory block.
///
/// # Arguments
/// * `ptr` - A pointer to the memory block to free.
///   This must be a pointer returned by a previous call to `malloc`, `calloc`, or `realloc`.
///
/// # Safety
/// This is unsafe because it operates on a raw pointer.
/// The caller must ensure that `ptr` is either null or a pointer returned by a previous call to
/// `malloc`, `calloc`, or `realloc`, and that it is not used after being freed to avoid undefined behavior.
///
/// # Notes
/// If `ptr` is null, this function does nothing.
/// Otherwise, it frees the memory block pointed to by `ptr` and removes it from the allocation tracker,
/// notifying the dev tools about the deallocation.
pub unsafe extern "C" fn free(ptr: *mut c_void) {
    unsafe { common::free(ptr) }
}

/// Copy n bytes from src to dest.
///
/// # Arguments
/// * `dest` - A pointer to the destination buffer where the content is to be copied.
/// * `src` - A pointer to the source of data to be copied.
/// * `n` - The number of bytes to copy.
///
/// # Safety
/// This is unsafe because it operates on raw pointers.
/// The caller must ensure that `dest` and `src` are valid pointers to memory blocks of at least `n` bytes,
/// and that the memory regions do not overlap in a way that violates Rust's aliasing rules.
/// Additionally, the caller must ensure that the memory pointed to by `dest` is writable and that the memory
/// pointed to by `src` is readable to avoid undefined behavior.
///
/// # Returns
/// A pointer to the destination buffer (`dest`).
///
/// # Notes
/// This function performs a byte-wise copy of `n` bytes from the memory area pointed to by `src` to the
/// memory area pointed to by `dest`.
/// The behavior is undefined if the memory areas overlap.
/// The caller is responsible for ensuring that the destination buffer has enough space to hold the copied
/// data and that both pointers are valid for the specified number of bytes.
pub unsafe extern "C" fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void {
    unsafe { common::memcpy(dest, src, n) }
}

/// Fill a block of memory with a byte value.
///
/// # Arguments
/// * `dest` - A pointer to the destination buffer where the content is to be filled.
/// * `c` - The byte value to fill the memory with (converted to unsigned char internally).
/// * `n` - The number of bytes to fill.
///
/// # Safety
/// This is unsafe because it operates on a raw pointer.
/// The caller must ensure that `dest` is a valid pointer to a memory block of at least `n` bytes and that it is
/// writable to avoid undefined behavior.
///
/// # Returns
/// A pointer to the destination buffer (`dest`).
///
/// # Notes
/// This function fills the first `n` bytes of the memory area pointed to by `dest` with the byte value `c`.
/// The caller is responsible for ensuring that the destination buffer has enough space to hold the filled data
/// and that the pointer is valid for the specified number of bytes.
pub unsafe extern "C" fn memset(dest: *mut c_void, c: i32, n: usize) -> *mut c_void {
    unsafe { common::memset(dest, c, n) }
}
