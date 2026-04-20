//! Shared msvcrt stdio helpers used by both win32 and win64 plugin crates.

use core::ffi::{c_char, c_int, c_void};

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
compile_error!("rine-common-msvcrt::stdio supports only x86/x86_64 targets");

#[cfg(target_arch = "x86_64")]
unsafe extern "C" {
    #[link_name = "printf"]
    fn host_printf(format: *const c_char, ...) -> c_int;
}

/// x86-64 Windows uses a different calling convention for variadic functions,
/// so we need a special thunk to forward to the host libc printf.
///
/// # Safety
/// This function is a naked thunk that forwards the raw argument layout to the host libc printf.
/// It must only be called on x86-64 targets, and the caller must ensure that the arguments are
/// correctly passed according to the Windows x64 calling convention.
///
/// # Notes
/// - The first four integer arguments are passed in RCX, RDX, R8, and R9.
/// - Additional arguments are passed on the stack, starting at [RSP + 0x28].
/// - The thunk rearranges the arguments to match the host libc printf's expected layout.
/// - This thunk is necessary because the Windows x64 calling convention for variadic functions
///   differs from the System V AMD64 ABI used by most host libc implementations.
/// - On non-x86-64 targets, this function is not valid and will panic if called and should be
///   guarded by a compile-time check.
#[cfg(target_arch = "x86_64")]
#[unsafe(naked)]
pub unsafe extern "C" fn printf_win64_thunk() -> c_int {
    // SAFETY: Naked thunk forwards raw argument layout to host libc printf.
    core::arch::naked_asm!(
        "mov rdi, rcx",
        "mov rsi, rdx",
        "mov rdx, r8",
        "mov rcx, r9",
        "mov r8, [rsp + 0x28]",
        "mov r9, [rsp + 0x30]",
        "xor eax, eax",
        "mov r10, [rsp + 0x38]",
        "mov [rsp + 0x08], r10",
        "mov r10, [rsp + 0x40]",
        "mov [rsp + 0x10], r10",
        "mov r10, [rsp + 0x48]",
        "mov [rsp + 0x18], r10",
        "mov r10, [rsp + 0x50]",
        "mov [rsp + 0x20], r10",
        "jmp {printf}",
        printf = sym host_printf,
    );
}

/// On non-x86-64 targets, this function is not valid and will panic if called.
///
/// # Safety
/// This function is a placeholder for the x86-64 specific printf thunk and should never be called on non-x86-64 targets.
/// It will panic if called, so callers must ensure that it is only invoked on x86-64 architectures, typically through
/// compile-time checks or conditional compilation.
///
/// # Returns
/// This function does not return a meaningful value.
/// It will always panic if called on non-x86-64 targets.
///
/// # Notes
/// - This function exists to satisfy the type signature of the export on non-x86-64 targets, but it is not a valid
///   implementation and will panic if called.
///   It should be guarded by compile-time checks to prevent accidental invocation on unsupported architectures.
#[cfg(not(target_arch = "x86_64"))]
pub unsafe extern "C" fn printf_win64_thunk() -> c_int {
    unreachable!("printf_win64_thunk is only valid on x86_64 targets")
}

/// On non-x86 targets, this function is not valid and will panic if called.
///
/// # Safety
/// This function is a placeholder for the x86-specific printf thunk and should never be called on non-x86 targets.
/// It will panic if called, so callers must ensure that it is only invoked on x86 architectures, typically through
/// compile-time checks or conditional compilation.
///
/// # Returns
/// This function does not return a meaningful value.
/// It will always panic if called on non-x86 targets.
///
/// # Notes
/// - This function exists to satisfy the type signature of the export on non-x86 targets, but it is not a valid
///   implementation and will panic if called.
///   It should be guarded by compile-time checks to prevent accidental invocation on unsupported architectures.
#[cfg(target_arch = "x86")]
#[unsafe(naked)]
pub unsafe extern "C" fn printf_x86_thunk() -> c_int {
    // SAFETY: x86 cdecl variadic arguments are stack-based on both sides.
    core::arch::naked_asm!("jmp printf@PLT",);
}

/// On non-x86 targets, this function is not valid and will panic if called.
///
/// # Safety
/// This function is a placeholder for the x86-specific printf thunk and should never be called on non-x86 targets.
/// It will panic if called, so callers must ensure that it is only invoked on x86 architectures, typically through
/// compile-time checks or conditional compilation.
///
/// # Returns
/// This function does not return a meaningful value.
/// It will always panic if called on non-x86 targets.
///
/// # Notes
/// - This function exists to satisfy the type signature of the export on non-x86 targets, but it is not a valid
///   implementation and will panic if called.
///   It should be guarded by compile-time checks to prevent accidental invocation on unsupported architectures.
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn printf_x86_thunk() -> c_int {
    unreachable!("printf_x86_thunk is only valid on x86 targets")
}

