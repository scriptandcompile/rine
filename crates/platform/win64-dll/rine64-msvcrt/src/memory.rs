//! MSVCRT C memory functions: malloc, calloc, free, memcpy.
//!
//! Forwards to the host libc. Since these are non-variadic and take simple
//! types, the `extern "win64"` declaration lets the compiler handle the
//! ABI translation to SysV calls internally.

use core::ffi::c_void;

/// malloc — allocate a block of memory.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "win64" fn malloc(size: usize) -> *mut c_void {
    unsafe { libc::malloc(size) }
}

/// calloc — allocate and zero-initialize an array.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "win64" fn calloc(count: usize, size: usize) -> *mut c_void {
    unsafe { libc::calloc(count, size) }
}

/// realloc — resize a memory block.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "win64" fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void {
    unsafe { libc::realloc(ptr, size) }
}

/// free — free a previously allocated memory block.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "win64" fn free(ptr: *mut c_void) {
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
