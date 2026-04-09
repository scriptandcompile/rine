use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use rine_common_kernel32 as common;
use rine_types::errors::WinBool;
use rine_types::handles::{Handle, HandleEntry, handle_table};

const MEM_COMMIT: u32 = 0x0000_1000;
const MEM_RESERVE: u32 = 0x0000_2000;
const MEM_RELEASE: u32 = 0x0000_8000;

static VIRTUAL_REGIONS: LazyLock<Mutex<HashMap<usize, usize>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn GetProcessHeap() -> isize {
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
pub unsafe extern "stdcall" fn HeapCreate(
    options: u32,
    _initial_size: usize,
    _maximum_size: usize,
) -> isize {
    let heap = rine_types::handles::HeapState {
        allocations: Mutex::new(HashMap::new()),
        flags: options,
    };
    handle_table().insert(HandleEntry::Heap(heap)).as_raw()
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
pub unsafe extern "stdcall" fn HeapDestroy(heap_handle: isize) -> WinBool {
    let handle = Handle::from_raw(heap_handle);
    rine_types::dev_notify!(on_handle_closed(heap_handle as i64));

    common::memory::heap_destroy(handle)
}

/// HeapAlloc — allocate a block from a heap.
///
/// # Arguments
/// * `heap_handle` - A handle to the heap from which the memory will be allocated, returned by HeapCreate or GetProcessHeap.
/// * `flags` - Allocation options. Supported flags:
///     * `HEAP_ZERO_MEMORY` (0x00000008): If this flag is specified, the allocated memory will be initialized to zero.
/// * `size` - The number of bytes to allocate. If this parameter is zero, the function allocates the minimum possible size (1 byte).
///
/// # Returns
/// If the function succeeds, the return value is a pointer to the allocated memory block. If the function fails, the return value is `NULL`.
///
/// # Safety
/// The caller must ensure that `heap_handle` is a valid handle returned by HeapCreate or GetProcessHeap, and that the heap has not been
/// destroyed. The caller is responsible for freeing the allocated memory using HeapFree when it is no longer needed.
///
/// # Note
/// * `HEAP_NO_SERIALIZE` (0x00000001) and `HEAP_GENERATE_EXCEPTIONS` (0x00000004) are accepted but have no effect in this implementation.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn HeapAlloc(heap_handle: isize, flags: u32, size: usize) -> *mut u8 {
    let handle = Handle::from_raw(heap_handle);
    if size == 0 {
        // Windows HeapAlloc with size 0 returns a valid non-null pointer.
        return common::memory::heap_alloc(handle, flags, 1);
    }
    common::memory::heap_alloc(handle, flags, size)
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
pub unsafe extern "stdcall" fn HeapSize(heap_handle: isize, _flags: u32, ptr: *const u8) -> usize {
    let handle = Handle::from_raw(heap_handle);

    common::memory::heap_size(handle, _flags, ptr)
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
pub unsafe extern "stdcall" fn HeapFree(heap_handle: isize, _flags: u32, ptr: *mut u8) -> WinBool {
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
pub unsafe extern "stdcall" fn HeapReAlloc(
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

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn VirtualAlloc(
    address: *mut u8,
    size: usize,
    alloc_type: u32,
    protect: u32,
) -> *mut u8 {
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
    VIRTUAL_REGIONS
        .lock()
        .unwrap()
        .insert(ptr as usize, alloc_size);
    ptr
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn VirtualFree(
    address: *mut u8,
    _size: usize,
    free_type: u32,
) -> WinBool {
    if address.is_null() {
        return WinBool::FALSE;
    }

    if free_type & MEM_RELEASE != 0 {
        let region_size = match VIRTUAL_REGIONS.lock().unwrap().remove(&(address as usize)) {
            Some(s) => s,
            None => return WinBool::FALSE,
        };
        let result = unsafe { libc::munmap(address.cast(), region_size) };
        return if result == 0 {
            WinBool::TRUE
        } else {
            WinBool::FALSE
        };
    }

    WinBool::TRUE
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn VirtualProtect(
    address: *mut u8,
    size: usize,
    new_protect: u32,
    old_protect: *mut u32,
) -> WinBool {
    if !old_protect.is_null() {
        unsafe { *old_protect = new_protect };
    }

    let result = unsafe {
        libc::mprotect(
            address.cast(),
            size,
            common::memory::win_protect_to_linux(new_protect),
        )
    };
    if result == 0 {
        WinBool::TRUE
    } else {
        WinBool::FALSE
    }
}

/// Query information about a range of pages in the virtual address space of the calling process.
///
/// Stub: returns 0 (failure).
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn VirtualQuery(
    _address: *const u8,
    _buffer: *mut u8,
    _length: usize,
) -> usize {
    unsafe { common::memory::virtual_query(_address, _buffer, _length) }
}
