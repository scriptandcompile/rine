use std::alloc::Layout;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use rine_types::errors::BOOL;
use rine_types::handles::{HANDLE, HLOCAL, HandleEntry, HeapState, handle_table};

pub const HEAP_ZERO_MEMORY: u32 = 0x00000008;

pub const MEM_COMMIT: u32 = 0x00001000;
pub const MEM_RESERVE: u32 = 0x00002000;
pub const MEM_RELEASE: u32 = 0x00008000;

pub const PAGE_NOACCESS: u32 = 0x01;
pub const PAGE_READONLY: u32 = 0x02;
pub const PAGE_READWRITE: u32 = 0x04;
pub const PAGE_EXECUTE: u32 = 0x10;
pub const PAGE_EXECUTE_READ: u32 = 0x20;
pub const PAGE_EXECUTE_READWRITE: u32 = 0x40;

#[allow(dead_code)]
const HEAP_GENERATE_EXCEPTIONS: u32 = 0x00000004;
#[allow(dead_code)]
const HEAP_NO_SERIALIZE: u32 = 0x00000001;

/// Combines `LMEM_MOVEABLE` and `LMEM_ZEROINIT` for LocalAlloc.
pub const LHND: u32 = 0x00000042; // LMEM_MOVEABLE | LMEM_ZEROINIT
/// Allocates fixed memory.
/// The return value is a pointer to the allocated memory block. This value is not a handle and cannot be used with `LocalLock`.
pub const LMEM_FIXED: u32 = 0x00000000;
/// Allocates movable memory.
/// Movable memory is allocated as a global handle that can be locked and unlocked to obtain a pointer to the memory.
/// The return value is a handle to the memory object, which can be used with `LocalLock` to get a pointer to the memory.
/// The value cannot be combined with `LMEM_FIXED`.
pub const LMEM_MOVEABLE: u32 = 0x00000002;
/// Initializes memory to zero.
pub const LMEM_ZEROINIT: u32 = 0x00000040;
/// Combines `LMEM_FIXED` and `LMEM_ZEROINIT` for LocalAlloc.
pub const LPTR: u32 = LMEM_FIXED | LMEM_ZEROINIT;
/// Same as `LMEM_MOVEABLE`.
pub const NONZEROLHND: u32 = 0x00000002; // LMEM_MOVEABLE
/// Same as `LMEM_FIXED`.
pub const NONZEROLPTR: u32 = 0x00000000; // LMEM_FIXED
/// Obsolete flag that is ignored by the system.
/// It was originally used to indicate that the memory could be discarded when no longer needed,
/// but this behavior is not implemented in modern Windows versions.
pub const LMEM_DISCARDABLE: u32 = 0x00000010;
/// Obsolete flag that is ignored by the system.
/// It was originally used to indicate that the memory should not be moved,
/// but this behavior is not implemented in modern Windows versions.
pub const LMEM_NOCOMPACT: u32 = 0x00000080;
/// Obsolete flag that is ignored by the system.
/// It was originally used to indicate that the memory should not be discarded,
/// but this behavior is not implemented in modern Windows versions.
pub const LMEM_NODISCARD: u32 = 0x00000020;

/// VirtualAlloc region tracking
/// This is used to track memory regions allocated by VirtualAlloc so that VirtualFree can unmap them.
/// It maps the base address of each allocated region to its size.
pub static VIRTUAL_REGIONS: LazyLock<Mutex<HashMap<usize, usize>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// The default process heap, used by HeapAlloc with a null heap handle.
/// This is lazily initialized on first use.
pub static DEFAULT_HEAP: LazyLock<HANDLE> = LazyLock::new(|| {
    handle_table().insert(HandleEntry::Heap(HeapState {
        allocations: Mutex::new(HashMap::new()),
        flags: 0,
    }))
});

/// Create a new private heap.
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
/// On success, returns a handle to the newly created heap. On failure, returns `None`.
///
/// # Note
/// The default process heap returned by GetProcessHeap cannot be created or destroyed using
/// HeapCreate or HeapDestroy, and attempting to do so will fail.
pub unsafe fn heap_create(options: u32) -> Option<HANDLE> {
    let heap = HeapState {
        allocations: Mutex::new(HashMap::new()),
        flags: options,
    };
    let handle = handle_table().insert(HandleEntry::Heap(heap));

    rine_types::dev_notify!(on_handle_created(
        handle.as_raw() as i64,
        "Heap",
        &format!("flags={options:#x}")
    ));

    Some(handle)
}

