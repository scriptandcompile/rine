//! kernel32 environment functions: GetEnvironmentVariableA/W,
//! SetEnvironmentVariableA/W, ExpandEnvironmentStringsA/W,
//! GetEnvironmentStringsW, FreeEnvironmentStringsW.

use std::sync::OnceLock;

use rine_types::environment;
use rine_types::errors::WinBool;
use rine_types::strings::{read_cstr, read_wstr, write_cstr, write_wstr};
use tracing::debug;

// ---------------------------------------------------------------------------
// GetEnvironmentVariableA / W
// ---------------------------------------------------------------------------

/// GetEnvironmentVariableA — retrieve an environment variable (ANSI).
///
/// Returns the number of characters copied (excluding null terminator),
/// or the required buffer size (including null terminator) if the buffer
/// is too small. Returns 0 if the variable is not found.
///
/// # Safety
/// `name` must be a valid null-terminated ANSI string.
/// `buffer` must point to at least `size` writable bytes, or be null.
#[allow(non_snake_case)]
pub unsafe extern "win64" fn GetEnvironmentVariableA(
    name: *const u8,
    buffer: *mut u8,
    size: u32,
) -> u32 {
    let var_name = match unsafe { read_cstr(name) } {
        Some(n) => n,
        None => return 0,
    };

    debug!(name = %var_name, "GetEnvironmentVariableA");

    match environment::get_var(&var_name) {
        Some(val) => unsafe { write_cstr(buffer, size, &val) },
        None => 0,
    }
}

/// GetEnvironmentVariableW — retrieve an environment variable (wide).
///
/// # Safety
/// `name` must be a valid null-terminated UTF-16LE string.
/// `buffer` must point to at least `size` writable u16 elements, or be null.
#[allow(non_snake_case)]
pub unsafe extern "win64" fn GetEnvironmentVariableW(
    name: *const u16,
    buffer: *mut u16,
    size: u32,
) -> u32 {
    let var_name = match unsafe { read_wstr(name) } {
        Some(n) => n,
        None => return 0,
    };

    debug!(name = %var_name, "GetEnvironmentVariableW");

    match environment::get_var(&var_name) {
        Some(val) => unsafe { write_wstr(buffer, size, &val) },
        None => 0,
    }
}

// ---------------------------------------------------------------------------
// SetEnvironmentVariableA / W
// ---------------------------------------------------------------------------

/// SetEnvironmentVariableA — set or delete an environment variable (ANSI).
///
/// If `value` is NULL the variable is deleted. Returns TRUE on success.
///
/// # Safety
/// `name` must be a valid null-terminated ANSI string.
/// `value` must be null or a valid null-terminated ANSI string.
#[allow(non_snake_case)]
pub unsafe extern "win64" fn SetEnvironmentVariableA(name: *const u8, value: *const u8) -> WinBool {
    let var_name = match unsafe { read_cstr(name) } {
        Some(n) => n,
        None => return WinBool::FALSE,
    };
    let var_value = unsafe { read_cstr(value) };

    debug!(name = %var_name, value = ?var_value, "SetEnvironmentVariableA");
    environment::set_var(&var_name, var_value.as_deref());
    WinBool::TRUE
}

/// SetEnvironmentVariableW — set or delete an environment variable (wide).
///
/// # Safety
/// `name` must be a valid null-terminated UTF-16LE string.
/// `value` must be null or a valid null-terminated UTF-16LE string.
#[allow(non_snake_case)]
pub unsafe extern "win64" fn SetEnvironmentVariableW(
    name: *const u16,
    value: *const u16,
) -> WinBool {
    let var_name = match unsafe { read_wstr(name) } {
        Some(n) => n,
        None => return WinBool::FALSE,
    };
    let var_value = unsafe { read_wstr(value) };

    debug!(name = %var_name, value = ?var_value, "SetEnvironmentVariableW");
    environment::set_var(&var_name, var_value.as_deref());
    WinBool::TRUE
}

// ---------------------------------------------------------------------------
// ExpandEnvironmentStringsA / W
// ---------------------------------------------------------------------------

/// ExpandEnvironmentStringsA — expand `%VAR%` references in a string (ANSI).
///
/// Returns the number of characters in the expanded string (including the
/// null terminator). If `dst_size` is 0 or `dst` is NULL, the function
/// returns the required buffer size.
///
/// # Safety
/// `src` must be a valid null-terminated ANSI string.
/// `dst` must be null or point to at least `dst_size` writable bytes.
#[allow(non_snake_case)]
pub unsafe extern "win64" fn ExpandEnvironmentStringsA(
    src: *const u8,
    dst: *mut u8,
    dst_size: u32,
) -> u32 {
    let input = match unsafe { read_cstr(src) } {
        Some(s) => s,
        None => return 0,
    };

    let expanded = environment::expand_vars(&input);
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

/// ExpandEnvironmentStringsW — expand `%VAR%` references in a string (wide).
///
/// # Safety
/// `src` must be a valid null-terminated UTF-16LE string.
/// `dst` must be null or point to at least `dst_size` writable u16 elements.
#[allow(non_snake_case)]
pub unsafe extern "win64" fn ExpandEnvironmentStringsW(
    src: *const u16,
    dst: *mut u16,
    dst_size: u32,
) -> u32 {
    let input = match unsafe { read_wstr(src) } {
        Some(s) => s,
        None => return 0,
    };

    let expanded = environment::expand_vars(&input);
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

// ---------------------------------------------------------------------------
// GetEnvironmentStringsW / FreeEnvironmentStringsW
// ---------------------------------------------------------------------------

// Cached wide environment block for `GetEnvironmentStringsW`.
//
// In a real Windows process this block is built at startup and freed by
// `FreeEnvironmentStrings`. We use a `OnceLock` so the first call builds
// the block and subsequent calls return the same pointer. The block is
// leaked intentionally — it lives for the process lifetime.

/// Thin wrapper so a raw pointer can live in a `static OnceLock`.
struct SyncPtr(*mut u16);
unsafe impl Send for SyncPtr {}
unsafe impl Sync for SyncPtr {}

static ENV_BLOCK_W: OnceLock<SyncPtr> = OnceLock::new();

/// GetEnvironmentStringsW — return a pointer to the wide environment block.
///
/// The returned pointer is a null-separated, double-null terminated block
/// of `NAME=value` entries. The caller is expected to free it with
/// `FreeEnvironmentStringsW`, but our implementation leaks intentionally.
///
/// # Safety
/// The returned pointer is valid for the process lifetime.
#[allow(non_snake_case)]
pub unsafe extern "win64" fn GetEnvironmentStringsW() -> *mut u16 {
    ENV_BLOCK_W
        .get_or_init(|| {
            let block = environment::build_wide_block();
            let boxed = block.into_boxed_slice();
            SyncPtr(Box::into_raw(boxed) as *mut u16)
        })
        .0
}

/// FreeEnvironmentStringsW — free a block returned by
/// `GetEnvironmentStringsW`.
///
/// Our implementation is a no-op (the block is leaked on purpose).
///
/// # Safety
/// `block` should be a pointer previously returned by
/// `GetEnvironmentStringsW` (or NULL).
#[allow(non_snake_case)]
pub unsafe extern "win64" fn FreeEnvironmentStringsW(_block: *mut u16) -> WinBool {
    // No-op: the block is leaked for the process lifetime.
    WinBool::TRUE
}
