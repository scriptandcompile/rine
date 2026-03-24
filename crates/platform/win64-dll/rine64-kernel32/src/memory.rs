#![allow(non_snake_case)]
//! kernel32 memory functions: VirtualProtect, VirtualQuery (minimal Phase 1).

use rine_types::errors::{self, WinBool};

/// VirtualProtect — change the protection on a region of pages.
///
/// Minimal stub: translates to mprotect. `old_protect` is written with
/// the new value (callers typically just need this out-param to not crash).
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
        errors::TRUE
    } else {
        errors::FALSE
    }
}

/// VirtualQuery — query information about a virtual memory region.
///
/// Stub: returns 0 (failure). Full implementation in Phase 2.
pub unsafe extern "win64" fn VirtualQuery(
    _address: *const u8,
    _buffer: *mut u8,
    _length: usize,
) -> usize {
    0
}

/// Translate Windows memory protection constants to Linux mprotect flags.
fn win_protect_to_linux(protect: u32) -> i32 {
    const PAGE_NOACCESS: u32 = 0x01;
    const PAGE_READONLY: u32 = 0x02;
    const PAGE_READWRITE: u32 = 0x04;
    const PAGE_EXECUTE: u32 = 0x10;
    const PAGE_EXECUTE_READ: u32 = 0x20;
    const PAGE_EXECUTE_READWRITE: u32 = 0x40;

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
