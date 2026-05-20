//! wsprintfA wrapper for the 32-bit user32 DLL.
//!
//! wsprintfA uses the __cdecl calling convention (unlike most user32 functions
//! which use __stdcall). The variadic arguments are accessed via the stack
//! frame pointer.

use rine_common_user32 as common;

/// Format a string with variadic arguments (ANSI version).
///
/// Uses cdecl calling convention. Arguments are on the stack after the
/// two named parameters. We read the variadic argument pointer from the
/// frame pointer register (ebp).
///
/// # Safety
/// * `output` must point to a buffer large enough for the result.
/// * `format` must be a valid null-terminated ANSI string.
/// * Variadic arguments must match the format specifiers.
///
/// # Notes
/// Not yet implemented:
/// - Field width (e.g. `%8d`)
/// - Flag characters (`-`, `+`, `0`, `#`, space)
/// - `%p` (pointer), `%o` (octal)
/// - `%*` dynamic width
/// - `%hd` (short), `%I64d` (long long)
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wsprintfA(output: *mut u8, format: *const u8) -> i32 {
    let ebp_val: *const u8;
    unsafe {
        core::arch::asm!("mov {0}, ebp", out(reg) ebp_val);
    }
    // Stack layout (cdecl):
    //   [ebp+0]  = saved ebp
    //   [ebp+4]  = return address
    //   [ebp+8]  = output  (1st param)
    //   [ebp+12] = format  (2nd param)
    //   [ebp+16] = first variadic arg
    let args = unsafe { ebp_val.add(16) };
    unsafe { common::format::rine_user32_wsprintf_a(output, format, args, 4) }
}
