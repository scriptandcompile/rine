use super::*;

/// Get the default process heap handle.
///
/// # Safety
/// The caller must not call HeapDestroy on the returned handle, and must not use the heap after it has been destroyed by the system.
/// The caller is responsible for freeing any memory allocated from the heap using HeapFree before the process exits.
///
/// # Returns
/// A handle to the default process heap. This handle is valid for the lifetime of the process and should not be closed by the caller.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetProcessHeap() -> HANDLE {
    *common::memory::DEFAULT_HEAP
}

/// HeapCreate — create a new private heap.
///
/// # Arguments
/// * `options`: heap flags (HEAP_GENERATE_EXCEPTIONS, HEAP_NO_SERIALIZE, etc.)
/// * `_initial_size`: ignored.
/// * `_maximum_size`: ignored.
///
/// # Safety
/// The caller must eventually call HeapDestroy on the returned handle, and must not use the heap after it has been destroyed.
/// The caller is responsible for freeing any memory allocated from the heap using HeapFree before destroying the heap.
///
/// # Returns
/// On success, returns a handle to the newly created heap. On failure, returns `NULL` (0).
///
/// # Note
/// The default process heap returned by GetProcessHeap cannot be created or destroyed using
/// HeapCreate or HeapDestroy, and attempting to do so will fail.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn HeapCreate(
    options: u32,
    _initial_size: usize,
    _maximum_size: usize,
) -> HANDLE {
    match unsafe { common::memory::heap_create(options) } {
        Some(handle) => handle,
        None => HANDLE::NULL,
    }
}

/// HeapDestroy — destroy a private heap.
///
/// # Arguments
/// * `heap_handle` - A handle to the heap to destroy, returned by HeapCreate.
///
/// # Returns
///
/// `TRUE` if the heap was successfully destroyed, or `FALSE` if the handle was invalid or the heap could not be destroyed.
///
/// # Safety
/// The caller must ensure that `heap_handle` is a valid handle returned by HeapCreate, and that there are no outstanding
/// allocations from the heap.
///
/// # Note
/// The default process heap cannot be destroyed, and attempting to do so will fail.
/// This does not free any outstanding allocations from the heap; it is the caller's responsibility to free them first.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn HeapDestroy(heap_handle: HANDLE) -> BOOL {
    rine_types::dev_notify!(on_handle_closed(heap_handle.as_raw() as i64));

    common::memory::heap_destroy(heap_handle)
}

/// Allocate a block from a heap.
///
/// # Arguments
/// * `heap_handle` - A handle to the heap from which the memory will be allocated, returned by HeapCreate or GetProcessHeap.
/// * `flags` - Allocation options. Supported flags:
///     * `HEAP_ZERO_MEMORY` (0x00000008): If this flag is specified, the allocated memory will be initialized to zero.
/// * `size` - The number of bytes to allocate. If this parameter is zero, the function allocates the minimum possible size (1 byte).
///
/// # Returns
/// If the function succeeds, the return value is a pointer to the allocated memory block.
/// If the function fails, the return value is `NULL`.
///
/// # Safety
/// The caller must ensure that `heap_handle` is a valid handle returned by HeapCreate or GetProcessHeap, and that the heap has not been
/// destroyed. The caller is responsible for freeing the allocated memory using HeapFree when it is no longer needed.
///
/// # Note
/// * `HEAP_NO_SERIALIZE` (0x00000001) and `HEAP_GENERATE_EXCEPTIONS` (0x00000004) are accepted but have no effect in this implementation.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn HeapAlloc(heap_handle: HANDLE, flags: u32, size: usize) -> *mut u8 {
    if size == 0 {
        // Windows HeapAlloc with size 0 returns a valid non-null pointer.
        return common::memory::heap_alloc(heap_handle, flags, 1);
    }
    common::memory::heap_alloc(heap_handle, flags, size)
}

