use rine_types::{
    errors::WinBool,
    os::{
        OsVersionInfoA, OsVersionInfoExA, OsVersionInfoExW, OsVersionInfoW, SIZEOF_OSVERSIONINFOA,
        SIZEOF_OSVERSIONINFOEXA, SIZEOF_OSVERSIONINFOEXW, SIZEOF_OSVERSIONINFOW, get_version,
    },
};

use tracing::{debug, warn};

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
pub fn get_version_packed() -> u32 {
    let ver = get_version();
    debug!("GetVersion: {}.{}.{}", ver.major, ver.minor, ver.build);
    let lo = (ver.major & 0xFF) | ((ver.minor & 0xFF) << 8);
    let hi = ver.build & 0xFFFF;
    (hi << 16) | lo
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
pub unsafe fn get_version_ex_w(info: *mut OsVersionInfoW) -> WinBool {
    if info.is_null() {
        return WinBool::FALSE;
    }

    let ver = get_version();
    let size = unsafe { (*info).os_version_info_size };

    match size {
        SIZEOF_OSVERSIONINFOW => {
            debug!(
                "GetVersionExW: {}.{}.{} (OSVERSIONINFOW)",
                ver.major, ver.minor, ver.build
            );
            unsafe { ver.fill_w(info) };
            WinBool::TRUE
        }
        SIZEOF_OSVERSIONINFOEXW => {
            debug!(
                "GetVersionExW: {}.{}.{} SP{}.{} (OSVERSIONINFOEXW)",
                ver.major, ver.minor, ver.build, ver.service_pack_major, ver.service_pack_minor
            );
            unsafe { ver.fill_ex_w(info.cast::<OsVersionInfoExW>()) };
            WinBool::TRUE
        }
        _ => {
            warn!("GetVersionExW: unexpected size {size}");
            WinBool::FALSE
        }
    }
}

/// Get the current spoofed version info in an ANSI struct.
///
/// # Arguments
/// * `info` - pointer to an `OSVERSIONINFOA` or `OSVERSIONINFOEXA` struct, indicated by the `os_version_info_size` field.
///
/// # Safety
/// `info` must point to a valid, writable `OSVERSIONINFOA` or `OSVERSIONINFOEXA` struct, and must not be null.
///
/// # Returns
/// `WinBool::TRUE` on success, `WinBool::FALSE` (0) on failure.
pub unsafe fn get_version_ex_a(info: *mut OsVersionInfoA) -> WinBool {
    if info.is_null() {
        return WinBool::FALSE;
    }

    let ver = get_version();
    let size = unsafe { (*info).os_version_info_size };

    match size {
        SIZEOF_OSVERSIONINFOA => {
            debug!(
                "GetVersionExA: {}.{}.{} (OSVERSIONINFOA)",
                ver.major, ver.minor, ver.build
            );
            unsafe { ver.fill_a(info) };
            WinBool::TRUE
        }
        SIZEOF_OSVERSIONINFOEXA => {
            debug!(
                "GetVersionExA: {}.{}.{} SP{}.{} (OSVERSIONINFOEXA)",
                ver.major, ver.minor, ver.build, ver.service_pack_major, ver.service_pack_minor
            );
            unsafe { ver.fill_ex_a(info.cast::<OsVersionInfoExA>()) };
            WinBool::TRUE
        }
        _ => {
            warn!("GetVersionExA: unexpected size {size}");
            WinBool::FALSE
        }
    }
}
