//! Minimal Windows threading structures (TEB) needed by PE startup code.
//!
//! The MinGW CRT accesses the Thread Environment Block via `gs:0x30` to read
//! the self-pointer and the stack base/limit. We allocate a fake TEB, fill
//! in the required fields, and set the `gs` base with `arch_prctl`.

use std::alloc::{Layout, alloc_zeroed};
use std::ptr;

use tracing::debug;

/// Size of the fake TEB allocation. The real TEB is >0x1000 bytes;
/// we allocate a full page so anything reading further fields gets zeroes.
const TEB_SIZE: usize = 0x1000;

/// NT_TIB / TEB field offsets (x86-64).
const TIB_STACK_BASE: usize = 0x08;
const TIB_STACK_LIMIT: usize = 0x10;
const TEB_SELF: usize = 0x30;
/// PEB pointer lives at offset 0x60.
const TEB_PEB: usize = 0x60;

/// Size of the fake PEB allocation.
const PEB_SIZE: usize = 0x1000;

/// Set up a minimal fake Thread Environment Block and point the `gs`
/// segment register at it via `arch_prctl(ARCH_SET_GS, ...)`.
///
/// # Safety
///
/// Must be called before transferring control to PE code that reads `gs:`.
/// The allocated TEB is intentionally leaked (lives for the process lifetime).
pub unsafe fn init_teb() {
    // Allocate a zeroed page for the TEB.
    let layout = Layout::from_size_align(TEB_SIZE, 16).unwrap();
    let teb = unsafe { alloc_zeroed(layout) };
    assert!(!teb.is_null(), "failed to allocate fake TEB");

    // Allocate a zeroed page for the PEB so any code dereferencing TEB.Peb
    // doesn't segfault on a null pointer.
    let peb_layout = Layout::from_size_align(PEB_SIZE, 16).unwrap();
    let peb = unsafe { alloc_zeroed(peb_layout) };
    assert!(!peb.is_null(), "failed to allocate fake PEB");

    // Fill in the self-pointer (gs:0x30 → TEB address).
    unsafe {
        let _teb64 = teb as *mut u64;
        // StackBase — use a reasonable value (current stack pointer + 1 MiB).
        // The CRT only uses this for the startup lock spin loop.
        let stack_base: u64;
        core::arch::asm!("mov {}, rsp", out(reg) stack_base);
        // Round up to page boundary and add generous headroom.
        let stack_base = (stack_base + 0x100000) & !0xFFF;
        ptr::write(teb.add(TIB_STACK_BASE) as *mut u64, stack_base);

        // StackLimit — low end of the stack.
        let stack_limit = stack_base.saturating_sub(0x200000); // 2 MiB below base
        ptr::write(teb.add(TIB_STACK_LIMIT) as *mut u64, stack_limit);

        // Self-pointer.
        ptr::write(teb.add(TEB_SELF) as *mut u64, teb as u64);

        // PEB pointer.
        ptr::write(teb.add(TEB_PEB) as *mut u64, peb as u64);
    }

    // Set the gs base register. ARCH_SET_GS = 0x1001.
    const ARCH_SET_GS: i32 = 0x1001;
    let ret = unsafe {
        libc::syscall(
            libc::SYS_arch_prctl,
            ARCH_SET_GS as libc::c_ulong,
            teb as u64,
        )
    };
    assert!(ret == 0, "arch_prctl(ARCH_SET_GS) failed: {ret}");

    debug!(
        teb = format_args!("{teb:#p}"),
        peb = format_args!("{peb:#p}"),
        "initialized fake TEB/PEB"
    );
}
