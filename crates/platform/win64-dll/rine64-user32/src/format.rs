//! wsprintfA wrapper for the 64-bit user32 DLL.
//!
//! Uses the Windows x64 calling convention (extern "win64"):
//! - RCX = output (1st arg)
//! - RDX = format (2nd arg)
//! - R8  = first variadic arg
//! - R9  = second variadic arg
//! - [rsp+0x28] = third variadic arg, etc.
//!
//! The callee (rine_user32_wsprintf_a) uses the System V AMD64 calling
//! convention on Linux hosts, so we translate between the two.

unsafe extern "C" {
    fn rine_user32_wsprintf_a(
        output: *mut u8,
        format: *const u8,
        args: *const u8,
        slot_size: u32,
    ) -> i32;
}

/// Format a string with variadic arguments (ANSI version).
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
#[unsafe(naked)]
pub unsafe extern "win64" fn wsprintfA(output: *mut u8, format: *const u8) -> i32 {
    core::arch::naked_asm!(
        "mov [rsp + 0x08], rcx",
        "mov [rsp + 0x10], rdx",
        "mov [rsp + 0x18], r8",
        "mov [rsp + 0x20], r9",
        "mov rdi, [rsp + 0x08]",
        "mov rsi, [rsp + 0x10]",
        "lea rdx, [rsp + 0x18]",
        "mov ecx, 8",
        "sub rsp, 40",
        "call {func}",
        "add rsp, 40",
        "ret",
        func = sym rine_user32_wsprintf_a,
    );
}
