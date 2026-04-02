//! kernel32 memory functions: Heap API and VirtualAlloc/VirtualFree/VirtualProtect/VirtualQuery.

use std::alloc::Layout;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use rine_types::errors::WinBool;
use rine_types::handles::{Handle, HandleEntry, HeapState, handle_table};

// ---------------------------------------------------------------------------
// Windows constants
// ---------------------------------------------------------------------------

const HEAP_ZERO_MEMORY: u32 = 0x00000008;
#[allow(dead_code)]
const HEAP_GENERATE_EXCEPTIONS: u32 = 0x00000004;
#[allow(dead_code)]
const HEAP_NO_SERIALIZE: u32 = 0x00000001;

const MEM_COMMIT: u32 = 0x00001000;
const MEM_RESERVE: u32 = 0x00002000;
const MEM_RELEASE: u32 = 0x00008000;

const PAGE_NOACCESS: u32 = 0x01;
const PAGE_READONLY: u32 = 0x02;
const PAGE_READWRITE: u32 = 0x04;
const PAGE_EXECUTE: u32 = 0x10;
const PAGE_EXECUTE_READ: u32 = 0x20;
const PAGE_EXECUTE_READWRITE: u32 = 0x40;

// ---------------------------------------------------------------------------
// Process default heap (lazy)
// ---------------------------------------------------------------------------

static DEFAULT_HEAP: LazyLock<Handle> = LazyLock::new(|| {
    handle_table().insert(HandleEntry::Heap(HeapState {
        allocations: Mutex::new(HashMap::new()),
        flags: 0,
    }))
});

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
    DEFAULT_HEAP.as_raw()
}

/// HeapCreate — create a new private heap.
///
/// `options`: heap flags (HEAP_GENERATE_EXCEPTIONS, HEAP_NO_SERIALIZE, etc.)
/// `initial_size` / `maximum_size`: ignored — we use the Rust allocator.
#[allow(non_snake_case, clippy::missing_safety_doc)]
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
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn HeapDestroy(heap_handle: isize) -> WinBool {
    let handle = Handle::from_raw(heap_handle);
    rine_types::dev_notify!(on_handle_closed(heap_handle as i64));

    // Don't allow destroying the default process heap.
    if heap_handle == DEFAULT_HEAP.as_raw() {
        return WinBool::FALSE;
    }

    match handle_table().remove(handle) {
        Some(HandleEntry::Heap(state)) => {
            // Free all outstanding allocations.
            let allocs = state.allocations.lock().unwrap();
            for (&addr, &(size, align)) in allocs.iter() {
                if let Ok(layout) = Layout::from_size_align(size, align) {
                    unsafe { std::alloc::dealloc(addr as *mut u8, layout) };
                }
            }
            WinBool::TRUE
        }
        Some(other) => {
            // Put it back — wasn't a heap handle.
            handle_table().insert(other);
            WinBool::FALSE
        }
        None => WinBool::FALSE,
    }
}

/// HeapAlloc — allocate a block from a heap.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn HeapAlloc(heap_handle: isize, flags: u32, size: usize) -> *mut u8 {
    if size == 0 {
        // Windows HeapAlloc with size 0 returns a valid non-null pointer.
        return heap_alloc_inner(heap_handle, flags, 1);
    }
    heap_alloc_inner(heap_handle, flags, size)
}

fn heap_alloc_inner(heap_handle: isize, flags: u32, size: usize) -> *mut u8 {
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
    let handle = Handle::from_raw(heap_handle);
    handle_table().with_heap(handle, |state| {
        state
            .allocations
            .lock()
            .unwrap()
            .insert(ptr as usize, (size, align));
    });

    ptr
}