/// Free a block of memory allocated from a heap by HeapAlloc.
///
/// # Arguments
/// * `heap_handle` - A handle to the heap from which the memory was allocated, returned by HeapCreate or GetProcessHeap.
/// * `_flags` - Ignored in this implementation.
/// * `ptr` - A pointer to a memory block allocated from the heap by HeapAlloc or HeapReAlloc.
///   If this parameter is `NULL`, the function does nothing and returns `TRUE`.
///
/// # Safety
/// The caller must ensure that `heap_handle` is a valid handle returned by HeapCreate or GetProcessHeap, and that there
/// are no outstanding allocations from the heap. The caller must also ensure that `ptr` is either `NULL` or a pointer
/// returned by HeapAlloc or HeapReAlloc from the specified heap, and that it has not already been freed.
/// Freeing an invalid pointer or a pointer from a different heap results in undefined behavior.
///
/// # Returns
/// If the function succeeds, the return value is `TRUE`. If the function fails, the return value is `FALSE`, and extended
/// error information should be (but currently cannot) be obtained by calling GetLastError.
///
/// # Notes
/// * If `ptr` is `NULL`, the function does nothing and returns `TRUE`.
/// * The default process heap cannot be destroyed, and attempting to do so will fail, but this function can still be used
///   to free allocations from the default heap.
/// * Missing implementation features:
///   * `_flags` semantics are ignored.
///   * No Win32-accurate `GetLastError` mapping is provided on failure.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn HeapFree(heap_handle: HANDLE, _flags: u32, ptr: *mut u8) -> BOOL {
    if ptr.is_null() {
        return BOOL::TRUE;
    }

    unsafe { common::memory::heap_free(heap_handle, _flags, ptr) }
}
/// Reallocate a block of memory from a heap by HeapReAlloc.
///
/// # Arguments
/// * `heap_handle` - A handle to the heap from which the memory was allocated, returned by HeapCreate or GetProcessHeap.
/// * `flags` - Allocation options. Supported flags:
///   * `HEAP_ZERO_MEMORY` (0x00000008): If this flag is specified and the new size is larger than the old size,
///     the additional memory will be initialized to zero.
/// * `ptr` - A pointer to a memory block allocated from the heap by HeapAlloc or HeapReAlloc.
///   If this parameter is `NULL`, the function behaves like HeapAlloc.
/// * `new_size` - The new size of the memory block in bytes. If this parameter is zero, the function allocates
///   the minimum possible size (1 byte).
///
/// # Safety
/// The caller must ensure that `heap_handle` is a valid handle returned by HeapCreate or GetProcessHeap, and that there
/// are no outstanding allocations from the heap. The caller must also ensure that `ptr` is either `NULL` or a pointer
/// returned by HeapAlloc or HeapReAlloc from the specified heap, and that it has not already been freed.
///
/// # Returns
/// If the function succeeds, the return value is a pointer to the reallocated memory block, which may be the same
/// as `ptr` or a different location. If the function fails, the return value is `NULL`, and extended error
/// information should be (but currently cannot) be obtained by calling GetLastError.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn HeapReAlloc(
    heap_handle: HANDLE,
    flags: u32,
    ptr: *mut u8,
    new_size: usize,
) -> *mut u8 {
    if ptr.is_null() {
        return common::memory::heap_alloc(heap_handle, flags, new_size);
    }

    unsafe { common::memory::heap_realloc(heap_handle, flags, ptr, new_size) }
}

