//! msvcrt stdio functions: printf, puts.
//!
//! `printf` requires special handling because it is variadic: Rust stable
//! does not support `extern "win64"` variadic declarations. Instead we use
//! a `#[naked]` assembly thunk that shuffles Windows x64 arguments into the
//! SysV x86_64 registers and tail-calls the host C library's `printf`.

use core::ffi::{c_char, c_int};

// Link to the host C library's printf (SysV ABI).
unsafe extern "C" {
    #[link_name = "printf"]
    fn host_printf(format: *const c_char, ...) -> c_int;
}

/// printf — Windows x64 ABI thunk forwarding to the host's SysV printf.
///
/// PE code calls this with Windows x64 convention (format in rcx, variadic
/// args in rdx/r8/r9/stack). The thunk translates to SysV x86_64 convention
/// and tail-calls libc's printf. Supports up to ~10 total arguments.
#[allow(clippy::missing_safety_doc)]
#[unsafe(naked)]
pub unsafe extern "C" fn printf() -> c_int {
    // SAFETY: naked function — all arguments are forwarded without
    // interpretation; the host libc printf handles format-string parsing.
    core::arch::naked_asm!(
        // Win64 → SysV register shuffle
        "mov rdi, rcx",           // 1st arg (format string)
        "mov rsi, rdx",           // 2nd arg
        "mov rdx, r8",            // 3rd arg
        "mov rcx, r9",            // 4th arg
        "mov r8, [rsp + 0x28]",   // 5th arg (from win64 stack, past shadow space)
        "mov r9, [rsp + 0x30]",   // 6th arg
        "xor eax, eax",           // AL = 0: no float args in XMM (SysV variadic)
        // Copy 7th-10th args from win64 stack to SysV stack positions.
        // Win64: [rsp+0x38..] → SysV: [rsp+0x08..] (overwriting shadow space)
        "mov r10, [rsp + 0x38]",
        "mov [rsp + 0x08], r10",
        "mov r10, [rsp + 0x40]",
        "mov [rsp + 0x10], r10",
        "mov r10, [rsp + 0x48]",
        "mov [rsp + 0x18], r10",
        "mov r10, [rsp + 0x50]",
        "mov [rsp + 0x20], r10",
        // Tail-call host printf (SysV ABI)
        "jmp {printf}",
        printf = sym host_printf,
    );
}

/// puts — write a string followed by a newline to stdout.
///
/// # Safety
/// `s` must be a valid, null-terminated C string.
pub unsafe extern "win64" fn puts(s: *const c_char) -> c_int {
    if s.is_null() {
        return libc::EOF;
    }
    tracing::trace!("msvcrt::puts");
    unsafe { libc::puts(s) }
}

/// fwrite — write blocks of data to a stream.
///
/// Translates the Windows CRT FILE* to a Linux fd by reading the marker
/// stored by `__iob_func`, then calls `libc::write`.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "win64" fn fwrite(
    ptr: *const u8,
    size: usize,
    count: usize,
    stream: *mut u8, // FILE*
) -> usize {
    let total = size.saturating_mul(count);
    if ptr.is_null() || stream.is_null() || total == 0 {
        return 0;
    }
    // Read the fd marker from the first 4 bytes of the fake FILE struct.
    let fd = unsafe { *(stream as *const i32) };
    let written = unsafe { libc::write(fd, ptr.cast(), total) };
    if written < 0 {
        return 0;
    }
    (written as usize) / size.max(1)
}

// Link to host vfprintf for potential future use.
#[allow(dead_code)]
unsafe extern "C" {
    #[link_name = "vfprintf"]
    fn host_vfprintf(stream: *mut libc::FILE, format: *const c_char, args: *mut u8) -> c_int;
}

/// fprintf — formatted output to a stream.
///
/// Minimal stub: writes the format string to the fd without substitution.
/// Full variadic win64 → SysV ABI translation for fprintf requires runtime
/// format string parsing, which is deferred to a later phase.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "win64" fn fprintf(
    stream: *mut u8, // FILE*
    format: *const c_char,
    // Variadic args ignored in this stub.
) -> c_int {
    if format.is_null() || stream.is_null() {
        return -1;
    }
    let fd = unsafe { *(stream as *const i32) };
    let len = unsafe { libc::strlen(format) };
    let written = unsafe { libc::write(fd, format.cast(), len) };
    if written < 0 { -1 } else { written as c_int }
}

/// vfprintf — formatted output with va_list.
///
/// Stub: just writes the format string without substitution. Full va_list
/// translation between win64 and SysV ABIs is not feasible without runtime
/// format string parsing.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "win64" fn vfprintf(
    stream: *mut u8,
    format: *const c_char,
    _args: *mut u8, // va_list
) -> c_int {
    // Best-effort: write the format string directly (no substitution).
    if format.is_null() || stream.is_null() {
        return -1;
    }
    let fd = unsafe { *(stream as *const i32) };
    let len = unsafe { libc::strlen(format) };
    let written = unsafe { libc::write(fd, format.cast(), len) };
    if written < 0 { -1 } else { written as c_int }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn puts_rejects_null() {
        // Should return EOF, not crash.
        let result = unsafe { puts(core::ptr::null()) };
        assert_eq!(result, libc::EOF);
    }

    #[test]
    fn puts_writes_string() {
        let s = c"hello from puts";
        let result = unsafe { puts(s.as_ptr()) };
        // puts returns a non-negative value on success.
        assert!(result >= 0);
    }
}
