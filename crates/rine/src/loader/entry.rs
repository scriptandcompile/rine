//! Entry point setup & execution.
//!
//! Sets up a Windows x64-compatible stack frame and transfers control to the
//! PE's `AddressOfEntryPoint`. The PE's CRT startup code then calls our
//! reimplemented DLL functions through the Import Address Table.

use std::convert::Infallible;

use rine_types::memory::{RelativeVirtualAddress, VirtualAddress};
use thiserror::Error;
use tracing::{debug, info};

use super::memory::LoadedImage;
use crate::pe::parser::ParsedPe;

#[derive(Debug, Error)]
pub enum EntryError {
    #[error("PE has no entry point (entry RVA is 0)")]
    NoEntryPoint,

    #[error("entry point {0} is outside the loaded image (size {1:#x})")]
    OutOfBounds(VirtualAddress, usize),
}

/// Execute the loaded PE image's entry point.
///
/// This function does not return — it either transfers control to the PE
/// (which calls `ExitProcess`) or terminates with the entry point's return code.
pub fn execute(image: &LoadedImage, parsed: &ParsedPe) -> Result<Infallible, EntryError> {
    let entry_rva = parsed.pe.entry as u32;
    if entry_rva == 0 {
        return Err(EntryError::NoEntryPoint);
    }

    let entry_rva = RelativeVirtualAddress::new(entry_rva);
    let entry_va = image.entry_point(entry_rva);

    // Bounds check: entry point must fall within the loaded image.
    if entry_rva.as_usize() >= image.size() {
        return Err(EntryError::OutOfBounds(entry_va, image.size()));
    }

    info!(
        entry = format_args!("{entry_va}"),
        rva = format_args!("{entry_rva}"),
        base = format_args!("{}", image.base()),
        "transferring control to PE entry point"
    );

    // Transfer control. The trampoline sets up a Windows x64 stack frame
    // and calls the entry point. If it returns, we get the exit code from rax.
    let exit_code = unsafe { trampoline(entry_va.as_usize()) };

    debug!(exit_code, "PE entry point returned");
    std::process::exit(exit_code as i32);
}

/// Assembly trampoline that calls a PE entry point with a proper Windows x64
/// ABI stack frame.
///
/// Windows x64 calling convention requires:
/// - Stack 16-byte aligned before the CALL instruction
/// - 32 bytes of shadow space reserved by the caller above the return address
///
/// The PE entry point (`mainCRTStartup` / `WinMainCRTStartup`) takes no
/// arguments. If it returns, the value in `rax` is the exit code.
///
/// # Safety
///
/// `entry` must be a valid code address within the loaded PE image
/// with correct memory protections (executable).
#[unsafe(naked)]
unsafe extern "C" fn trampoline(_entry: usize) -> u64 {
    // SAFETY: We set up and tear down a valid stack frame. The entry
    // address has been bounds-checked by the caller.
    core::arch::naked_asm!(
        // rdi = entry point address (SysV 1st arg from Rust caller)
        "push rbp",
        "mov rbp, rsp",
        // Align stack to 16 bytes (should already be after push rbp,
        // but AND ensures it in case of unusual entry conditions).
        "and rsp, -16",
        // Reserve 32 bytes of shadow space for the Windows x64 callee.
        // After push rbp: rsp ≡ 0 mod 16.
        // sub 32: rsp ≡ 0 mod 16 (32 = 2 × 16).
        // CALL will push return addr (8 bytes) → callee sees rsp ≡ 8 mod 16. ✓
        "sub rsp, 32",
        // Call the PE entry point.
        "call rdi",
        // Entry point returned (exit code in rax).
        "mov rsp, rbp",
        "pop rbp",
        "ret",
    );
}
