//! msvcrt stdio exports backed by shared common helpers.

use core::ffi::{c_char, c_int};

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
/// On x86_64 targets, this function is implemented as a thunk to the real `printf` to handle
/// the cdecl variadic arguments correctly. This requires a simple naked function shim due
/// to the differences between the Windows x64 and System V AMD64 ABIs.
#[rine_dlls::implemented]
pub unsafe extern "C" fn printf() -> c_int {
    unsafe { common::printf_win64_thunk() }
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
#[rine_dlls::implemented]
pub unsafe extern "win64" fn puts(s: *const c_char) -> c_int {
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
#[rine_dlls::partial]
pub unsafe extern "win64" fn fprintf(stream: *mut u8, format: *const c_char) -> c_int {
    unsafe { common::write_format_to_stream(stream.cast(), format) }
}

/// Writes formatted wide-character output to the specified file stream.
///
/// # Arguments
/// * `stream`: A pointer to a FILE stream where the output will be written.
/// * `format`: A pointer to a null-terminated wide character string (UTF-16LE) that contains
///   the format string.
///
/// # Safety
/// * The caller must ensure that `stream` is a valid pointer to a FILE stream and that `format`
///   is a valid null-terminated wide character string.
///   Additionally, any variadic arguments must match the format specifiers in `format`.
///
/// # Returns
/// * The number of wide characters written, or a negative value if an error occurs.
///
/// # Notes
/// This function is a partial implementation. Currently, the wide format string is decoded
/// from UTF-16LE to UTF-8 and written to the stream as-is without handling any format
/// specifiers or variadic arguments. A full implementation would need to parse the wide
/// format string, process format specifiers (e.g., %s, %d), and consume variadic arguments
/// according to the Microsoft x64 calling convention.
#[rine_dlls::partial]
pub unsafe extern "win64" fn fwprintf(stream: *mut u8, format: *const u16) -> c_int {
    unsafe { common::write_wide_format_to_stream(stream.cast(), format) }
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
#[rine_dlls::partial]
pub unsafe extern "win64" fn vfprintf(
    stream: *mut u8,
    format: *const c_char,
    _args: *mut u8,
) -> c_int {
    unsafe { common::write_format_to_stream(stream.cast(), format) }
}

/// Writes a wide character to the specified file stream.
///
/// # Arguments
/// * `c`: The wide character (UTF-16 code unit) to write.
/// * `stream`: A pointer to a FILE stream where the character will be written.
///
/// # Safety
/// * The caller must ensure that `stream` is a valid pointer to a FILE stream.
///
/// # Returns
/// * On success, returns the wide character cast to `wint_t`.
/// * On error, returns WEOF (`u32::MAX`).
///
/// # Notes
/// This function is a partial implementation. Currently, the wide character is
/// encoded to UTF-8 and written directly to the file descriptor, without any
/// buffering or locale-aware transcoding. A full implementation would need to
/// handle file stream buffering, text vs binary mode, and locale-dependent
/// encoding (raw UTF-16LE bytes on Windows).
#[rine_dlls::partial]
pub unsafe extern "win64" fn fputwc(c: u16, stream: *mut u8) -> u32 {
    unsafe { common::fputwc_to_stream(c, stream.cast()) }
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
#[rine_dlls::implemented]
pub unsafe extern "win64" fn fwrite(
    ptr: *const u8,
    size: usize,
    count: usize,
    stream: *mut u8,
) -> usize {
    unsafe { common::write_buffer_to_stream(ptr.cast(), size, count, stream.cast()) }
}
