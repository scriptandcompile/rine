use super::*;

/// Allocates a block of memory from the default process heap.
///
/// # Arguments
/// * `uflags` - Allocation options. Supported flags:
///   - `GHND` (0x0042): Combines `GMEM_MOVEABLE` and `GMEM_ZEROINIT`.
///   - `GMEM_FIXED` (0x0000): Allocates fixed memory.
///     The return value is a pointer to the allocated memory block.
///     This value is not a handle and cannot be used with `GlobalLock`.
///   - `GMEM_MOVEABLE` (0x0002): Allocates movable memory.
///     Movable memory is allocated as a global handle that can be locked and unlocked to obtain a pointer to the memory.
///     The return value is a handle to the allocated memory block.
///     To translate a movable memory handle to a pointer, use `GlobalLock`.
///   - `GMEM_ZEROINIT` (0x0040): Initializes memory to zero.
///   - `GPTR` (0x0040): Combines `GMEM_FIXED` and `GMEM_ZEROINIT`.
/// * `size` - The number of bytes to allocate.
///   If this parameter is zero, the function allocates the minimum possible size (1 byte).
///
/// # Safety
/// The caller is responsible for ensuring that the allocated memory is freed using `GlobalFree` when it is no longer needed.
/// Failure to do so may result in memory leaks or other undefined behavior. Additionally, the caller must ensure that the `uflags`
/// parameter is set to a valid combination of flags, as invalid combinations may result in undefined behavior.
/// For example, `GMEM_MOVEABLE` cannot be combined with `GMEM_FIXED`.
///
/// # Returns
/// If the function succeeds, the return value is a pointer to the allocated memory block if `GMEM_FIXED` is specified,
/// or a handle to the allocated memory block if `GMEM_MOVEABLE` is specified.
/// If the function fails, the return value is `NULL`, and extended error information should be (but currently cannot)
/// obtained by calling `GetLastError`.
///
/// # Notes
/// The default process heap cannot be destroyed, and attempting to do so will fail,
/// but this function can still be used to allocate memory from the default heap.
/// This function is a simplified implementation of the Windows API `GlobalAlloc` that only supports allocation from the default process heap,
/// and does not support all of the flags or behaviors of the Windows API. It is provided for compatibility with code that uses `GlobalAlloc`,
/// but for new code or code that requires more advanced heap management features,
/// it is recommended to use `HeapAlloc` with the default heap handle instead.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GlobalAlloc(_uflags: u32, size: usize) -> HGLOBAL {
    unsafe { common::memory::global_alloc(_uflags, size) }
}

/// Frees the specified global memory object and invalidates its handle.
///
/// # Arguments
/// * `hmem` - A handle to the global memory object. This handle is returned by `GlobalAlloc`.
///   If this parameter is `NULL`, the function does nothing and returns `NULL`.
///
/// # Safety
/// The caller must ensure that `hmem` is a valid handle returned by `GlobalAlloc`, and that it has not already been freed.
/// Freeing an invalid handle or a handle that has already been freed results in undefined behavior.
/// Additionally, the caller must ensure that the memory being freed is not currently in use by any other part of the program.
///
/// # Returns
/// If the function succeeds, the return value is `NULL`. If the function fails, the return value is the handle passed in `hmem`,
/// and extended error information should be (but currently cannot) obtained by calling `GetLastError`.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GlobalFree(hmem: HGLOBAL) -> HGLOBAL {
    unsafe { common::memory::global_free(hmem) }
}

/// Lock a global memory object and return a pointer to the first byte of the memory block.
///
/// # Arguments
/// * `hmem` - A handle to the global memory object. This handle is returned by `GlobalAlloc`.
///   If this parameter is `NULL`, the function returns `NULL`.
///
/// # Safety
/// The caller must ensure that `hmem` is a valid handle returned by `GlobalAlloc`, and that it has not already been freed.
/// Locking an invalid handle or a handle that has already been freed results in undefined behavior.
/// Additionally, the caller must ensure that the memory being accessed through the returned pointer is not currently in use by any
/// other part of the program, and that it is properly synchronized if accessed from multiple threads.
///
/// # Returns
/// If the function succeeds, the return value is a pointer to the first byte of the memory block associated with the handle.
/// If the function fails, the return value is `NULL`, and extended error information should be (but currently cannot)
/// obtained by calling `GetLastError`.
///
/// # Notes
/// This function is a simplified implementation of the Windows API `GlobalLock` that only supports locking memory allocated from the default process heap,
/// and does not support all of the flags or behaviors of the Windows API.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GlobalLock(hmem: HGLOBAL) -> *mut u8 {
    unsafe { common::memory::global_lock(hmem) }
}

/// Unlock a global memory object.
///
/// # Arguments
/// * `hmem` - A handle to the global memory object. This handle is returned by `GlobalAlloc`.
///   If this parameter is `NULL`, the function does nothing and returns `TRUE`.
///
/// # Safety
/// The caller must ensure that `hmem` is a valid handle returned by `GlobalAlloc`, and that it has not already been freed.
/// Unlocking an invalid handle or a handle that has already been freed results in undefined behavior.
/// Additionally, the caller must ensure that the memory being accessed through the handle is not
/// currently in use by any other part of the program, and that it is properly synchronized if accessed from multiple threads.
/// Finally, the caller must ensure that the function is not called concurrently from multiple threads with the same handle,
/// as this may also result in undefined behavior.
///
/// # Returns
/// If the function succeeds, the return value is `TRUE`.
/// If the function fails, the return value is `FALSE`, and extended error information should be (but currently cannot)
/// obtained by calling `GetLastError`.
///
/// # Notes
/// Since our `GlobalAlloc` implementation doesn't actually support movable memory, there's nothing to do here, and the function always succeeds.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GlobalUnlock(hmem: HGLOBAL) -> BOOL {
    unsafe { super::local::LocalUnlock(hmem.into()) }
}

/// Reallocates a global memory object.
///
/// # Arguments
/// * `hmem` - A handle to the global memory object. This handle is returned by `GlobalAlloc`.
/// * `new_size` - The new size of the memory block, in bytes.
///   If this parameter is zero, the function allocates the minimum possible size (1 byte).
/// * `flags` - The reallocation flags.
///   If `LMEM_MOVEABLE` is specified, the function allocates a new block of memory,
///   copies the data from the old block to the new block, frees the old block, and returns a handle to the new block.
///   If `LMEM_MOVEABLE` is not specified, the function attempts to resize the existing block in place, and returns the same handle on success.
///
/// # Safety
/// The caller must ensure that `hmem` is a valid handle returned by `GlobalAlloc`, and that it has not already been freed.
/// Reallocating an invalid handle or a handle that has already been freed results in undefined behavior.
/// Additionally, the caller must ensure that the memory being accessed through the handle is not currently in use by any other
/// part of the program, and that it is properly synchronized if accessed from multiple threads.
///
/// # Returns
/// If the function succeeds, the return value is a handle to the reallocated memory block.
/// If the function fails, the return value is `NULL`, and extended error information should be (but currently cannot)
/// obtained by calling `GetLastError`.
///
/// # Notes
/// Since our `GlobalAlloc` implementation doesn't actually support movable memory, we ignore the flags and just allocate a new block of memory,
/// copy the data, and free the old block. This means that the function always returns a new handle, and the old handle is always freed,
/// regardless of the flags.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GlobalReAlloc(
    _hmem: HGLOBAL,
    _new_size: usize,
    _flags: u32,
) -> HGLOBAL {
    unsafe { common::memory::global_realloc(_hmem, _new_size, _flags) }
}
