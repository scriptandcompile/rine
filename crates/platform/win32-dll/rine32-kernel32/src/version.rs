use rine_common_kernel32 as common;
use rine_types::errors::WinBool;
use rine_types::os::{OsVersionInfoA, OsVersionInfoW};

/// Gets a packed `u32` encoding the OS version.
///
/// Layout: `LOBYTE(LOWORD)` = major, `HIBYTE(LOWORD)` = minor,
/// `HIWORD` = build number.
///
/// # Safety
/// Called from PE code via the Windows ABI. The caller must ensure the
/// global version info has been initialised before entry.
///
/// # Returns
/// Returns the version as a packed `u32` on success, or `WinBool::FALSE` (0) on failure.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetVersion() -> u32 {
    common::version::get_version_packed()
}

/// Get the current spoofed version info in a wide struct.
///
/// # Arguments
/// * `info` - pointer to an `OSVERSIONINFOW` or `OSVERSIONINFOEXW` struct, indicated by the `os_version_info_size` field.
///
/// # Safety
/// `info` must point to a valid, writable `OSVERSIONINFOW` or
/// `OSVERSIONINFOEXW` struct, and must not be null.
///
/// # Returns
/// `WinBool::TRUE` on success, `WinBool::FALSE` (0) on failure.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetVersionExW(info: *mut OsVersionInfoW) -> WinBool {
    unsafe { common::version::get_version_ex_w(info) }
}

/// Get the current spoofed version info in an ANSI struct.
///
/// # Arguments
/// * `info` - pointer to an `OSVERSIONINFOA` or `OSVERSIONINFOEXA` struct, indicated by the `os_version_info_size` field.
///
/// # Safety
/// `info` must point to a valid, writable `OSVERSIONINFOA` or
/// `OSVERSIONINFOEXA` struct, and must not be null.
///
/// # Returns
/// `WinBool::TRUE` on success, `WinBool::FALSE` (0) on failure.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetVersionExA(info: *mut OsVersionInfoA) -> WinBool {
    unsafe { common::version::get_version_ex_a(info) }
}
