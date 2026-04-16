//! kernel32 environment functions: GetEnvironmentVariableA/W,
//! SetEnvironmentVariableA/W, ExpandEnvironmentStringsA/W,
//! GetEnvironmentStringsW, FreeEnvironmentStringsW.

use std::sync::OnceLock;

use rine_common_kernel32 as common;

use rine_types::environment;
use rine_types::errors::WinBool;

/// Thin wrapper so a raw pointer can live in a `static OnceLock`.
/// Cached wide environment block for `GetEnvironmentStringsW`.
///
/// In a real Windows process this block is built at startup and freed by
/// `FreeEnvironmentStrings`. We use a `OnceLock` so the first call builds
/// the block and subsequent calls return the same pointer. The block is
/// leaked intentionally — it lives for the process lifetime.
struct SyncPtr(*mut u16);
unsafe impl Send for SyncPtr {}
unsafe impl Sync for SyncPtr {}

static ENV_BLOCK_W: OnceLock<SyncPtr> = OnceLock::new();

/// Get the value of an environment variable.
///
/// # Arguments
/// * `name`: A pointer to a null-terminated string that specifies the name of the environment variable. The string is case-sensitive.
/// * `buffer`: A pointer to a buffer that receives the value of the environment variable as a null-terminated string.
/// * `size`: The size of the buffer, in characters.
///
/// # Safety
/// * `name` must be a valid pointer to a null-terminated string.
/// * `buffer` must be a valid pointer to a buffer of at least `size` characters.
/// * The function does not perform any synchronization; the caller must ensure that concurrent calls do not cause data races.
///
/// # Returns
/// If the function succeeds, the return value is the number of characters stored in the buffer,
/// not including the terminating null character.
/// If the buffer is too small to hold the value, the return value is the size of the buffer required to hold the value,
/// including the terminating null character.
/// If the specified environment variable is not found, the return value is zero.
/// If the function fails for any other reason, the return value is zero.
/// To get extended error information, call GetLastError.
/// Currently, this implementation does not set GetLastError on failure.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetEnvironmentVariableA(
    name: *const u8,
    buffer: *mut u8,
    size: u32,
) -> u32 {
    unsafe { common::environment::get_environment_variable_a(name, buffer, size) }
}

/// Get the value of an environment variable.
///
/// # Arguments
/// * `name`: A pointer to a null-terminated string that specifies the name of the environment variable. The string is case-sensitive.
/// * `buffer`: A pointer to a buffer that receives the value of the environment variable as a null-terminated string.
/// * `size`: The size of the buffer, in characters.
///
/// # Safety
/// * `name` must be a valid pointer to a null-terminated string.
/// * `buffer` must be a valid pointer to a buffer of at least `size` characters.
/// * The function does not perform any synchronization; the caller must ensure that concurrent calls do not cause data races.
///
/// # Returns
/// If the function succeeds, the return value is the number of characters stored in the buffer,
/// not including the terminating null character.
/// If the buffer is too small to hold the value, the return value is the size of the buffer required to hold the value,
/// including the terminating null character.
/// If the specified environment variable is not found, the return value is zero.
/// If the function fails for any other reason, the return value is zero.
/// To get extended error information, call GetLastError.
/// Currently, this implementation does not set GetLastError on failure.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetEnvironmentVariableW(
    name: *const u16,
    buffer: *mut u16,
    size: u32,
) -> u32 {
    unsafe { common::environment::get_environment_variable_w(name, buffer, size) }
}

/// Set the value of an environment variable.
///
/// # Arguments
/// * `name`: A pointer to a null-terminated string that specifies the name of the environment variable. The string is case-sensitive.
/// * `value`: A pointer to a null-terminated string that specifies the value of the environment variable. If this parameter is NULL, the variable is deleted from the environment.
///
/// # Safety
/// * `name` must be a valid pointer to a null-terminated string.
/// * `value` must be null or a valid pointer to a null-terminated string.
/// * The function does not perform any synchronization; the caller must ensure that concurrent calls do not cause data races.
///
/// # Returns
/// If the function succeeds, the return value is TRUE.
/// If the function fails, the return value is FALSE.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn SetEnvironmentVariableA(name: *const u8, value: *const u8) -> WinBool {
    unsafe { common::environment::set_environment_variable_a(name, value) }
}

/// Set the value of an environment variable.
///
/// # Arguments
/// * `name`: A pointer to a null-terminated string that specifies the name of the environment variable. The string is case-sensitive.
/// * `value`: A pointer to a null-terminated string that specifies the value of the environment variable. If this parameter is NULL, the variable is deleted from the environment.
///
/// # Safety
/// * `name` must be a valid pointer to a null-terminated string.
/// * `value` must be null or a valid pointer to a null-terminated string.
/// * The function does not perform any synchronization; the caller must ensure that concurrent calls do not cause data races.
///
/// # Returns
/// If the function succeeds, the return value is TRUE.
/// If the function fails, the return value is FALSE.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn SetEnvironmentVariableW(
    name: *const u16,
    value: *const u16,
) -> WinBool {
    unsafe { common::environment::set_environment_variable_w(name, value) }
}