/// HeapFree — free a block allocated by HeapAlloc.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn HeapFree(heap_handle: isize, _flags: u32, ptr: *mut u8) -> WinBool {
    if ptr.is_null() {
        return WinBool::TRUE;
    }

    let handle = Handle::from_raw(heap_handle);
    let removed = handle_table().with_heap(handle, |state| {
        state.allocations.lock().unwrap().remove(&(ptr as usize))
    });

    match removed {
        Some(Some((size, align))) => {
            if let Ok(layout) = Layout::from_size_align(size, align) {
                unsafe { std::alloc::dealloc(ptr, layout) };
            }
            WinBool::TRUE
        }
        _ => WinBool::FALSE,
    }
}

/// HeapReAlloc — reallocate a block from a heap.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn HeapReAlloc(
    heap_handle: isize,
    flags: u32,
    ptr: *mut u8,
    new_size: usize,
) -> *mut u8 {
    if ptr.is_null() {
        return unsafe { HeapAlloc(heap_handle, flags, new_size) };
    }

    let handle = Handle::from_raw(heap_handle);
    let actual_new_size = if new_size == 0 { 1 } else { new_size };

    // Look up old allocation.
    let old_info = handle_table().with_heap(handle, |state| {
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
    handle_table().with_heap(handle, |state| {
        let mut allocs = state.allocations.lock().unwrap();
        allocs.remove(&(ptr as usize));
        allocs.insert(new_ptr as usize, (actual_new_size, old_align));
    });

    new_ptr
}

/// HeapSize — return the size of a heap allocation.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn HeapSize(heap_handle: isize, _flags: u32, ptr: *const u8) -> usize {
    let handle = Handle::from_raw(heap_handle);
    let result = handle_table().with_heap(handle, |state| {
        let allocs = state.allocations.lock().unwrap();
        allocs.get(&(ptr as usize)).map(|&(size, _)| size)
    });

    match result {
        Some(Some(size)) => size,
        _ => usize::MAX,
    }
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

    let prot = win_protect_to_linux(new_protect);
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

/// Translate Windows memory protection constants to Linux mprotect flags.
fn win_protect_to_linux(protect: u32) -> i32 {
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
        let ptr = unsafe { HeapAlloc(heap, HEAP_ZERO_MEMORY, 128) };
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
        let ptr = unsafe { HeapAlloc(heap, HEAP_ZERO_MEMORY, 256) };
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
        let ptr = unsafe { HeapReAlloc(heap, HEAP_ZERO_MEMORY, std::ptr::null_mut(), 64) };
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

        let new_ptr = unsafe { HeapReAlloc(heap, HEAP_ZERO_MEMORY, ptr, 64) };
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
                PAGE_READWRITE,
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
        let ptr = unsafe { VirtualAlloc(std::ptr::null_mut(), 8192, MEM_COMMIT, PAGE_READWRITE) };
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
                PAGE_READWRITE,
            )
        };
        assert!(ptr.is_null());
    }

    #[test]
    fn virtual_alloc_invalid_type_fails() {
        let ptr = unsafe { VirtualAlloc(std::ptr::null_mut(), 4096, 0, PAGE_READWRITE) };
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
                PAGE_READWRITE,
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
                PAGE_READWRITE,
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
                PAGE_READWRITE,
            )
        };
        assert!(!ptr.is_null());
        let mut old: u32 = 0;
        let result = unsafe { VirtualProtect(ptr, 4096, PAGE_READONLY, &mut old) };
        assert!(result.is_true());
        assert_eq!(old, PAGE_READONLY);
        unsafe { VirtualFree(ptr, 0, MEM_RELEASE) };
    }

    #[test]
    fn virtual_protect_null_old_protect() {
        let ptr = unsafe {
            VirtualAlloc(
                std::ptr::null_mut(),
                4096,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_READWRITE,
            )
        };
        assert!(!ptr.is_null());
        let result = unsafe { VirtualProtect(ptr, 4096, PAGE_READONLY, std::ptr::null_mut()) };
        assert!(result.is_true());
        unsafe { VirtualFree(ptr, 0, MEM_RELEASE) };
    }

    // ── win_protect_to_linux ────────────────────────────────────

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
