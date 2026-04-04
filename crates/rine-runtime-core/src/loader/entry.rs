//! Entry point setup & execution.
//!
//! Sets up an architecture-appropriate Windows-compatible stack frame and
//! transfers control to the PE's `AddressOfEntryPoint`. The PE's CRT startup
//! code then calls our reimplemented DLL functions through the Import Address
//! Table.

use rine_types::memory::{RelativeVirtualAddress, VirtualAddress};
use thiserror::Error;
use tracing::{debug, info};

use super::memory::LoadedImage;
use crate::pe::parser::{ParsedPe, PeFormat};

#[derive(Debug, Error)]
pub enum EntryError {
    #[error("PE has no entry point (entry RVA is 0)")]
    NoEntryPoint,

    #[error("entry point {0} is outside the loaded image (size {1:#x})")]
    OutOfBounds(VirtualAddress, usize),

    #[error(
        "{format:?} entry trampoline requires host architecture `{required}`, current host is `{host}`"
    )]
    HostArchMismatch {
        format: PeFormat,
        required: &'static str,
        host: &'static str,
    },
}

/// Execute the loaded PE image's entry point.
///
/// Transfers control to the PE entry point. If the entry point returns
/// (rather than calling `ExitProcess`), this returns the exit code.
///
/// **Note:** Most PE executables call `ExitProcess` which invokes
/// `std::process::exit()` and never returns here.
pub fn execute(image: &LoadedImage, parsed: &ParsedPe) -> Result<i32, EntryError> {
    let entry_rva = parsed.pe.entry;
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

    let exit_code = unsafe { execute_entry(entry_va.as_usize(), parsed.format)? };

    debug!(exit_code, "PE entry point returned");
    Ok(exit_code as i32)
}

unsafe fn execute_entry(entry: usize, format: PeFormat) -> Result<u64, EntryError> {
    match format {
        PeFormat::Pe32Plus => {
            #[cfg(target_arch = "x86_64")]
            {
                Ok(unsafe { trampoline_x64(entry) })
            }

            #[cfg(not(target_arch = "x86_64"))]
            {
                Err(EntryError::HostArchMismatch {
                    format,
                    required: "x86_64",
                    host: std::env::consts::ARCH,
                })
            }
        }
        PeFormat::Pe32 => {
            #[cfg(target_arch = "x86")]
            {
                Ok(unsafe { trampoline_x86(entry) as u64 })
            }

            #[cfg(not(target_arch = "x86"))]
            {
                Err(EntryError::HostArchMismatch {
                    format,
                    required: "x86",
                    host: std::env::consts::ARCH,
                })
            }
        }
    }
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
#[cfg(target_arch = "x86_64")]
#[unsafe(naked)]
unsafe extern "C" fn trampoline_x64(_entry: usize) -> u64 {
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

/// Assembly trampoline for PE32 (x86). The PE entry point takes no arguments
/// and returns an exit code in `eax`.
#[cfg(target_arch = "x86")]
#[unsafe(naked)]
unsafe extern "C" fn trampoline_x86(_entry: usize) -> u32 {
    // SAFETY: We set up and tear down a valid x86 stack frame and transfer
    // control to the bounds-checked entry address.
    core::arch::naked_asm!(
        // cdecl first argument is at [ebp+8].
        "push ebp",
        "mov ebp, esp",
        "mov eax, [ebp + 8]",
        // Keep stack alignment conservative before entering CRT startup.
        "and esp, -16",
        "call eax",
        // Return PE exit code from eax.
        "mov esp, ebp",
        "pop ebp",
        "ret",
    );
}
