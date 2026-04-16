use std::sync::OnceLock;

use rine_types::errors::WinBool;
use rine_types::strings::{read_cstr, read_wstr, write_cstr, write_wstr};

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

/// Get the environment strings for the current process.
///
/// # Safety
/// * The function does not perform any synchronization; the caller must ensure that concurrent calls do not cause data races.
///
/// # Returns
/// If the function succeeds, the return value is a pointer to a block of environment strings for the current process.
/// The block is a null-terminated block of null-terminated strings.
/// The last string is followed by a null character.
/// The block should be freed using FreeEnvironmentStringsA when it is no longer needed.
/// Currently, this implementation returns a pointer to a static block of environment strings that is intended to live for
/// the duration of the process, so it does not actually allocate or free any memory, and the FreeEnvironmentStringsA
/// function is a no-op.
#[unsafe(no_mangle)]
pub unsafe fn get_environment_strings() -> *mut u8 {
    ENV_BLOCK_W
        .get_or_init(|| {
            let block = rine_types::environment::build_wide_block();
            let boxed = block.into_boxed_slice();
            SyncPtr(Box::into_raw(boxed) as *mut u16)
        })
        .0 as *mut u8
}

/// Get the environment strings for the current process.
///
/// # Safety
/// * The function does not perform any synchronization; the caller must ensure that concurrent calls do not cause data races.
///
/// # Returns
/// If the function succeeds, the return value is a pointer to a block of environment strings for the current process.
/// The block is a null-terminated block of null-terminated strings.
/// The last string is followed by a null character.
/// The block should be freed using FreeEnvironmentStringsW when it is no longer needed.
/// Currently, this implementation returns a pointer to a static block of environment strings that is intended to live for
/// the duration of the process, so it does not actually allocate or free any memory, and the FreeEnvironmentStringsW
/// function is a no-op.
#[unsafe(no_mangle)]
pub unsafe fn get_environment_strings_w() -> *mut u16 {
    ENV_BLOCK_W
        .get_or_init(|| {
            let block = rine_types::environment::build_wide_block();
            let boxed = block.into_boxed_slice();
            SyncPtr(Box::into_raw(boxed) as *mut u16)
        })
        .0
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
pub unsafe fn get_environment_variable_a(name: *const u8, buffer: *mut u8, size: u32) -> u32 {
    let var_name = match unsafe { read_cstr(name) } {
        Some(n) => n,
        None => return 0,
    };

    match rine_types::environment::get_var(&var_name) {
        Some(val) => unsafe { write_cstr(buffer, size, &val) },
        None => 0,
    }
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
pub unsafe fn get_environment_variable_w(name: *const u16, buffer: *mut u16, size: u32) -> u32 {
    let var_name = match unsafe { read_wstr(name) } {
        Some(n) => n,
        None => return 0,
    };

    match rine_types::environment::get_var(&var_name) {
        Some(val) => unsafe { write_wstr(buffer, size, &val) },
        None => 0,
    }
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
pub unsafe fn set_environment_variable_a(name: *const u8, value: *const u8) -> WinBool {
    let var_name = match unsafe { read_cstr(name) } {
        Some(n) => n,
        None => return WinBool::FALSE,
    };

    let var_value = unsafe { read_cstr(value) };
    rine_types::environment::set_var(&var_name, var_value.as_deref());
    WinBool::TRUE
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
pub unsafe fn set_environment_variable_w(name: *const u16, value: *const u16) -> WinBool {
    let var_name = match unsafe { read_wstr(name) } {
        Some(n) => n,
        None => return WinBool::FALSE,
    };

    let var_value = unsafe { read_wstr(value) };
    rine_types::environment::set_var(&var_name, var_value.as_deref());
    WinBool::TRUE
}

/// Expand environment-variable strings and replaces them with the values defined for the current user.
/// For more information, see Environment Variables.
///
/// # Arguments
/// * `src`: A pointer to a null-terminated string that contains environment-variable strings of the form `%VAR%`. The string is case-sensitive.
/// * `dst`: A pointer to a buffer that receives the expanded string. If the buffer is not large enough to hold the expanded string, the function fails and returns the required buffer size, in characters, including the terminating null character.
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
/// If the buffer is too small to hold the expanded string, the return value is the size of the buffer required to hold the expanded string, including the terminating null character.
/// If the function fails for any other reason, the return value is zero.
/// To get extended error information, call GetLastError.
/// Currently, this implementation does not set GetLastError on failure.
pub unsafe fn expand_environment_strings_a(src: *const u8, dst: *mut u8, dst_size: u32) -> u32 {
    let input = match unsafe { read_cstr(src) } {
        Some(s) => s,
        None => return 0,
    };

    let expanded = rine_types::environment::expand_vars(&input);
    let needed = expanded.len() as u32 + 1;

    if dst.is_null() || dst_size < needed {
        return needed;
    }

    unsafe {
        core::ptr::copy_nonoverlapping(expanded.as_ptr(), dst, expanded.len());
        *dst.add(expanded.len()) = 0;
    }
    needed
}

/// Expand environment-variable strings and replaces them with the values defined for the current user.
/// For more information, see Environment Variables.
///
/// # Arguments
/// * `src`: A pointer to a null-terminated string that contains environment-variable strings of the form `%VAR%`. The string is case-sensitive.
/// * `dst`: A pointer to a buffer that receives the expanded string. If the buffer is not large enough to hold the expanded string, the function fails and returns the required buffer size, in characters, including the terminating null character.
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
/// If the buffer is too small to hold the expanded string, the return value is the size of the buffer required to hold the expanded string, including the terminating null character.
/// If the function fails for any other reason, the return value is zero.
/// To get extended error information, call GetLastError.
/// Currently, this implementation does not set GetLastError on failure.
pub unsafe fn expand_environment_strings_w(src: *const u16, dst: *mut u16, dst_size: u32) -> u32 {
    let input = match unsafe { read_wstr(src) } {
        Some(s) => s,
        None => return 0,
    };

    let expanded = rine_types::environment::expand_vars(&input);
    let encoded: Vec<u16> = expanded.encode_utf16().collect();
    let needed = encoded.len() as u32 + 1;

    if dst.is_null() || dst_size < needed {
        return needed;
    }

    unsafe {
        core::ptr::copy_nonoverlapping(encoded.as_ptr(), dst, encoded.len());
        *dst.add(encoded.len()) = 0;
    }
    needed
}