/// Get the size of a heap allocation.
///
/// # Arguments
/// * `heap_handle` - A handle to the heap from which the memory was allocated, returned by HeapCreate or GetProcessHeap.
/// * `_flags` - Ignored in this implementation.
/// * `ptr` - A pointer to a memory block allocated from the heap by HeapAlloc or HeapReAlloc.
///
/// # Safety
/// The caller must ensure that `heap_handle` is a valid handle returned by HeapCreate or GetProcessHeap,
/// and that the heap has not been destroyed.
/// The caller must also ensure that `ptr` is a pointer returned by HeapAlloc or HeapReAlloc from the specified heap,
/// and that it has not already been freed. An invalid handle or pointer results in undefined behavior.
///
/// # Returns
/// The size of the allocated block in bytes, or `-1` (usize::MAX) if the handle or pointer is invalid.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn HeapSize(heap_handle: HANDLE, _flags: u32, ptr: *const u8) -> usize {
    common::memory::heap_size(heap_handle, _flags, ptr)
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- GetProcessHeap -------------------------------------------------

    #[test]
    fn get_process_heap_returns_nonzero() {
        let h = unsafe { GetProcessHeap() };
        assert_ne!(
            h,
            HANDLE::NULL,
            "GetProcessHeap should return a valid handle"
        );
    }

    #[test]
    fn get_process_heap_is_stable() {
        let h1 = unsafe { GetProcessHeap() };
        let h2 = unsafe { GetProcessHeap() };
        assert_eq!(h1, h2, "GetProcessHeap should return the same handle");
    }

    // -- HeapCreate / HeapDestroy --------------------------------------

    #[test]
    fn heap_create_returns_nonzero() {
        let h = unsafe { HeapCreate(0, 0, 0) };
        assert_ne!(h, HANDLE::NULL);
        unsafe { HeapDestroy(h) };
    }

    #[test]
    fn heap_create_different_from_process_heap() {
        let default = unsafe { GetProcessHeap() };
        let custom = unsafe { HeapCreate(0, 0, 0) };
        assert_ne!(default, custom);
        unsafe { HeapDestroy(custom) };
    }

    #[test]
    fn heap_destroy_default_heap_fails() {
        let h = unsafe { GetProcessHeap() };
        let result = unsafe { HeapDestroy(h) };
        assert!(
            !result.is_true(),
            "should not be able to destroy default heap"
        );
    }

    #[test]
    fn heap_destroy_invalid_handle_fails() {
        let result = unsafe { HeapDestroy(HANDLE::from_raw(0xDEAD)) };
        assert!(!result.is_true());
    }

    // -- HeapAlloc / HeapFree ------------------------------------------

    #[test]
    fn heap_alloc_returns_non_null() {
        let heap = unsafe { GetProcessHeap() };
        let ptr = unsafe { HeapAlloc(heap, 0, 64) };
        assert!(!ptr.is_null());
        unsafe { HeapFree(heap, 0, ptr) };
    }

    #[test]
    fn heap_alloc_zero_size_returns_non_null() {
        let heap = unsafe { GetProcessHeap() };
        let ptr = unsafe { HeapAlloc(heap, 0, 0) };
        assert!(!ptr.is_null(), "HeapAlloc(0) should return a valid pointer");
        unsafe { HeapFree(heap, 0, ptr) };
    }

    #[test]
    fn heap_alloc_zero_memory_flag() {
        let heap = unsafe { GetProcessHeap() };
        let ptr = unsafe { HeapAlloc(heap, common::memory::HEAP_ZERO_MEMORY, 128) };
        assert!(!ptr.is_null());
        let slice = unsafe { std::slice::from_raw_parts(ptr, 128) };
        assert!(
            slice.iter().all(|&b| b == 0),
            "HEAP_ZERO_MEMORY should zero the block"
        );
        unsafe { HeapFree(heap, 0, ptr) };
    }

    #[test]
    fn heap_alloc_write_read() {
        let heap = unsafe { GetProcessHeap() };
        let ptr = unsafe { HeapAlloc(heap, 0, 16) };
        assert!(!ptr.is_null());
        unsafe {
            std::ptr::write_bytes(ptr, 0xAB, 16);
            assert_eq!(*ptr, 0xAB);
            assert_eq!(*ptr.add(15), 0xAB);
        }
        unsafe { HeapFree(heap, 0, ptr) };
    }

    #[test]
    fn heap_free_null_succeeds() {
        let heap = unsafe { GetProcessHeap() };
        let result = unsafe { HeapFree(heap, 0, std::ptr::null_mut()) };
        assert!(result.is_true(), "HeapFree(NULL) should succeed");
    }

    #[test]
    fn heap_free_invalid_pointer_fails() {
        let heap = unsafe { GetProcessHeap() };
        let result = unsafe { HeapFree(heap, 0, 0xDEAD_BEEF as *mut u8) };
        assert!(
            !result.is_true(),
            "HeapFree with unknown pointer should fail"
        );
    }

    #[test]
    fn heap_alloc_on_custom_heap() {
        let heap = unsafe { HeapCreate(0, 0, 0) };
        let ptr = unsafe { HeapAlloc(heap, common::memory::HEAP_ZERO_MEMORY, 256) };
        assert!(!ptr.is_null());
        let slice = unsafe { std::slice::from_raw_parts(ptr, 256) };
        assert!(slice.iter().all(|&b| b == 0));
        unsafe { HeapFree(heap, 0, ptr) };
        unsafe { HeapDestroy(heap) };
    }

    // -- HeapReAlloc ----------------------------------------------------

    #[test]
    fn heap_realloc_grows() {
        let heap = unsafe { GetProcessHeap() };
        let ptr = unsafe { HeapAlloc(heap, 0, 16) };
        assert!(!ptr.is_null());
        unsafe { std::ptr::write_bytes(ptr, 0x42, 16) };

        let new_ptr = unsafe { HeapReAlloc(heap, 0, ptr, 64) };
        assert!(!new_ptr.is_null());
        let slice = unsafe { std::slice::from_raw_parts(new_ptr, 16) };
        assert!(slice.iter().all(|&b| b == 0x42));

        unsafe { HeapFree(heap, 0, new_ptr) };
    }

    #[test]
    fn heap_realloc_shrinks() {
        let heap = unsafe { GetProcessHeap() };
        let ptr = unsafe { HeapAlloc(heap, 0, 256) };
        assert!(!ptr.is_null());
        unsafe { std::ptr::write_bytes(ptr, 0x55, 256) };

        let new_ptr = unsafe { HeapReAlloc(heap, 0, ptr, 32) };
        assert!(!new_ptr.is_null());
        let slice = unsafe { std::slice::from_raw_parts(new_ptr, 32) };
        assert!(slice.iter().all(|&b| b == 0x55));

        unsafe { HeapFree(heap, 0, new_ptr) };
    }

    #[test]
    fn heap_realloc_null_acts_as_alloc() {
        let heap = unsafe { GetProcessHeap() };
        let ptr = unsafe {
            HeapReAlloc(
                heap,
                common::memory::HEAP_ZERO_MEMORY,
                std::ptr::null_mut(),
                64,
            )
        };
        assert!(!ptr.is_null());
        let slice = unsafe { std::slice::from_raw_parts(ptr, 64) };
        assert!(slice.iter().all(|&b| b == 0));
        unsafe { HeapFree(heap, 0, ptr) };
    }

    #[test]
    fn heap_realloc_zero_memory_zeros_extra() {
        let heap = unsafe { GetProcessHeap() };
        let ptr = unsafe { HeapAlloc(heap, 0, 16) };
        assert!(!ptr.is_null());
        unsafe { std::ptr::write_bytes(ptr, 0xFF, 16) };

        let new_ptr = unsafe { HeapReAlloc(heap, common::memory::HEAP_ZERO_MEMORY, ptr, 64) };
        assert!(!new_ptr.is_null());
        let orig = unsafe { std::slice::from_raw_parts(new_ptr, 16) };
        assert!(orig.iter().all(|&b| b == 0xFF));
        let ext = unsafe { std::slice::from_raw_parts(new_ptr.add(16), 48) };
        assert!(
            ext.iter().all(|&b| b == 0),
            "extended region should be zeroed"
        );

        unsafe { HeapFree(heap, 0, new_ptr) };
    }

    // -- HeapSize -------------------------------------------------------

    #[test]
    fn heap_size_returns_allocation_size() {
        let heap = unsafe { GetProcessHeap() };
        let ptr = unsafe { HeapAlloc(heap, 0, 100) };
        assert!(!ptr.is_null());
        let size = unsafe { HeapSize(heap, 0, ptr) };
        assert_eq!(size, 100);
        unsafe { HeapFree(heap, 0, ptr) };
    }

    #[test]
    fn heap_size_after_realloc() {
        let heap = unsafe { GetProcessHeap() };
        let ptr = unsafe { HeapAlloc(heap, 0, 50) };
        let new_ptr = unsafe { HeapReAlloc(heap, 0, ptr, 200) };
        let size = unsafe { HeapSize(heap, 0, new_ptr) };
        assert_eq!(size, 200);
        unsafe { HeapFree(heap, 0, new_ptr) };
    }

    #[test]
    fn heap_size_invalid_ptr_returns_max() {
        let heap = unsafe { GetProcessHeap() };
        let size = unsafe { HeapSize(heap, 0, 0xBAAD as *const u8) };
        assert_eq!(size, usize::MAX);
    }
}
