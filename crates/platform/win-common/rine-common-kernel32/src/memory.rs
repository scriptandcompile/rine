use std::alloc::Layout;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use rine_types::errors::WinBool;
use rine_types::handles::{Handle, HandleEntry, HeapState, handle_table};

const HEAP_ZERO_MEMORY: u32 = 0x00000008;

pub const PAGE_NOACCESS: u32 = 0x01;
pub const PAGE_READONLY: u32 = 0x02;
pub const PAGE_READWRITE: u32 = 0x04;
pub const PAGE_EXECUTE: u32 = 0x10;
pub const PAGE_EXECUTE_READ: u32 = 0x20;
pub const PAGE_EXECUTE_READWRITE: u32 = 0x40;

/// The default process heap, used by HeapAlloc with a null heap handle.
/// This is lazily initialized on first use.
pub static DEFAULT_HEAP: LazyLock<Handle> = LazyLock::new(|| {
    handle_table().insert(HandleEntry::Heap(HeapState {
        allocations: Mutex::new(HashMap::new()),
        flags: 0,
    }))
});

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
pub fn heap_alloc(heap_handle: Handle, flags: u32, size: usize) -> *mut u8 {
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
pub unsafe fn heap_free(heap_handle: Handle, _flags: u32, ptr: *mut u8) -> WinBool {
    if ptr.is_null() {
        return WinBool::TRUE;
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
            WinBool::TRUE
        }
        _ => WinBool::FALSE,
    }
}

/// Destroy a heap created by HeapCreate, freeing all outstanding allocations from the heap in the process.
///
/// # Arguments
/// * `heap_handle` - A handle to the heap to destroy, returned by HeapCreate.
///
/// # Note
/// The default process heap cannot be destroyed, and attempting to do so will fail.
pub fn heap_destroy(heap_handle: Handle) -> WinBool {
    // Don't allow destroying the default process heap.
    if heap_handle == *DEFAULT_HEAP {
        return WinBool::FALSE;
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
            WinBool::TRUE
        }
        Some(HandleEntry::Window(_)) => {
            // Window handles should not be destroyed via HeapDestroy.
            WinBool::FALSE
        }
        Some(other) => {
            // Put it back — wasn't a heap handle.
            handle_table().insert(other);
            WinBool::FALSE
        }
        None => WinBool::FALSE,
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
pub fn heap_size(heap_handle: Handle, _flags: u32, ptr: *const u8) -> usize {
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