/// Expand environment-variable strings and replaces them with the values defined for the current user.
/// For more information, see Environment Variables.
///
/// # Arguments
/// * `src`: A pointer to a null-terminated string that contains environment-variable strings of the form `%VAR%`.
///   The string is case-sensitive.
/// * `dst`: A pointer to a buffer that receives the expanded string.
///   If the buffer is not large enough to hold the expanded string, the function fails and returns the
///   required buffer size, in characters, including the terminating null character.
/// * `dst_size`: The size of the buffer pointed to by `dst`, in characters.
///
/// # Safety
/// * `src` must be a valid pointer to a null-terminated string.
/// * `dst` must be a valid pointer to a buffer of at least `dst_size` characters.
/// * The function does not perform any synchronization; the caller must ensure that concurrent calls do not cause data races.
///
/// # Returns
/// If the function succeeds, the return value is the number of characters stored in the buffer,
/// not including the terminating null character.
/// If the buffer is too small to hold the expanded string, the return value is the size of the buffer required to hold
/// the expanded string, including the terminating null character.
/// If the function fails for any other reason, the return value is zero.
/// To get extended error information, call GetLastError.
/// Currently, this implementation does not set GetLastError on failure.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn ExpandEnvironmentStringsA(
    src: *const u8,
    dst: *mut u8,
    dst_size: u32,
) -> u32 {
    unsafe { common::environment::expand_environment_strings_a(src, dst, dst_size) }
}

/// Expand environment-variable strings and replaces them with the values defined for the current user.
/// For more information, see Environment Variables.
///
/// # Arguments
/// * `src`: A pointer to a null-terminated string that contains environment-variable strings of the form `%VAR%`.
///   The string is case-sensitive.
/// * `dst`: A pointer to a buffer that receives the expanded string.
///   If the buffer is not large enough to hold the expanded string, the function fails and returns the
///   required buffer size, in characters, including the terminating null character.
/// * `dst_size`: The size of the buffer pointed to by `dst`, in characters.
///
/// # Safety
/// * `src` must be a valid pointer to a null-terminated string.
/// * `dst` must be a valid pointer to a buffer of at least `dst_size` characters.
/// * The function does not perform any synchronization; the caller must ensure that concurrent calls do not cause data races.
///
/// # Returns
/// If the function succeeds, the return value is the number of characters stored in the buffer,
/// not including the terminating null character.
/// If the buffer is too small to hold the expanded string, the return value is the size of the buffer required to hold
/// the expanded string, including the terminating null character.
/// If the function fails for any other reason, the return value is zero.
/// To get extended error information, call GetLastError.
/// Currently, this implementation does not set GetLastError on failure.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn ExpandEnvironmentStringsW(
    src: *const u16,
    dst: *mut u16,
    dst_size: u32,
) -> u32 {
    unsafe { common::environment::expand_environment_strings_w(src, dst, dst_size) }
}

/// GetEnvironmentStringsW — return a pointer to the wide environment block.
///
/// The returned pointer is a null-separated, double-null terminated block
/// of `NAME=value` entries. The caller is expected to free it with
/// `FreeEnvironmentStringsW`, but our implementation leaks intentionally.
///
/// # Safety
/// The returned pointer is valid for the process lifetime.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetEnvironmentStringsW() -> *mut u16 {
    ENV_BLOCK_W
        .get_or_init(|| {
            let block = environment::build_wide_block();
            let boxed = block.into_boxed_slice();
            SyncPtr(Box::into_raw(boxed) as *mut u16)
        })
        .0
}

/// Free a block of environment strings returned by the GetEnvironmentStringsW function.
///
/// # Arguments
/// * `_block`: A pointer to the environment block returned by the GetEnvironmentStringsW function. This parameter must not be NULL.
///   Currently, this implementation does not actually free any memory, as the environment block is stored in a static variable
///   and is intended to live for the duration of the program. This stub currently always returns `WinBool::TRUE` and does not perform
///   any error checking, but in a more complete implementation, it should check if the provided pointer matches the one stored in
///   `ENV_BLOCK_W` and return `WinBool::FALSE` if it does not, or if the pointer is NULL.
///
/// # Safety
/// * `_block` must be a valid pointer returned by GetEnvironmentStringsW and must not be NULL.
/// * The function does not perform any synchronization; the caller must ensure that concurrent calls do not cause data races.
///
/// # Returns
/// If the function succeeds, the return value is TRUE.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn FreeEnvironmentStringsW(_block: *mut u16) -> WinBool {
    // No-op: the block is leaked for the process lifetime.
    WinBool::TRUE
}
