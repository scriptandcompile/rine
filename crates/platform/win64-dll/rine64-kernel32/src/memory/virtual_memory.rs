use super::*;

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
#[rine_dlls::partial]
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
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn VirtualFree(address: *mut u8, _size: usize, free_type: u32) -> BOOL {
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
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn VirtualProtect(
    address: *mut u8,
    size: usize,
    new_protect: u32,
    old_protect: *mut u32,
) -> BOOL {
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
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn VirtualQuery(
    _address: *const u8,
    _buffer: *mut u8,
    _length: usize,
) -> usize {
    unsafe { common::memory::virtual_query(_address, _buffer, _length) }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- VirtualAlloc / VirtualFree ------------------------------------

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

    // -- VirtualProtect -------------------------------------------------

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
