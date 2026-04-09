//! kernel32 memory functions: Heap API and VirtualAlloc/VirtualFree/VirtualProtect/VirtualQuery.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use rine_common_kernel32 as common;
use rine_types::errors::WinBool;
use rine_types::handles::{Handle, HandleEntry, HeapState, handle_table};

// ---------------------------------------------------------------------------
// Windows constants
// ---------------------------------------------------------------------------

#[allow(dead_code)]
const HEAP_GENERATE_EXCEPTIONS: u32 = 0x00000004;
#[allow(dead_code)]
const HEAP_NO_SERIALIZE: u32 = 0x00000001;

const MEM_COMMIT: u32 = 0x00001000;
const MEM_RESERVE: u32 = 0x00002000;
const MEM_RELEASE: u32 = 0x00008000;

// ---------------------------------------------------------------------------
// VirtualAlloc region tracking
// ---------------------------------------------------------------------------

static VIRTUAL_REGIONS: LazyLock<Mutex<HashMap<usize, usize>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// ---------------------------------------------------------------------------
// Heap API
// ---------------------------------------------------------------------------

/// GetProcessHeap — return the default process heap handle.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn GetProcessHeap() -> isize {
    common::memory::DEFAULT_HEAP.as_raw()
}

/// HeapCreate — create a new private heap.
///
/// # Arguments
/// * `options`: heap flags (HEAP_GENERATE_EXCEPTIONS, HEAP_NO_SERIALIZE, etc.)
/// * `initial_size` / `maximum_size`: ignored — we use the Rust allocator.
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
pub unsafe extern "win64" fn HeapCreate(
    options: u32,
    _initial_size: usize,
    _maximum_size: usize,
) -> isize {
    let heap = HeapState {
        allocations: Mutex::new(HashMap::new()),
        flags: options,
    };
    let h = handle_table().insert(HandleEntry::Heap(heap));

    rine_types::dev_notify!(on_handle_created(
        h.as_raw() as i64,
        "Heap",
        &format!("flags={options:#x}")
    ));
    h.as_raw()
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
pub unsafe extern "win64" fn HeapDestroy(heap_handle: isize) -> WinBool {
    let handle = Handle::from_raw(heap_handle);
    rine_types::dev_notify!(on_handle_closed(heap_handle as i64));

    common::memory::heap_destroy(handle)
}

/// HeapAlloc — allocate a block from a heap.
#[allow(non_snake_case, clippy::missing_safety_doc)]
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
#[allow(non_snake_case)]
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

/// HeapSize — return the size of a heap allocation.
///
/// # Arguments
/// * `heap_handle` - A handle to the heap from which the memory was allocated, returned by HeapCreate or GetProcessHeap.
/// * `_flags` - Ignored in this implementation.
/// * `ptr` - A pointer to a memory block allocated from the heap by HeapAlloc or HeapReAlloc.
///
/// # Returns
/// The size of the allocated block in bytes, or `-1` (usize::MAX) if the handle or pointer is invalid.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn HeapSize(heap_handle: isize, _flags: u32, ptr: *const u8) -> usize {
    let handle = Handle::from_raw(heap_handle);

    common::memory::heap_size(handle, _flags, ptr)
}

// ---------------------------------------------------------------------------
// VirtualAlloc / VirtualFree
// ---------------------------------------------------------------------------

/// VirtualAlloc — reserve/commit virtual memory.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn VirtualAlloc(
    address: *mut u8,
    size: usize,
    alloc_type: u32,
    protect: u32,
) -> *mut u8 {
    // We only handle MEM_COMMIT, MEM_RESERVE, or both.
    if alloc_type & (MEM_COMMIT | MEM_RESERVE) == 0 {
        return std::ptr::null_mut();
    }

    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize };
    let alloc_size = (size + page_size - 1) & !(page_size - 1);
    if alloc_size == 0 {
        return std::ptr::null_mut();
    }

    let prot = common::memory::win_protect_to_linux(protect);

    let addr_hint = if address.is_null() {
        std::ptr::null_mut()
    } else {
        address.cast()
    };

    let mut flags = libc::MAP_PRIVATE | libc::MAP_ANONYMOUS;
    if !address.is_null() {
        flags |= libc::MAP_FIXED;
    }

    let result = unsafe { libc::mmap(addr_hint, alloc_size, prot, flags, -1, 0) };

    if result == libc::MAP_FAILED {
        return std::ptr::null_mut();
    }

    let ptr = result as *mut u8;

    // Track the region so VirtualFree can unmap it.
    VIRTUAL_REGIONS
        .lock()
        .unwrap()
        .insert(ptr as usize, alloc_size);

    rine_types::dev_notify!(on_memory_allocated(
        ptr as u64,
        alloc_size as u64,
        "VirtualAlloc"
    ));

    ptr
}

