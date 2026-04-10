use rine_common_kernel32 as common;
use rine_types::os::{OsVersionInfoA, OsVersionInfoW};

/// `GetVersion` — return a packed `DWORD` encoding the OS version.
///
/// Layout: `LOBYTE(LOWORD)` = major, `HIBYTE(LOWORD)` = minor,
/// `HIWORD` = build number.
///
/// # Safety
///
/// Called from PE code via the Windows ABI. The caller must ensure the
/// global version info has been initialised before entry.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetVersion() -> u32 {
    common::version::get_version_packed()
}

/// `GetVersionExW` — fill an `OSVERSIONINFOW` or `OSVERSIONINFOEXW` with the
/// spoofed Windows version.
///
/// The caller sets `dwOSVersionInfoSize` to indicate which struct variant
/// they allocated. We accept both the base and Ex sizes.
///
/// Returns `TRUE` (1) on success, `FALSE` (0) on failure.
///
/// # Safety
/// `info` must point to a valid, writable `OSVERSIONINFOW` or
/// `OSVERSIONINFOEXW` whose `dwOSVersionInfoSize` field is set correctly.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetVersionExW(info: *mut OsVersionInfoW) -> i32 {
    unsafe { common::version::get_version_ex_w(info) }
}

/// `GetVersionExA` — ANSI variant of `GetVersionExW`.
///
/// # Arguments
/// * `info` - pointer to an `OSVERSIONINFOA` or `OSVERSIONINFOEXA` struct, indicated by the `os_version_info_size` field.
///
/// # Safety
/// `info` must point to a valid, writable `OSVERSIONINFOA` or
/// `OSVERSIONINFOEXA` struct, and must not be null.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetVersionExA(info: *mut OsVersionInfoA) -> i32 {
    unsafe { common::version::get_version_ex_a(info) }
}
