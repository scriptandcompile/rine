//! Minimal Windows threading structures (TEB) needed by PE startup code.
//!
//! The MinGW CRT accesses the Thread Environment Block via `gs:0x30` to read
//! the self-pointer and the stack base/limit. We allocate a fake TEB, fill
//! in the required fields, and set the `gs` base with `arch_prctl`.

use std::alloc::{Layout, alloc_zeroed};
use std::ptr;

use rine_runtime_core::pe::parser::PeFormat;
use thiserror::Error;
use tracing::debug;

/// Size of the fake TEB allocation. The real TEB is >0x1000 bytes;
/// we allocate a full page so anything reading further fields gets zeroes.
const TEB_SIZE: usize = 0x1000;

#[derive(Debug, Clone, Copy)]
struct TebLayout {
    stack_base: usize,
    stack_limit: usize,
    teb_self: usize,
    teb_peb: usize,
}

const TEB_LAYOUT_X64: TebLayout = TebLayout {
    stack_base: 0x08,
    stack_limit: 0x10,
    teb_self: 0x30,
    teb_peb: 0x60,
};

#[cfg(target_pointer_width = "32")]
const TEB_LAYOUT_X86: TebLayout = TebLayout {
    stack_base: 0x04,
    stack_limit: 0x08,
    teb_self: 0x18,
    teb_peb: 0x30,
};

/// Size of the fake PEB allocation.
const PEB_SIZE: usize = 0x1000;

#[derive(Debug, Error)]
pub enum ThreadingError {
    #[error("failed to allocate fake TEB")]
    TebAllocFailed,

    #[error("failed to allocate fake PEB")]
    PebAllocFailed,

    #[error("PE32 requires a 32-bit host runtime, but current host arch is `{host_arch}`")]
    HostArchMismatch { host_arch: &'static str },

    #[error("arch_prctl({op}) failed: {ret}")]
    ArchPrctlFailed { op: &'static str, ret: i64 },
}

/// Set up a minimal fake Thread Environment Block for the requested PE format.
///
/// PE32+ (x64) uses `gs`; PE32 (x86) uses `fs` and 32-bit pointer-sized fields.
///
/// # Safety
///
/// Must be called before transferring control to PE code that reads segment-based
/// TEB fields (`gs:` for PE32+, `fs:` for PE32).
/// The allocated TEB is intentionally leaked (lives for the process lifetime).
pub unsafe fn init_teb_for_format(format: PeFormat) -> Result<(), ThreadingError> {
    match format {
        PeFormat::Pe32Plus => unsafe { init_teb_x64() },
        PeFormat::Pe32 => unsafe { init_teb_x86() },
    }
}

unsafe fn init_teb_x64() -> Result<(), ThreadingError> {
    // Allocate a zeroed page for the TEB.
    let layout = Layout::from_size_align(TEB_SIZE, 16).unwrap();
    let teb = unsafe { alloc_zeroed(layout) };
    if teb.is_null() {
        return Err(ThreadingError::TebAllocFailed);
    }

    // Allocate a zeroed page for the PEB so any code dereferencing TEB.Peb
    // doesn't segfault on a null pointer.
    let peb_layout = Layout::from_size_align(PEB_SIZE, 16).unwrap();
    let peb = unsafe { alloc_zeroed(peb_layout) };
    if peb.is_null() {
        return Err(ThreadingError::PebAllocFailed);
    }

    // Fill in the self-pointer (gs:0x30 → TEB address).
    unsafe {
        let layout = TEB_LAYOUT_X64;
        // StackBase — use a reasonable value (current stack pointer + 1 MiB).
        // The CRT only uses this for the startup lock spin loop.
        let stack_base: u64;
        core::arch::asm!("mov {}, rsp", out(reg) stack_base);
        // Round up to page boundary and add generous headroom.
        let stack_base = (stack_base + 0x100000) & !0xFFF;
        ptr::write(teb.add(layout.stack_base) as *mut u64, stack_base);

        // StackLimit — low end of the stack.
        let stack_limit = stack_base.saturating_sub(0x200000); // 2 MiB below base
        ptr::write(teb.add(layout.stack_limit) as *mut u64, stack_limit);

        // Self-pointer.
        ptr::write(teb.add(layout.teb_self) as *mut u64, teb as u64);

        // PEB pointer.
        ptr::write(teb.add(layout.teb_peb) as *mut u64, peb as u64);
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
    if ret != 0 {
        return Err(ThreadingError::ArchPrctlFailed {
            op: "ARCH_SET_GS",
            ret,
        });
    }

    debug!(
        teb = format_args!("{teb:#p}"),
        peb = format_args!("{peb:#p}"),
        "initialized fake x64 TEB/PEB"
    );

    Ok(())
}

unsafe fn init_teb_x86() -> Result<(), ThreadingError> {
    if cfg!(target_pointer_width = "64") {
        return Err(ThreadingError::HostArchMismatch {
            host_arch: std::env::consts::ARCH,
        });
    }

    // This branch will be used by the dedicated 32-bit runtime binary.
    // We still build rine primarily on x86_64, where PE32 dispatch never reaches here.
    #[cfg(target_arch = "x86")]
    {
        let layout = Layout::from_size_align(TEB_SIZE, 16).unwrap();
        let teb = unsafe { alloc_zeroed(layout) };
        if teb.is_null() {
            return Err(ThreadingError::TebAllocFailed);
        }

        let peb_layout = Layout::from_size_align(PEB_SIZE, 16).unwrap();
        let peb = unsafe { alloc_zeroed(peb_layout) };
        if peb.is_null() {
            return Err(ThreadingError::PebAllocFailed);
        }

        let layout = TEB_LAYOUT_X86;

        let stack_base: u32;
        unsafe {
            core::arch::asm!("mov {}, esp", out(reg) stack_base);
        }
        let stack_base = (stack_base + 0x100000) & !0xFFF;
        let stack_limit = stack_base.saturating_sub(0x200000);

        unsafe {
            ptr::write(teb.add(layout.stack_base) as *mut u32, stack_base);
            ptr::write(teb.add(layout.stack_limit) as *mut u32, stack_limit);
            ptr::write(teb.add(layout.teb_self) as *mut u32, teb as u32);
            ptr::write(teb.add(layout.teb_peb) as *mut u32, peb as u32);
        }

        // Linux i386 does not expose ARCH_SET_FS via arch_prctl like x86_64.
        // Wiring `fs` to this TEB is done in the 32-bit runtime bring-up.
        debug!(
            teb = format_args!("{teb:#p}"),
            peb = format_args!("{peb:#p}"),
            "initialized fake x86 TEB/PEB (fs base wiring pending)"
        );

        return Ok(());
    }

    #[allow(unreachable_code)]
    Err(ThreadingError::HostArchMismatch {
        host_arch: std::env::consts::ARCH,
    })
}