/// VirtualFree — free/decommit virtual memory.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn VirtualFree(
    address: *mut u8,
    _size: usize,
    free_type: u32,
) -> WinBool {
    if address.is_null() {
        return WinBool::FALSE;
    }

    // MEM_RELEASE: release the entire region (size must be 0 on Windows,
    // but we're lenient).
    if free_type & MEM_RELEASE != 0 {
        let region_size = match VIRTUAL_REGIONS.lock().unwrap().remove(&(address as usize)) {
            Some(s) => s,
            None => return WinBool::FALSE,
        };

        let result = unsafe { libc::munmap(address.cast(), region_size) };
        if result == 0 {
            rine_types::dev_notify!(on_memory_freed(
                address as u64,
                region_size as u64,
                "VirtualFree"
            ));
        }
        return if result == 0 {
            WinBool::TRUE
        } else {
            WinBool::FALSE
        };
    }

    // MEM_DECOMMIT: just madvise DONTNEED (keeps reservation).
    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize };
    let rounded = (_size + page_size - 1) & !(page_size - 1);
    if rounded > 0 {
        unsafe { libc::madvise(address.cast(), rounded, libc::MADV_DONTNEED) };
    }
    WinBool::TRUE
}

// ---------------------------------------------------------------------------
// VirtualProtect / VirtualQuery (existing Phase 1)
// ---------------------------------------------------------------------------

/// VirtualProtect — change the protection on a region of pages.
///
/// Minimal stub: translates to mprotect. `old_protect` is written with
/// the new value (callers typically just need this out-param to not crash).
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn VirtualProtect(
    address: *mut u8,
    size: usize,
    new_protect: u32,
    old_protect: *mut u32,
) -> WinBool {
    if !old_protect.is_null() {
        unsafe { *old_protect = new_protect };
    }

    let prot = common::memory::win_protect_to_linux(new_protect);
    let result = unsafe { libc::mprotect(address.cast(), size, prot) };
    if result == 0 {
        WinBool::TRUE
    } else {
        WinBool::FALSE
    }
}

/// VirtualQuery — query information about a virtual memory region.
///
/// Stub: returns 0 (failure). Full implementation in Phase 2.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn VirtualQuery(
    _address: *const u8,
    _buffer: *mut u8,
    _length: usize,
) -> usize {
    0
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
                MEM_COMMIT | MEM_RESERVE,
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
        let freed = unsafe { VirtualFree(ptr, 0, MEM_RELEASE) };
        assert!(freed.is_true());
    }

    #[test]
    fn virtual_alloc_commit_only() {
        let ptr = unsafe {
            VirtualAlloc(
                std::ptr::null_mut(),
                8192,
                MEM_COMMIT,
                common::memory::PAGE_READWRITE,
            )
        };
        assert!(!ptr.is_null());
        unsafe {
            *ptr = 42;
            assert_eq!(*ptr, 42);
        }
        let freed = unsafe { VirtualFree(ptr, 0, MEM_RELEASE) };
        assert!(freed.is_true());
    }

    #[test]
    fn virtual_alloc_zero_size_fails() {
        let ptr = unsafe {
            VirtualAlloc(
                std::ptr::null_mut(),
                0,
                MEM_COMMIT | MEM_RESERVE,
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
        let result = unsafe { VirtualFree(std::ptr::null_mut(), 0, MEM_RELEASE) };
        assert!(!result.is_true());
    }

    #[test]
    fn virtual_free_unknown_address_fails() {
        let result = unsafe { VirtualFree(0xDEAD_0000 as *mut u8, 0, MEM_RELEASE) };
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
                MEM_COMMIT | MEM_RESERVE,
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
        let freed = unsafe { VirtualFree(ptr, 0, MEM_RELEASE) };
        assert!(freed.is_true());
    }

    #[test]
    fn virtual_alloc_page_aligned() {
        let ptr = unsafe {
            VirtualAlloc(
                std::ptr::null_mut(),
                1,
                MEM_COMMIT | MEM_RESERVE,
                common::memory::PAGE_READWRITE,
            )
        };
        assert!(!ptr.is_null());
        assert_eq!(
            ptr as usize % 4096,
            0,
            "VirtualAlloc should return page-aligned memory"
        );
        let freed = unsafe { VirtualFree(ptr, 0, MEM_RELEASE) };
        assert!(freed.is_true());
    }

    // ── VirtualProtect ──────────────────────────────────────────

    #[test]
    fn virtual_protect_sets_old_protect() {
        let ptr = unsafe {
            VirtualAlloc(
                std::ptr::null_mut(),
                4096,
                MEM_COMMIT | MEM_RESERVE,
                common::memory::PAGE_READWRITE,
            )
        };
        assert!(!ptr.is_null());
        let mut old: u32 = 0;
        let result = unsafe { VirtualProtect(ptr, 4096, common::memory::PAGE_READONLY, &mut old) };
        assert!(result.is_true());
        assert_eq!(old, common::memory::PAGE_READONLY);
        unsafe { VirtualFree(ptr, 0, MEM_RELEASE) };
    }

    #[test]
    fn virtual_protect_null_old_protect() {
        let ptr = unsafe {
            VirtualAlloc(
                std::ptr::null_mut(),
                4096,
                MEM_COMMIT | MEM_RESERVE,
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
        unsafe { VirtualFree(ptr, 0, MEM_RELEASE) };
    }
}
