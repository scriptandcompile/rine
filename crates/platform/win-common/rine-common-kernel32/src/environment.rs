use std::sync::OnceLock;

use rine_types::errors::BOOL;
use rine_types::strings::{LPCSTR, LPCWSTR};
use text::{write_cstr, write_wstr};

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
///
/// # Notes
/// Missing implementation features:
/// - The returned block is intentionally leaked for process lifetime instead of reference-counted/freeable per call.
/// - Pointer ownership validation by `FreeEnvironmentStringsA/W` is not enforced at this layer.
#[unsafe(no_mangle)]
pub unsafe fn get_environment_strings_a() -> *mut u8 {
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
///
/// # Notes
/// Missing implementation features:
/// - The returned block is intentionally leaked for process lifetime instead of reference-counted/freeable per call.
/// - Pointer ownership validation by `FreeEnvironmentStringsA/W` is not enforced at this layer.
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

/// Free a block of environment strings returned by `get_environment_strings`.
///
/// # Arguments
/// * `block`: A pointer to a block of environment strings returned by `get_environment_strings`.
///   The block is a null-terminated block of null-terminated strings.
///   The last string is followed by a null character.
///
/// # Safety
/// `block` must be either null or a pointer previously returned by `get_environment_strings`.
/// This currently validates pointer ownership but does not reclaim memory,
/// because the environment block is intentionally process-lifetime cached.
///
/// # Returns
/// If `block` is null, the return value is `BOOL::FALSE`.
/// If `block` is a pointer previously returned by `get_environment_strings`, the return value is `BOOL::TRUE`.
/// If `block` is a non-null pointer not returned by `get_environment_strings`, the return value is `BOOL::FALSE`.
/// Currently, this implementation does not set GetLastError on failure.
pub unsafe fn free_environment_strings_a(block: *mut u8) -> BOOL {
    if block.is_null() {
        return BOOL::FALSE;
    }

    let Some(cached) = ENV_BLOCK_W.get() else {
        return BOOL::FALSE;
    };

    if cached.0 as *mut u8 == block {
        BOOL::TRUE
    } else {
        BOOL::FALSE
    }
}

/// Free a block of environment strings returned by `get_environment_strings_w`.
///
/// # Arguments
/// * `block`: A pointer to a block of environment strings returned by `get_environment_strings_w`.
///   The block is a null-terminated block of null-terminated strings.
///   The last string is followed by a null character.
///
/// # Safety
/// `block` must be either null or a pointer previously returned by `get_environment_strings_w`.
/// This currently validates pointer ownership but does not reclaim memory,
/// because the environment block is intentionally process-lifetime cached.
///
/// # Returns
/// If `block` is null, the return value is `BOOL::FALSE`.
/// If `block` is a pointer previously returned by `get_environment_strings_w`, the return value is `BOOL::TRUE`.
/// If `block` is a non-null pointer not returned by `get_environment_strings_w`, the return value is `BOOL::FALSE`.
/// Currently, this implementation does not set GetLastError on failure.
///
/// # Notes
/// This function currently does not set `GetLastError` on failure.
pub unsafe fn free_environment_strings_w(block: *mut u16) -> BOOL {
    if block.is_null() {
        return BOOL::FALSE;
    }

    let Some(cached) = ENV_BLOCK_W.get() else {
        return BOOL::FALSE;
    };

    if cached.0 == block {
        BOOL::TRUE
    } else {
        BOOL::FALSE
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
pub unsafe fn get_environment_variable_a(name: LPCSTR, buffer: *mut u8, size: u32) -> u32 {
    let var_name = match name.read_string() {
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
pub unsafe fn get_environment_variable_w(name: LPCWSTR, buffer: *mut u16, size: u32) -> u32 {
    let var_name = match name.read_string() {
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
/// * `name`: A pointer to a null-terminated string that specifies the name of the environment variable.
///   The string is case-sensitive.
/// * `value`: A pointer to a null-terminated string that specifies the value of the environment variable.
///   If this parameter is NULL, the variable is deleted from the environment.
///
/// # Safety
/// * `name` must be a valid pointer to a null-terminated string.
/// * `value` must be null or a valid pointer to a null-terminated string.
/// * The function does not perform any synchronization; the caller must ensure that concurrent calls do not cause data races.
///
/// # Returns
/// If the function succeeds, the return value is TRUE.
/// If the function fails, the return value is FALSE.
pub unsafe fn set_environment_variable_a(name: LPCSTR, value: LPCSTR) -> BOOL {
    let var_name = match name.read_string() {
        Some(n) => n,
        None => return BOOL::FALSE,
    };

    let var_value = value.read_string();
    rine_types::environment::set_var(&var_name, var_value.as_deref());
    BOOL::TRUE
}

/// Set the value of an environment variable.
///
/// # Arguments
/// * `name`: A pointer to a null-terminated string that specifies the name of the environment variable.
///   The string is case-sensitive.
/// * `value`: A pointer to a null-terminated string that specifies the value of the environment variable.
///   If this parameter is NULL, the variable is deleted from the environment.
///
/// # Safety
/// * `name` must be a valid pointer to a null-terminated string.
/// * `value` must be null or a valid pointer to a null-terminated string.
/// * The function does not perform any synchronization; the caller must ensure that concurrent calls do not cause data races.
///
/// # Returns
/// If the function succeeds, the return value is TRUE.
/// If the function fails, the return value is FALSE.
pub unsafe fn set_environment_variable_w(name: LPCWSTR, value: LPCWSTR) -> BOOL {
    let var_name = match name.read_string() {
        Some(n) => n,
        None => return BOOL::FALSE,
    };

    let var_value = value.read_string();
    rine_types::environment::set_var(&var_name, var_value.as_deref());
    BOOL::TRUE
}

/// Expand environment-variable strings and replaces them with the values defined for the current user.
/// For more information, see Environment Variables.
///
/// # Arguments
/// * `src`: A pointer to a null-terminated string that contains environment-variable strings of the form `%VAR%`. The string is case-sensitive.
/// * `dst`: A pointer to a buffer that receives the expanded string. If the buffer is not large enough to hold the expanded string,
///   the function fails and returns the required buffer size, in characters, including the terminating null character.
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
/// If the buffer is too small to hold the expanded string, the return value is the size of the buffer required to hold the expanded string,
/// including the terminating null character.
/// If the function fails for any other reason, the return value is zero.
/// To get extended error information, call GetLastError.
/// Currently, this implementation does not set GetLastError on failure.
pub unsafe fn expand_environment_strings_a(src: LPCSTR, dst: *mut u8, dst_size: u32) -> u32 {
    let input = match src.read_string() {
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
/// * `src`: A pointer to a null-terminated string that contains environment-variable strings of the form `%VAR%`.
///   The string is case-sensitive.
/// * `dst`: A pointer to a buffer that receives the expanded string. If the buffer is not large enough to hold the expanded string,
///   the function fails and returns the required buffer size, in characters, including the terminating null character.
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
/// If the buffer is too small to hold the expanded string, the return value is the size of the buffer required to hold the expanded string,
/// including the terminating null character.
/// If the function fails for any other reason, the return value is zero.
/// To get extended error information, call GetLastError.
/// Currently, this implementation does not set GetLastError on failure.
pub unsafe fn expand_environment_strings_w(src: LPCWSTR, dst: *mut u16, dst_size: u32) -> u32 {
    let input = match src.read_string() {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn to_lpcstr(s: &[u8]) -> LPCSTR {
        unsafe { std::mem::transmute(s.as_ptr()) }
    }

    fn to_lpcwstr(s: &[u16]) -> LPCWSTR {
        unsafe { std::mem::transmute(s.as_ptr()) }
    }

    fn set_var(name: &str, value: Option<&str>) {
        rine_types::environment::set_var(name, value);
    }

    // ── get_environment_variable_a ─────────────────────────────────

    #[test]
    fn get_env_var_a_found() {
        set_var("RINE_K32_GET_A", Some("hello_world"));

        let name = b"RINE_K32_GET_A\0";
        let mut buf = [0u8; 64];
        let result = unsafe { get_environment_variable_a(to_lpcstr(name), buf.as_mut_ptr(), 64) };

        assert_eq!(result, 11);
        assert_eq!(&buf[..11], b"hello_world");
        assert_eq!(buf[11], 0);

        set_var("RINE_K32_GET_A", None);
    }

    #[test]
    fn get_env_var_a_not_found() {
        let name = b"RINE_K32_NONEXISTENT_XYZ\0";
        let mut buf = [0u8; 64];
        let result = unsafe { get_environment_variable_a(to_lpcstr(name), buf.as_mut_ptr(), 64) };
        assert_eq!(result, 0);
    }

    #[test]
    fn get_env_var_a_null_name() {
        let mut buf = [0u8; 64];
        let result = unsafe { get_environment_variable_a(LPCSTR::NULL, buf.as_mut_ptr(), 64) };
        assert_eq!(result, 0);
    }

    #[test]
    fn get_env_var_a_buffer_too_small() {
        set_var("RINE_K32_GET_SMALL", Some("abcdefghij"));

        let name = b"RINE_K32_GET_SMALL\0";
        let mut buf = [0u8; 4];
        let result = unsafe { get_environment_variable_a(to_lpcstr(name), buf.as_mut_ptr(), 4) };

        assert_eq!(result, 11);

        set_var("RINE_K32_GET_SMALL", None);
    }

    #[test]
    fn get_env_var_a_null_dst_returns_needed() {
        set_var("RINE_K32_GET_NDST", Some("data"));

        let name = b"RINE_K32_GET_NDST\0";
        let result =
            unsafe { get_environment_variable_a(to_lpcstr(name), std::ptr::null_mut(), 0) };

        assert_eq!(result, 5);

        set_var("RINE_K32_GET_NDST", None);
    }

    // ── get_environment_variable_w ─────────────────────────────────

    #[test]
    fn get_env_var_w_found() {
        set_var("RINE_K32_GET_W", Some("wide_val"));

        let name: Vec<u16> = "RINE_K32_GET_W\0".encode_utf16().collect();
        let mut buf = [0u16; 64];
        let result = unsafe { get_environment_variable_w(to_lpcwstr(&name), buf.as_mut_ptr(), 64) };

        assert_eq!(result, 8);

        let expected: Vec<u16> = "wide_val\0".encode_utf16().collect();
        assert_eq!(&buf[..9], &expected[..]);

        set_var("RINE_K32_GET_W", None);
    }

    #[test]
    fn get_env_var_w_not_found() {
        let name: Vec<u16> = "RINE_K32_NONEXISTENT_W\0".encode_utf16().collect();
        let mut buf = [0u16; 64];
        let result = unsafe { get_environment_variable_w(to_lpcwstr(&name), buf.as_mut_ptr(), 64) };
        assert_eq!(result, 0);
    }

    #[test]
    fn get_env_var_w_null_name() {
        let mut buf = [0u16; 64];
        let result = unsafe { get_environment_variable_w(LPCWSTR::NULL, buf.as_mut_ptr(), 64) };
        assert_eq!(result, 0);
    }

    // ── set_environment_variable_a ─────────────────────────────────

    #[test]
    fn set_env_var_a_set() {
        let name = b"RINE_K32_SET_A\0";
        let value = b"set_value\0";

        let result = unsafe { set_environment_variable_a(to_lpcstr(name), to_lpcstr(value)) };
        assert_eq!(result, BOOL::TRUE);

        assert_eq!(
            rine_types::environment::get_var("RINE_K32_SET_A"),
            Some("set_value".to_string())
        );

        set_var("RINE_K32_SET_A", None);
    }

    #[test]
    fn set_env_var_a_delete() {
        set_var("RINE_K32_SET_DEL", Some("delete_me"));

        let name = b"RINE_K32_SET_DEL\0";
        let result = unsafe { set_environment_variable_a(to_lpcstr(name), LPCSTR::NULL) };
        assert_eq!(result, BOOL::TRUE);
        assert!(rine_types::environment::get_var("RINE_K32_SET_DEL").is_none());
    }

    #[test]
    fn set_env_var_a_null_name() {
        let value = b"val\0";
        let result = unsafe { set_environment_variable_a(LPCSTR::NULL, to_lpcstr(value)) };
        assert_eq!(result, BOOL::FALSE);
    }

    // ── set_environment_variable_w ─────────────────────────────────

    #[test]
    fn set_env_var_w_set() {
        let name: Vec<u16> = "RINE_K32_SET_W\0".encode_utf16().collect();
        let value: Vec<u16> = "w_set_val\0".encode_utf16().collect();

        let result = unsafe { set_environment_variable_w(to_lpcwstr(&name), to_lpcwstr(&value)) };
        assert_eq!(result, BOOL::TRUE);

        assert_eq!(
            rine_types::environment::get_var("RINE_K32_SET_W"),
            Some("w_set_val".to_string())
        );

        set_var("RINE_K32_SET_W", None);
    }

    #[test]
    fn set_env_var_w_delete() {
        set_var("RINE_K32_SET_DEL_W", Some("delete_me_w"));

        let name: Vec<u16> = "RINE_K32_SET_DEL_W\0".encode_utf16().collect();
        let result = unsafe { set_environment_variable_w(to_lpcwstr(&name), LPCWSTR::NULL) };
        assert_eq!(result, BOOL::TRUE);
        assert!(rine_types::environment::get_var("RINE_K32_SET_DEL_W").is_none());
    }

    #[test]
    fn set_env_var_w_null_name() {
        let value: Vec<u16> = "val\0".encode_utf16().collect();
        let result = unsafe { set_environment_variable_w(LPCWSTR::NULL, to_lpcwstr(&value)) };
        assert_eq!(result, BOOL::FALSE);
    }

    // ── expand_environment_strings_a ───────────────────────────────

    #[test]
    fn expand_a_no_vars() {
        let src = b"plain_string\0";
        let mut dst = [0u8; 128];
        let result = unsafe { expand_environment_strings_a(to_lpcstr(src), dst.as_mut_ptr(), 128) };

        assert_eq!(result, 13);
        assert_eq!(&dst[..12], b"plain_string");
        assert_eq!(dst[12], 0);
    }

    #[test]
    fn expand_a_known_var() {
        set_var("RINE_K32_EXP_A", Some("replaced"));

        let src = b"before_%RINE_K32_EXP_A%_after\0";
        let mut dst = [0u8; 128];
        let result = unsafe { expand_environment_strings_a(to_lpcstr(src), dst.as_mut_ptr(), 128) };

        let expected = b"before_replaced_after";
        assert_eq!(result, expected.len() as u32 + 1);
        assert_eq!(&dst[..expected.len()], expected);
        assert_eq!(dst[expected.len()], 0);

        set_var("RINE_K32_EXP_A", None);
    }

    #[test]
    fn expand_a_unknown_var_preserved() {
        let src = b"%RINE_K32_UNKNOWN_42%\0";
        let mut dst = [0u8; 128];
        let result = unsafe { expand_environment_strings_a(to_lpcstr(src), dst.as_mut_ptr(), 128) };

        let expected = b"%RINE_K32_UNKNOWN_42%";
        assert_eq!(result, expected.len() as u32 + 1);
        assert_eq!(&dst[..expected.len()], expected);
    }

    #[test]
    fn expand_a_null_src() {
        let mut dst = [0u8; 128];
        let result = unsafe { expand_environment_strings_a(LPCSTR::NULL, dst.as_mut_ptr(), 128) };
        assert_eq!(result, 0);
    }

    #[test]
    fn expand_a_null_dst_returns_needed() {
        let src = b"hello\0";
        let result =
            unsafe { expand_environment_strings_a(to_lpcstr(src), std::ptr::null_mut(), 0) };
        assert_eq!(result, 6);
    }

    #[test]
    fn expand_a_buffer_too_small() {
        set_var("RINE_K32_EXP_SM", Some("abcdefghij"));

        let src = b"%RINE_K32_EXP_SM%\0";
        let mut dst = [0u8; 4];
        let result = unsafe { expand_environment_strings_a(to_lpcstr(src), dst.as_mut_ptr(), 4) };

        assert_eq!(result, 11);

        set_var("RINE_K32_EXP_SM", None);
    }

    #[test]
    fn expand_a_multiple_vars() {
        set_var("RINE_K32_M1", Some("X"));
        set_var("RINE_K32_M2", Some("Y"));

        let src = b"%RINE_K32_M1%-%RINE_K32_M2%\0";
        let mut dst = [0u8; 128];
        let result = unsafe { expand_environment_strings_a(to_lpcstr(src), dst.as_mut_ptr(), 128) };

        let expected = b"X-Y";
        assert_eq!(result, expected.len() as u32 + 1);
        assert_eq!(&dst[..expected.len()], expected);

        set_var("RINE_K32_M1", None);
        set_var("RINE_K32_M2", None);
    }

    // ── expand_environment_strings_w ───────────────────────────────

    #[test]
    fn get_env_var_w_buffer_too_small() {
        set_var("RINE_K32_GET_W_SM", Some("long_value_here"));

        let name: Vec<u16> = "RINE_K32_GET_W_SM\0".encode_utf16().collect();
        let mut buf = [0u16; 4];
        let result = unsafe { get_environment_variable_w(to_lpcwstr(&name), buf.as_mut_ptr(), 4) };

        assert_eq!(result, 16);

        set_var("RINE_K32_GET_W_SM", None);
    }

    #[test]
    fn expand_w_unknown_var_preserved() {
        let src: Vec<u16> = "%UNKNOWN_W_VAR%\0".encode_utf16().collect();
        let mut dst = [0u16; 128];
        let result =
            unsafe { expand_environment_strings_w(to_lpcwstr(&src), dst.as_mut_ptr(), 128) };

        let expected: Vec<u16> = "%UNKNOWN_W_VAR%".encode_utf16().collect();
        assert_eq!(result, expected.len() as u32 + 1);
        assert_eq!(&dst[..expected.len()], &expected);
    }

    #[test]
    fn expand_w_null_src() {
        let mut dst = [0u16; 128];
        let result = unsafe { expand_environment_strings_w(LPCWSTR::NULL, dst.as_mut_ptr(), 128) };
        assert_eq!(result, 0);
    }

    #[test]
    fn expand_w_null_dst_returns_needed() {
        let src: Vec<u16> = "hi\0".encode_utf16().collect();
        let result =
            unsafe { expand_environment_strings_w(to_lpcwstr(&src), std::ptr::null_mut(), 0) };
        assert_eq!(result, 3);
    }

    #[test]
    fn expand_w_unicode_chars() {
        set_var("RINE_K32_EXP_U", Some("\u{00e9}\u{20ac}"));

        let src: Vec<u16> = "%RINE_K32_EXP_U%\0".encode_utf16().collect();
        let mut dst = [0u16; 128];
        let result =
            unsafe { expand_environment_strings_w(to_lpcwstr(&src), dst.as_mut_ptr(), 128) };

        let expected: Vec<u16> = "\u{00e9}\u{20ac}".encode_utf16().collect();
        assert_eq!(result, expected.len() as u32 + 1);
        assert_eq!(&dst[..expected.len()], &expected);
        assert_eq!(dst[expected.len()], 0);

        set_var("RINE_K32_EXP_U", None);
    }

    // ── get_environment_strings / free_environment_strings ─────────

    #[test]
    fn get_env_strings_returns_non_null() {
        let ptr = unsafe { get_environment_strings_a() };
        assert!(!ptr.is_null());
    }

    #[test]
    fn get_env_strings_w_returns_non_null() {
        let ptr = unsafe { get_environment_strings_w() };
        assert!(!ptr.is_null());
    }

    #[test]
    fn get_env_strings_w_double_null_terminated() {
        let ptr = unsafe { get_environment_strings_w() };
        assert!(!ptr.is_null());

        let mut count = 0u32;
        let mut i = 0;
        loop {
            let val = unsafe { *ptr.add(i) };
            if val == 0 {
                count += 1;
                if count == 2 {
                    break;
                }
            } else {
                count = 0;
            }
            i += 1;
            // safe-guard: block should not be unreasonably large
            assert!(i < 1_000_000, "block too large, seems broken");
        }
    }

    #[test]
    fn free_env_strings_a_with_correct_ptr() {
        let ptr = unsafe { get_environment_strings_a() };
        assert_eq!(unsafe { free_environment_strings_a(ptr) }, BOOL::TRUE);
    }

    #[test]
    fn free_env_strings_a_null() {
        assert_eq!(
            unsafe { free_environment_strings_a(std::ptr::null_mut()) },
            BOOL::FALSE
        );
    }

    #[test]
    fn free_env_strings_a_wrong_ptr() {
        let mut dummy = 0u8;
        assert_eq!(
            unsafe { free_environment_strings_a(&mut dummy) },
            BOOL::FALSE
        );
    }

    #[test]
    fn free_env_strings_w_with_correct_ptr() {
        let ptr = unsafe { get_environment_strings_w() };
        assert_eq!(unsafe { free_environment_strings_w(ptr) }, BOOL::TRUE);
    }

    #[test]
    fn free_env_strings_w_null() {
        assert_eq!(
            unsafe { free_environment_strings_w(std::ptr::null_mut()) },
            BOOL::FALSE
        );
    }

    #[test]
    fn free_env_strings_w_wrong_ptr() {
        let mut dummy = 0u16;
        assert_eq!(
            unsafe { free_environment_strings_w(&mut dummy) },
            BOOL::FALSE
        );
    }
}
