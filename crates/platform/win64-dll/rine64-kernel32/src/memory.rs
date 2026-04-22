//! kernel32 memory functions: Heap API and VirtualAlloc/VirtualFree/VirtualProtect/VirtualQuery.

use rine_common_kernel32 as common;
use rine_types::errors::WinBool;
use rine_types::handles::Handle;

/// Get the default process heap handle.
///
/// # Safety
/// The caller must not call HeapDestroy on the returned handle, and must not use the heap after it has been destroyed by the system.
/// The caller is responsible for freeing any memory allocated from the heap using HeapFree before the process exits.
///
/// # Returns
/// A handle to the default process heap. This handle is valid for the lifetime of the process and should not be closed by the caller.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetProcessHeap() -> isize {
    common::memory::DEFAULT_HEAP.as_raw()
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
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn HeapCreate(
    options: u32,
    _initial_size: usize,
    _maximum_size: usize,
) -> isize {
    match unsafe { common::memory::heap_create(options) } {
        Some(handle) => handle.as_raw(),
        None => 0,
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
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn HeapDestroy(heap_handle: isize) -> WinBool {
    let handle = Handle::from_raw(heap_handle);
    rine_types::dev_notify!(on_handle_closed(heap_handle as i64));

    common::memory::heap_destroy(handle)
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
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn HeapAlloc(heap_handle: isize, flags: u32, size: usize) -> *mut u8 {
    let handle = Handle::from_raw(heap_handle);
    if size == 0 {
        // Windows HeapAlloc with size 0 returns a valid non-null pointer.
        return common::memory::heap_alloc(handle, flags, 1);
    }
    common::memory::heap_alloc(handle, flags, size)
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
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn HeapFree(heap_handle: isize, _flags: u32, ptr: *mut u8) -> WinBool {
    if ptr.is_null() {
        return WinBool::TRUE;
    }

    let handle = Handle::from_raw(heap_handle);

    unsafe { common::memory::heap_free(handle, _flags, ptr) }
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
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn HeapReAlloc(
    heap_handle: isize,
    flags: u32,
    ptr: *mut u8,
    new_size: usize,
) -> *mut u8 {
    let handle = Handle::from_raw(heap_handle);

    if ptr.is_null() {
        return common::memory::heap_alloc(handle, flags, new_size);
    }

    unsafe { common::memory::heap_realloc(handle, flags, ptr, new_size) }
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
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn HeapSize(heap_handle: isize, _flags: u32, ptr: *const u8) -> usize {
    let handle = Handle::from_raw(heap_handle);

    common::memory::heap_size(handle, _flags, ptr)
}

// ---------------------------------------------------------------------------
// VirtualAlloc / VirtualFree
// ---------------------------------------------------------------------------

/// Allocate memory in the virtual address space of the calling process.
///
/// # Arguments
/// * `address` - The desired starting address of the allocated region. If `NULL`, the system determines where to allocate the region.
/// * `size` - The size of the region in bytes. If this parameter is zero, the function fails.
/// * `alloc_type` - Allocation options. Supported flags:
///   * `MEM_COMMIT` (0x00001000): Allocate physical storage in memory or the paging file for the specified region of pages.
///     The actual allocation of memory for the pages is deferred until the pages are accessed.
///   * `MEM_RESERVE` (0x00002000): Reserve a range of the process's virtual address space without allocating any actual physical storage.
///     The reserved range cannot be used until it is committed.
/// * `protect` - Memory protection options. Supported flags:
///   * `PAGE_READWRITE` (0x04): Enables read and write access to the committed region of pages. If an attempt is made to write to a
///     page that is committed with `PAGE_READWRITE` protection, the system raises a guard page exception.
///
/// # Safety
/// The caller is responsible for ensuring that the specified address range is valid and does not overlap with any existing allocations.
/// The caller must also ensure that the allocated memory is freed using VirtualFree when it is no longer needed. Failure to do so may
/// result in memory leaks or other undefined behavior. Additionally, the caller must ensure that the `alloc_type` and `protect`
/// parameters are set to valid combinations of flags, as invalid combinations may result in undefined behavior.
/// For example, `MEM_RELEASE` cannot be used with `MEM_COMMIT` or `MEM_RESERVE`, and `PAGE_EXECUTE` cannot be used with
/// `MEM_RESERVE` alone.
///
/// # Returns
/// If the function succeeds, the return value is a pointer to the allocated memory region. If the function fails, the return value is
/// `NULL`, and extended error information should be (but currently cannot be) obtained by calling GetLastError.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn VirtualAlloc(
    address: *mut u8,
    size: usize,
    alloc_type: u32,
    protect: u32,
) -> *mut u8 {
    unsafe { common::memory::virtual_alloc(address, size, alloc_type, protect) }
}

/// Free or decommit memory in the virtual address space of the calling process.
///
/// # Arguments
/// * `address` - A pointer to the base address of the region of pages to be freed or decommitted.
///   This must be a pointer returned by VirtualAlloc.
/// * `size` - The size of the region of pages to be freed or decommitted, in bytes.
///   If `MEM_RELEASE` is specified in `free_type`, this parameter must be 0 (zero), and the
///   function will free the entire region allocated by VirtualAlloc. If `MEM_DECOMMIT` is
///   specified in `free_type`, this parameter must be greater than 0, and the function will
///   decommit the specified range of pages, making them inaccessible but keeping the reservation intact.
/// * `free_type` - Freeing options. Supported flags:
///   * `MEM_DECOMMIT` (0x00004000): Decommit the specified region of committed pages, making them
///     inaccessible and releasing the physical storage but keeping the reservation intact.
///     The function fails if any pages in the specified range are not committed.
///   * `MEM_RELEASE` (0x00008000): Release the entire region of pages allocated by VirtualAlloc,
///     starting at the specified address. The function fails if the specified address is not the
///     base address returned by VirtualAlloc or if any pages in the region are still committed.
///     `MEM_RELEASE` cannot be used with `MEM_DECOMMIT` or with a non-zero `size` parameter, as it
///     always releases the entire region allocated by VirtualAlloc.
///
/// # Safety
/// The caller is responsible for ensuring that the specified address range is valid and was allocated by VirtualAlloc.
/// The caller must also ensure that the `free_type` parameter is set to a valid combination of flags, as invalid
/// combinations may result in undefined behavior. For example, `MEM_RELEASE` cannot be used with `MEM_DECOMMIT`
/// or with a non-zero `size` parameter, as it always releases the entire region allocated by VirtualAlloc.
/// Additionally, the caller must ensure that the memory being freed or decommitted is not currently in use by
/// any threads, as accessing memory after it has been freed or decommitted may result in undefined behavior.
/// Finally, the caller must ensure that the function is not called on memory that has already been freed or decommitted,
/// as this may also result in undefined behavior.
///
/// # Returns
/// If the function succeeds, the return value is `TRUE`.
/// If the function fails, the return value is `FALSE`, and extended error information should
/// be (but currently cannot be) obtained by calling GetLastError.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn VirtualFree(
    address: *mut u8,
    _size: usize,
    free_type: u32,
) -> WinBool {
    unsafe { common::memory::virtual_free(address, _size, free_type) }
}

/// Change the protection on a region of committed pages in the virtual address space of the calling process.
///
/// # Arguments
/// * `address` - A pointer to the base address of the region of pages whose access protection attributes are to be changed.
///   This must be a pointer returned by VirtualAlloc.
/// * `size` - The size of the region whose access protection attributes are to be changed, in bytes.
///   The function rounds this value up to the next page boundary. If this parameter is zero, the function fails.
/// * `new_protect` - The memory protection option. Supported flags:
///   * `PAGE_NOACCESS` (0x01): Disables all access to the committed region of pages.
///     An attempt to read from, write to, or execute a page that is committed with `PAGE_NOACCESS` protection causes the
///     system to raise a guard page exception.
///   * `PAGE_READONLY` (0x02): Enables read-only access to the committed region of pages. An attempt to write to a page
///     that is committed with `PAGE_READONLY` protection causes the system to raise a guard page exception.
///   * `PAGE_READWRITE` (0x04): Enables read and write access to the committed region of pages. An attempt to write to
///     a page that is committed with `PAGE_READWRITE` protection causes the system to raise a guard page exception.
///   * `PAGE_EXECUTE` (0x10): Enables execute access to the committed region of pages. An attempt to read from or write
///     to a page that is committed with `PAGE_EXECUTE` protection causes the system to raise a guard page exception.
///   * `PAGE_EXECUTE_READ` (0x20): Enables execute and read access to the committed region of pages. An attempt to write
///     to a page that is committed with `PAGE_EXECUTE_READ` protection causes the system to raise a guard page exception.
///   * `PAGE_EXECUTE_READWRITE` (0x40): Enables execute, read, and write access to the committed region of pages.
///     An attempt to write to a page that is committed with `PAGE_EXECUTE_READWRITE` protection causes the system to
///     raise a guard page exception.
/// * `old_protect` - An optional pointer to a variable that receives the previous access protection of the first page in
///   the specified region of pages. If this parameter is `NULL`, the function does not return the previous access protection.
///   The function fails if this parameter is not `NULL` and the caller does not have read access to the memory pointed to
///   by this parameter.
///
/// # Safety
/// The caller is responsible for ensuring that the specified address range is valid and was allocated by VirtualAlloc.
/// The caller must also ensure that the `new_protect` parameter is set to a valid combination of flags, as invalid
/// combinations may result in undefined behavior. For example, `PAGE_EXECUTE` cannot be used with `MEM_RESERVE` alone.
/// Additionally, the caller must ensure that the memory whose protection is being changed is not currently in use by any
/// threads, as accessing memory after its protection has been changed may result in undefined behavior. Finally, the caller
/// must ensure that the function is not called on memory that has already been freed or decommitted, as this may also result
/// in undefined behavior.
///
/// # Returns
/// If the function succeeds, the return value is `TRUE`. If the function fails, the return value is `FALSE`, and extended
/// error information should be (but currently cannot be) obtained by calling GetLastError.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn VirtualProtect(
    address: *mut u8,
    size: usize,
    new_protect: u32,
    old_protect: *mut u32,
) -> WinBool {
    unsafe { common::memory::virtual_protect(address, size, new_protect, old_protect) }
}

/// Query information about a range of pages in the virtual address space of the calling process.
///
/// # Arguments
/// * `_address` - A pointer to the base address of the region of pages to be queried.
///   This can be any address in the process's virtual address space, and does not need to be the base address of an allocated region.
/// * `_buffer` - A pointer to a buffer that receives information about the specified page range.
///   The structure of this buffer is implementation-defined and may not match the MEMORY_BASIC_INFORMATION
///   structure used by the Windows API.
/// * `_length` - The size of the buffer in bytes.
///   The function fails if this parameter is smaller than the size of the information returned by the function.
///
/// # Safety
/// The caller is responsible for ensuring that the specified address range is valid and that the buffer is large
/// enough to receive the information returned by the function.
/// The caller must also ensure that the function is not called on memory that has already been freed or decommitted,
/// as this may result in undefined behavior.
/// Additionally, the caller must ensure that the buffer pointed to by `_buffer` is properly aligned for the information
/// returned by the function, as misaligned access may result in undefined behavior.
/// Finally, the caller must ensure that the function is not called concurrently from multiple threads with overlapping
/// address ranges, as this may also result in undefined behavior.
///
/// # Returns
/// The function returns the size of the information returned in bytes, or `-1` (usize::MAX) if the specified address is
/// invalid or if the buffer is too small to receive the information.
/// Extended error information should be (but currently cannot be) obtained by calling GetLastError.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn VirtualQuery(
    _address: *const u8,
    _buffer: *mut u8,
    _length: usize,
) -> usize {
    unsafe { common::memory::virtual_query(_address, _buffer, _length) }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── GetProcessHeap ──────────────────────────────────────────

    #[test]
    fn get_process_heap_returns_nonzero() {
        let h = unsafe { GetProcessHeap() };
        assert_ne!(h, 0, "GetProcessHeap should return a valid handle");
    }

    #[test]
    fn get_process_heap_is_stable() {
        let h1 = unsafe { GetProcessHeap() };
        let h2 = unsafe { GetProcessHeap() };
        assert_eq!(h1, h2, "GetProcessHeap should return the same handle");
    }

    // ── HeapCreate / HeapDestroy ────────────────────────────────

    #[test]
    fn heap_create_returns_nonzero() {
        let h = unsafe { HeapCreate(0, 0, 0) };
        assert_ne!(h, 0);
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
        let result = unsafe { HeapDestroy(0xDEAD) };
        assert!(!result.is_true());
    }

    // ── HeapAlloc / HeapFree ────────────────────────────────────

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

    // ── HeapReAlloc ─────────────────────────────────────────────

    #[test]
    fn heap_realloc_grows() {
        let heap = unsafe { GetProcessHeap() };
        let ptr = unsafe { HeapAlloc(heap, 0, 16) };
        assert!(!ptr.is_null());
        unsafe { std::ptr::write_bytes(ptr, 0x42, 16) };

        let new_ptr = unsafe { HeapReAlloc(heap, 0, ptr, 64) };
        assert!(!new_ptr.is_null());
        // Original data preserved
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
        // Original 16 bytes preserved
        let orig = unsafe { std::slice::from_raw_parts(new_ptr, 16) };
        assert!(orig.iter().all(|&b| b == 0xFF));
        // Extension zeroed
        let ext = unsafe { std::slice::from_raw_parts(new_ptr.add(16), 48) };
        assert!(
            ext.iter().all(|&b| b == 0),
            "extended region should be zeroed"
        );

        unsafe { HeapFree(heap, 0, new_ptr) };
    }

    // ── HeapSize ────────────────────────────────────────────────

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

    // ── VirtualAlloc / VirtualFree ──────────────────────────────

    #[test]
    fn virtual_alloc_commit_reserve() {
        let ptr = unsafe {
            VirtualAlloc(
                std::ptr::null_mut(),
                4096,
                common::memory::MEM_COMMIT | common::memory::MEM_RESERVE,
                common::memory::PAGE_READWRITE,
            )
        };
        assert!(!ptr.is_null());
        // Should be usable — write + read
        unsafe {
            std::ptr::write_bytes(ptr, 0xCC, 4096);
            assert_eq!(*ptr, 0xCC);
            assert_eq!(*ptr.add(4095), 0xCC);
        }
        let freed = unsafe { VirtualFree(ptr, 0, common::memory::MEM_RELEASE) };
        assert!(freed.is_true());
    }

    #[test]
    fn virtual_alloc_commit_only() {
        let ptr = unsafe {
            VirtualAlloc(
                std::ptr::null_mut(),
                8192,
                common::memory::MEM_COMMIT,
                common::memory::PAGE_READWRITE,
            )
        };
        assert!(!ptr.is_null());
        unsafe {
            *ptr = 42;
            assert_eq!(*ptr, 42);
        }
        let freed = unsafe { VirtualFree(ptr, 0, common::memory::MEM_RELEASE) };
        assert!(freed.is_true());
    }

    #[test]
    fn virtual_alloc_zero_size_fails() {
        let ptr = unsafe {
            VirtualAlloc(
                std::ptr::null_mut(),
                0,
                common::memory::MEM_COMMIT | common::memory::MEM_RESERVE,
                common::memory::PAGE_READWRITE,
            )
        };
        assert!(ptr.is_null());
    }

    #[test]
    fn virtual_alloc_invalid_type_fails() {
        let ptr = unsafe {
            VirtualAlloc(
                std::ptr::null_mut(),
                4096,
                0,
                common::memory::PAGE_READWRITE,
            )
        };
        assert!(ptr.is_null());
    }

    #[test]
    fn virtual_free_null_fails() {
        let result = unsafe { VirtualFree(std::ptr::null_mut(), 0, common::memory::MEM_RELEASE) };
        assert!(!result.is_true());
    }

    #[test]
    fn virtual_free_unknown_address_fails() {
        let result = unsafe { VirtualFree(0xDEAD_0000 as *mut u8, 0, common::memory::MEM_RELEASE) };
        assert!(!result.is_true());
    }

    #[test]
    fn virtual_alloc_large_region() {
        // 1 MB allocation
        let size = 1024 * 1024;
        let ptr = unsafe {
            VirtualAlloc(
                std::ptr::null_mut(),
                size,
                common::memory::MEM_COMMIT | common::memory::MEM_RESERVE,
                common::memory::PAGE_READWRITE,
            )
        };
        assert!(!ptr.is_null());
        // Write to first and last bytes
        unsafe {
            *ptr = 1;
            *ptr.add(size - 1) = 2;
            assert_eq!(*ptr, 1);
            assert_eq!(*ptr.add(size - 1), 2);
        }
        let freed = unsafe { VirtualFree(ptr, 0, common::memory::MEM_RELEASE) };
        assert!(freed.is_true());
    }

    #[test]
    fn virtual_alloc_page_aligned() {
        let ptr = unsafe {
            VirtualAlloc(
                std::ptr::null_mut(),
                1,
                common::memory::MEM_COMMIT | common::memory::MEM_RESERVE,
                common::memory::PAGE_READWRITE,
            )
        };
        assert!(!ptr.is_null());
        assert_eq!(
            ptr as usize % 4096,
            0,
            "VirtualAlloc should return page-aligned memory"
        );
        let freed = unsafe { VirtualFree(ptr, 0, common::memory::MEM_RELEASE) };
        assert!(freed.is_true());
    }

    // ── VirtualProtect ──────────────────────────────────────────

    #[test]
    fn virtual_protect_sets_old_protect() {
        let ptr = unsafe {
            VirtualAlloc(
                std::ptr::null_mut(),
                4096,
                common::memory::MEM_COMMIT | common::memory::MEM_RESERVE,
                common::memory::PAGE_READWRITE,
            )
        };
        assert!(!ptr.is_null());
        let mut old: u32 = 0;
        let result = unsafe { VirtualProtect(ptr, 4096, common::memory::PAGE_READONLY, &mut old) };
        assert!(result.is_true());
        assert_eq!(old, common::memory::PAGE_READONLY);
        unsafe { VirtualFree(ptr, 0, common::memory::MEM_RELEASE) };
    }

    #[test]
    fn virtual_protect_null_old_protect() {
        let ptr = unsafe {
            VirtualAlloc(
                std::ptr::null_mut(),
                4096,
                common::memory::MEM_COMMIT | common::memory::MEM_RESERVE,
                common::memory::PAGE_READWRITE,
            )
        };
        assert!(!ptr.is_null());
        let result = unsafe {
            VirtualProtect(
                ptr,
                4096,
                common::memory::PAGE_READONLY,
                std::ptr::null_mut(),
            )
        };
        assert!(result.is_true());
        unsafe { VirtualFree(ptr, 0, common::memory::MEM_RELEASE) };
    }
}
