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

// ---------------------------------------------------------------------------
// Process default heap (lazy)
// ---------------------------------------------------------------------------

pub static DEFAULT_HEAP: LazyLock<Handle> = LazyLock::new(|| {
    handle_table().insert(HandleEntry::Heap(HeapState {
        allocations: Mutex::new(HashMap::new()),
        flags: 0,
    }))
});

/// The default process heap, used by HeapAlloc with a null heap handle.
/// This is lazily initialized on first use.
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

/// Convert Windows memory protection flags to Linux `mmap` protection flags.
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