/// Write a formatted string to a stream. This is a very minimal implementation that only supports
/// writing the format string itself, without any actual formatting or variadic argument handling.
///
/// # Arguments
/// - `stream` must be a valid pointer to a FILE-like struct (fake FILE in our case) that has a
///   valid file descriptor in its first 4 bytes.
/// - `format` must be a valid null-terminated C string.
///   The function will attempt to write this string directly to the file descriptor associated
///   with the stream, so it should not contain any format specifiers or require any formatting.
///   This is a simplified implementation that does not handle variadic arguments or actual formatting logic.
///
/// # Safety
/// - `stream` must point to a valid FILE-like struct with a valid file descriptor in its first 4 bytes.
/// - `format` must be a valid null-terminated C string.
/// - The caller must ensure that the file descriptor is valid and that writing to it is safe.
/// - This function does not perform any formatting and will write the format string as-is to the stream.
///
/// # Notes
/// - This function is a simplified implementation meant to support basic printf functionality without
///   full formatting capabilities.
///   It is primarily intended for use in the context of the MSVCRT plugin where the format string is
///   often a simple string literal without format specifiers. It does not handle variadic arguments
///   or any complex formatting logic, and it assumes that the caller is providing a well-formed format
///   string that can be written directly to the stream.
#[inline]
pub unsafe fn write_format_to_stream(stream: *const c_void, format: *const c_char) -> c_int {
    if format.is_null() || stream.is_null() {
        return -1;
    }
    // Fake FILE structs store an fd marker in their first 4 bytes.
    let fd = unsafe { *(stream as *const i32) };
    let len = unsafe { libc::strlen(format) };
    let written = unsafe { libc::write(fd, format.cast(), len) };
    if written < 0 { -1 } else { written as c_int }
}

/// Write a buffer to a stream. This is a very minimal implementation that directly writes the buffer to the
/// file descriptor associated with the stream, without any buffering or error handling.
///
/// # Arguments
/// - `ptr` must be a valid pointer to a buffer of at least `size * count` bytes.
/// - `size` is the size of each element in bytes.
/// - `count` is the number of elements to write.
/// - `stream` must be a valid pointer to a FILE-like struct (fake FILE in our case) that has a valid file
///   descriptor in its first 4 bytes.
///
/// # Safety
/// - `ptr` must point to a valid buffer of at least `size * count` bytes.
/// - `stream` must point to a valid FILE-like struct with a valid file descriptor in its first 4 bytes.
/// - The caller must ensure that the file descriptor is valid and that writing to it is safe.
/// - This function does not perform any buffering, error handling, or special handling for text vs binary mode.
///   It simply writes the raw bytes from the buffer to the file descriptor associated with the stream.
/// - The caller must ensure that the buffer and stream are valid and that the write operation is safe to perform.
///
/// # Returns
/// The number of elements successfully written, which may be less than `count` if an error occurs or if the end
/// of the file is reached.
/// Returns 0 if `ptr` or `stream` is null, or if `size * count` is 0, or if an error occurs during writing.
///
/// # Notes
/// - This function is a simplified implementation meant to support basic fwrite functionality without
///   full buffering or error handling capabilities. It is primarily intended for use in the context of
///   the MSVCRT plugin where fwrite is often used in a straightforward manner to write raw data to a stream.
#[inline]
pub unsafe fn write_buffer_to_stream(
    ptr: *const c_void,
    size: usize,
    count: usize,
    stream: *const c_void,
) -> usize {
    let total = size.saturating_mul(count);
    if ptr.is_null() || stream.is_null() || total == 0 {
        return 0;
    }
    let fd = unsafe { *(stream as *const i32) };
    let written = unsafe { libc::write(fd, ptr, total) };
    if written < 0 {
        return 0;
    }
    (written as usize) / size.max(1)
}

/// Write a null-terminated string to stdout, followed by a newline. This is a simple wrapper around libc::puts
/// that checks for null pointers and returns EOF on error, mimicking the behavior of the standard C library function.
///
/// # Arguments
/// - `s` must be a valid pointer to a null-terminated C string.
///   If `s` is null, the function will return EOF and will not attempt to write anything to stdout.
///
/// # Safety
/// - `s` must point to a valid null-terminated C string.
///   The caller must ensure that `s` is not null and that it points to a valid string in memory.
///
/// # Returns
/// - On success, returns a non-negative integer (the number of characters written, excluding the null terminator).
/// - If `s` is null, returns EOF (-1) and does not attempt to write anything to stdout.
/// - If an error occurs while writing to stdout, the behavior is implementation-defined, but this function will
///   return the result of libc::puts, which typically returns EOF.
#[inline]
pub unsafe fn puts_to_stdout(s: *const c_char) -> c_int {
    if s.is_null() {
        return libc::EOF;
    }
    unsafe { libc::puts(s) }
}
