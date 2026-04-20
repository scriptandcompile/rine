//! msvcrt stdio exports backed by shared common helpers.

use core::ffi::{c_char, c_int, c_void};

use rine_common_msvcrt as common;

/// Writes formatted output to stdout.
///
/// # Arguments
/// * Technically, this function can take a variable number of arguments like the C `printf`,
///   but this requires a custom calling convention depending on the ABI.
///
/// # Safety
/// * The caller must ensure that the provided arguments match the format string, as with C's `printf`.
///
/// # Returns
/// * The number of characters printed, or a negative value if an error occurs.
///
/// # Notes
/// On x86 targets, this function is implemented as a thunk to the real `printf` to handle
/// the cdecl variadic arguments correctly. This requires a pretty complex naked function shim due
/// to the differences between the Windows x86 and System V x86 ABIs.
pub unsafe extern "C" fn printf() -> c_int {
    unsafe { common::printf_x86_thunk() }
}

/// Writes an ANSI string followed by a newline to stdout.
///
/// # Arguments
/// * `s`: A pointer to a null-terminated C string to be written to stdout.
///
/// # Safety
/// * The caller must ensure that `s` is a valid null-terminated C string.
///
/// # Returns
/// * A non-negative value on success, or EOF on error.
pub unsafe extern "C" fn puts(s: *const c_char) -> c_int {
    unsafe { common::puts_to_stdout(s) }
}

/// Writes formatted output to the specified file stream.
///
/// # Arguments
/// * `stream`: A pointer to a FILE stream where the output will be written.
/// * `format`: A pointer to a null-terminated C string that contains the format string.
///
/// # Safety
/// * The caller must ensure that `stream` is a valid pointer to a FILE stream and that `format`
///   is a valid null-terminated C string.
///   Additionally, any variadic arguments must match the format specifiers in `format`.
///
/// # Returns
/// * The number of characters printed, or a negative value if an error occurs.
///
/// # Notes
/// Currently, the format string is written to the stream as is without formatting the text in any way.
pub unsafe extern "C" fn fprintf(stream: *mut u8, format: *const c_char) -> c_int {
    unsafe { common::write_format_to_stream(stream.cast(), format) }
}

/// Writes formatted output to the specified file stream using a va_list of arguments.
///
/// # Arguments
/// * `stream`: A pointer to a FILE stream where the output will be written.
/// * `format`: A pointer to a null-terminated C string that contains the format string.
/// * `_args`: A pointer to a va_list of arguments to be formatted according to `format`.
///
/// # Safety
/// * The caller must ensure that `stream` is a valid pointer to a FILE stream and that `format`
///   is a valid null-terminated C string.
///   Additionally, any variadic arguments must match the format specifiers in `format`.
///
/// # Returns
/// * The number of characters printed, or a negative value if an error occurs.
///
/// # Notes
/// Currently, the format string is written to the stream as is without formatting the text in any way.
pub unsafe extern "C" fn vfprintf(stream: *mut u8, format: *const c_char, _args: *mut u8) -> c_int {
    unsafe { common::write_format_to_stream(stream.cast(), format) }
}

/// Writes data from a buffer to the specified file stream.
///
/// # Arguments
/// * `ptr`: A pointer to the buffer containing the data to be written.
/// * `size`: The size in bytes of each element to be written.
/// * `count`: The number of elements to be written.
/// * `stream`: A pointer to a FILE stream where the output will be written.
///
/// # Safety
/// * The caller must ensure that `ptr` is a valid pointer to a buffer of at least `size * count` bytes, and that `stream` is a valid pointer to a FILE stream.
///
/// # Returns
/// * The number of elements successfully written, which may be less than `count` if an error occurs.
pub unsafe extern "C" fn fwrite(
    ptr: *const c_void,
    size: usize,
    count: usize,
    stream: *const c_void,
) -> usize {
    unsafe { common::write_buffer_to_stream(ptr, size, count, stream) }
}