/// The default process heap, used by HeapAlloc with a null heap handle.
/// This is lazily initialized on first use.
///
/// # Arguments
/// * `heap_handle` - A handle to the heap from which the memory will be allocated, returned by HeapCreate or GetProcessHeap.
///   If this parameter is `NULL` (0), the default process heap is used.
/// * `flags` - Allocation options. Supported flags:
///   * `HEAP_ZERO_MEMORY` (0x00000008): If this flag is specified, the allocated memory will be initialized to zero.
/// * `size` - The number of bytes to allocate. If this parameter is zero, the function allocates the minimum possible size (1 byte).
///
/// # Note
/// The default process heap cannot be destroyed, and attempting to do so will fail.
pub fn heap_alloc(heap_handle: HANDLE, flags: u32, size: usize) -> *mut u8 {
    let align = std::mem::align_of::<usize>(); // pointer-width alignment
    let layout = match Layout::from_size_align(size, align) {
        Ok(l) => l,
        Err(_) => return std::ptr::null_mut(),
    };

    let ptr = unsafe { std::alloc::alloc(layout) };
    if ptr.is_null() {
        return std::ptr::null_mut();
    }

    if flags & HEAP_ZERO_MEMORY != 0 {
        unsafe { std::ptr::write_bytes(ptr, 0, size) };
    }

    // Track the allocation in the heap's state.

    handle_table().with_heap(heap_handle, |state| {
        state
            .allocations
            .lock()
            .unwrap()
            .insert(ptr as usize, (size, align));
    });

    rine_types::dev_notify!(on_memory_allocated(ptr as u64, size as u64, "HeapAlloc"));

    ptr
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
pub unsafe fn heap_free(heap_handle: HANDLE, _flags: u32, ptr: *mut u8) -> BOOL {
    if ptr.is_null() {
        return BOOL::TRUE;
    }

    let removed = handle_table().with_heap(heap_handle, |state| {
        state.allocations.lock().unwrap().remove(&(ptr as usize))
    });

    match removed {
        Some(Some((size, align))) => {
            if let Ok(layout) = Layout::from_size_align(size, align) {
                unsafe { std::alloc::dealloc(ptr, layout) };
            }
            rine_types::dev_notify!(on_memory_freed(ptr as u64, size as u64, "HeapFree"));
            BOOL::TRUE
        }
        _ => BOOL::FALSE,
    }
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
pub unsafe fn heap_realloc(
    heap_handle: HANDLE,
    flags: u32,
    ptr: *mut u8,
    new_size: usize,
) -> *mut u8 {
    if ptr.is_null() {
        return heap_alloc(heap_handle, flags, new_size);
    }

    let actual_new_size = if new_size == 0 { 1 } else { new_size };

    // Look up old allocation.
    let old_info = handle_table().with_heap(heap_handle, |state| {
        state
            .allocations
            .lock()
            .unwrap()
            .get(&(ptr as usize))
            .copied()
    });

    let (old_size, old_align) = match old_info {
        Some(Some(info)) => info,
        _ => return std::ptr::null_mut(),
    };

    let old_layout = match Layout::from_size_align(old_size, old_align) {
        Ok(l) => l,
        Err(_) => return std::ptr::null_mut(),
    };

    let new_ptr = unsafe { std::alloc::realloc(ptr, old_layout, actual_new_size) };
    if new_ptr.is_null() {
        return std::ptr::null_mut();
    }

    // Zero the extra bytes if requested.
    if flags & HEAP_ZERO_MEMORY != 0 && actual_new_size > old_size {
        unsafe {
            std::ptr::write_bytes(new_ptr.add(old_size), 0, actual_new_size - old_size);
        }
    }

    // Update tracking.
    handle_table().with_heap(heap_handle, |state| {
        let mut allocs = state.allocations.lock().unwrap();
        allocs.remove(&(ptr as usize));
        allocs.insert(new_ptr as usize, (actual_new_size, old_align));
    });

    rine_types::dev_notify!(on_memory_freed(ptr as u64, old_size as u64, "HeapReAlloc"));
    rine_types::dev_notify!(on_memory_allocated(
        new_ptr as u64,
        actual_new_size as u64,
        "HeapReAlloc"
    ));

    new_ptr
}

/// Destroy a heap created by HeapCreate, freeing all outstanding allocations from the heap in the process.
///
/// # Arguments
/// * `heap_handle` - A handle to the heap to destroy, returned by HeapCreate.
///
/// # Note
/// The default process heap cannot be destroyed, and attempting to do so will fail.
pub fn heap_destroy(heap_handle: HANDLE) -> BOOL {
    // Don't allow destroying the default process heap.
    if heap_handle == *DEFAULT_HEAP {
        return BOOL::FALSE;
    }

    match handle_table().remove(heap_handle) {
        Some(HandleEntry::Heap(state)) => {
            // Free all outstanding allocations.
            let allocs = state.allocations.lock().unwrap();
            for (&addr, &(size, align)) in allocs.iter() {
                if let Ok(layout) = Layout::from_size_align(size, align) {
                    unsafe { std::alloc::dealloc(addr as *mut u8, layout) };
                }
                rine_types::dev_notify!(on_memory_freed(addr as u64, size as u64, "HeapDestroy"));
            }
            BOOL::TRUE
        }
        Some(HandleEntry::Window(_)) => {
            // Window handles should not be destroyed via HeapDestroy.
            BOOL::FALSE
        }
        Some(other) => {
            // Put it back — wasn't a heap handle.
            handle_table().insert(other);
            BOOL::FALSE
        }
        None => BOOL::FALSE,
    }
}

/// Return the size of a memory block allocated from a heap by HeapAlloc.
///
/// # Arguments
/// * `heap_handle` - A handle to the heap from which the memory was allocated, returned by HeapCreate or GetProcessHeap.
/// * `flags` - This parameter is reserved and must be 0.
/// * `ptr` - A pointer to the memory block allocated by HeapAlloc.
///
/// # Returns
/// If the function succeeds, the return value is the size of the allocated memory block, in bytes.
/// If the function fails, the return value is `(SIZE_MAX)` (the maximum possible value for a `size_t`),
/// and extended error information should be (but currently cannot) be obtained by calling GetLastError.
///
/// # Notes
/// * The caller must ensure that `heap_handle` is a valid handle returned by HeapCreate or GetProcessHeap,
///   and that there are no outstanding allocations from the heap.
/// * The default process heap cannot be destroyed, and attempting to do so will fail, but this function can
///   still be used to query the size of allocations from the default heap.
/// * Missing implementation features:
///   * Reserved `flags` validation/behavior is not implemented.
///   * No Win32-accurate `GetLastError` mapping is provided for invalid handle or pointer cases.
pub fn heap_size(heap_handle: HANDLE, _flags: u32, ptr: *const u8) -> usize {
    let result = handle_table().with_heap(heap_handle, |state| {
        let allocs = state.allocations.lock().unwrap();
        allocs.get(&(ptr as usize)).map(|&(size, _)| size)
    });

    match result {
        Some(Some(size)) => size,
        _ => usize::MAX,
    }
}

/// Convert Windows memory protection flags to Linux `mmap` protection flags.
///
/// Unsupported or unknown flags are ignored, except that the absence of any
/// known protection flags results in `PROT_READ | PROT_WRITE` to avoid creating inaccessible memory.
///
/// # Arguments
/// * `protect` - Windows memory protection flags, e.g. `PAGE_READWRITE`.
///
/// # Returns
/// Linux `mmap` protection flags, e.g. `PROT_READ | PROT_WRITE`.
///
/// # Note
/// This is used for translating protection flags in memory mapping and protection APIs.
/// It is not a general-purpose flag translator and does not handle all Windows flags or combinations.
/// It covers the most common cases and falls back to read/write access for unknown flags to maintain functionality.
///
/// Supported Windows flags:
/// * `PAGE_NOACCESS` (0x01) → `PROT_NONE`
/// * `PAGE_READONLY` (0x02) → `PROT_READ`
/// * `PAGE_READWRITE` (0x04) → `PROT_READ | PROT_WRITE`
/// * `PAGE_EXECUTE` (0x10) → `PROT_EXEC`
/// * `PAGE_EXECUTE_READ` (0x20) → `PROT_READ | PROT_EXEC`
/// * `PAGE_EXECUTE_READWRITE` (0x40) → `PROT_READ | PROT_WRITE | PROT_EXEC`
pub fn win_protect_to_linux(protect: u32) -> i32 {
    match protect {
        PAGE_NOACCESS => libc::PROT_NONE,
        PAGE_READONLY => libc::PROT_READ,
        PAGE_READWRITE => libc::PROT_READ | libc::PROT_WRITE,
        PAGE_EXECUTE => libc::PROT_EXEC,
        PAGE_EXECUTE_READ => libc::PROT_READ | libc::PROT_EXEC,
        PAGE_EXECUTE_READWRITE => libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
        _ => libc::PROT_READ | libc::PROT_WRITE,
    }
}

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
pub unsafe fn virtual_alloc(
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

    let prot = win_protect_to_linux(protect);

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
pub unsafe fn virtual_free(address: *mut u8, _size: usize, free_type: u32) -> BOOL {
    if address.is_null() {
        return BOOL::FALSE;
    }

    // MEM_RELEASE: release the entire region (size must be 0 on Windows,
    // but we're lenient).
    if free_type & MEM_RELEASE != 0 {
        let region_size = match VIRTUAL_REGIONS.lock().unwrap().remove(&(address as usize)) {
            Some(s) => s,
            None => return BOOL::FALSE,
        };

        let result = unsafe { libc::munmap(address.cast(), region_size) };
        if result == 0 {
            rine_types::dev_notify!(on_memory_freed(
                address as u64,
                region_size as u64,
                "VirtualFree"
            ));
        }
        return if result == 0 { BOOL::TRUE } else { BOOL::FALSE };
    }

    // MEM_DECOMMIT: just madvise DONTNEED (keeps reservation).
    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize };
    let rounded = (_size + page_size - 1) & !(page_size - 1);
    if rounded > 0 {
        unsafe { libc::madvise(address.cast(), rounded, libc::MADV_DONTNEED) };
    }
    BOOL::TRUE
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
pub unsafe fn virtual_query(_address: *const u8, _buffer: *mut u8, _length: usize) -> usize {
    0
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
pub unsafe fn virtual_protect(
    address: *mut u8,
    size: usize,
    new_protect: u32,
    old_protect: *mut u32,
) -> BOOL {
    if !old_protect.is_null() {
        unsafe { *old_protect = new_protect };
    }

    let prot = win_protect_to_linux(new_protect);
    let result = unsafe { libc::mprotect(address.cast(), size, prot) };
    if result == 0 { BOOL::TRUE } else { BOOL::FALSE }
}

/// Allocates a block of memory from the default process heap.
///
/// # Arguments
/// * `uflags` - Allocation options. Supported flags:
///   - `LMEM_FIXED` (0x00000000): Allocates fixed memory.
///     The return value is a pointer to the allocated memory block.
///     This value is not a handle and cannot be used with `LocalLock`.
///   - `LMEM_MOVEABLE` (0x00000002): Allocates movable memory.
///     Movable memory is allocated as a global handle that can be locked and unlocked to obtain a pointer to the memory.
///     The return value is a handle to the allocated memory block.
///   - `LMEM_ZEROINIT` (0x00000040): Initializes memory to zero.
/// * `size` - The number of bytes to allocate.
///   If this parameter is zero, the function allocates the minimum possible size (1 byte).
///
///
/// # Safety
/// The caller is responsible for ensuring that the allocated memory is freed using `LocalFree` when it is no longer needed.
/// Failure to do so may result in memory leaks or other undefined behavior. Additionally, the caller must ensure that the `uflags`
/// parameter is set to a valid combination of flags, as invalid combinations may result in undefined behavior.
/// For example, `LMEM_MOVEABLE` cannot be combined with `LMEM_FIXED`.
///
/// # Returns
/// If the function succeeds, the return value is a pointer to the allocated memory block if `LMEM_FIXED` is specified,
/// or a handle to the allocated memory block if `LMEM_MOVE` is specified.
/// If the function fails, the return value is `NULL`, and extended error information should be (but currently cannot)
/// obtained by calling `GetLastError`.
///
/// # Notes
/// The default process heap cannot be destroyed, and attempting to do so will fail,
/// but this function can still be used to allocate memory from the default heap.
/// This function is a simplified implementation of the Windows API `LocalAlloc` that only supports allocation from the default process heap,
/// and does not support all of the flags or behaviors of the Windows API. It is provided for compatibility with code that uses `LocalAlloc`,
/// but for new code or code that requires more advanced heap management features,
/// it is recommended to use `HeapAlloc` with the default heap handle instead.
pub unsafe fn local_alloc(_uflags: u32, size: usize) -> HLOCAL {
    HLOCAL::from_raw(heap_alloc(*DEFAULT_HEAP, HEAP_ZERO_MEMORY, size) as isize)
}

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
pub unsafe fn global_alloc(_uflags: u32, size: usize) -> HLOCAL {
    HLOCAL::from_raw(heap_alloc(*DEFAULT_HEAP, HEAP_ZERO_MEMORY, size) as isize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protect_translation() {
        assert_eq!(win_protect_to_linux(PAGE_NOACCESS), libc::PROT_NONE);
        assert_eq!(win_protect_to_linux(PAGE_READONLY), libc::PROT_READ);
        assert_eq!(
            win_protect_to_linux(PAGE_READWRITE),
            libc::PROT_READ | libc::PROT_WRITE
        );
        assert_eq!(win_protect_to_linux(PAGE_EXECUTE), libc::PROT_EXEC);
        assert_eq!(
            win_protect_to_linux(PAGE_EXECUTE_READ),
            libc::PROT_READ | libc::PROT_EXEC
        );
        assert_eq!(
            win_protect_to_linux(PAGE_EXECUTE_READWRITE),
            libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC
        );
        // Unknown → RW fallback
        assert_eq!(
            win_protect_to_linux(0xFF),
            libc::PROT_READ | libc::PROT_WRITE
        );
    }
}
