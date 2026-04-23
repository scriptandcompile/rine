//! kernel32 version functions: GetVersionExA/W, GetVersion.

use rine_common_kernel32 as common;
use rine_types::errors::WinBool;
use rine_types::os::{OsVersionInfoA, OsVersionInfoW};

// ---------------------------------------------------------------------------
// GetVersionExW
// ---------------------------------------------------------------------------

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
pub unsafe extern "win64" fn GetVersionExW(info: *mut OsVersionInfoW) -> WinBool {
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
pub unsafe extern "win64" fn GetVersionExA(info: *mut OsVersionInfoA) -> WinBool {
    unsafe { common::version::get_version_ex_a(info) }
}

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
pub unsafe extern "win64" fn GetVersion() -> u32 {
    common::version::get_version_packed()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    use rine_types::os::{
        self, OsVersionInfoExA, OsVersionInfoExW, OsVersionInfoW, SIZEOF_OSVERSIONINFOEXA,
        SIZEOF_OSVERSIONINFOEXW, SIZEOF_OSVERSIONINFOW, VER_NT_WORKSTATION, VER_PLATFORM_WIN32_NT,
        VersionInfo,
    };
    use serial_test::serial;

    #[test]
    #[serial]
    fn get_version_packs_correctly() {
        os::set_version(VersionInfo {
            major: 6,
            minor: 1,
            build: 7601,
            service_pack_major: 1,
            service_pack_minor: 0,
            csd_version: "Service Pack 1".into(),
        });
        let v = unsafe { GetVersion() };
        assert_eq!(v & 0xFF, 6); // major
        assert_eq!((v >> 8) & 0xFF, 1); // minor
        assert_eq!(v >> 16, 7601); // build
    }

    #[test]
    #[serial]
    fn get_version_ex_w_fills_base_struct() {
        os::set_version(VersionInfo {
            major: 10,
            minor: 0,
            build: 19045,
            ..Default::default()
        });

        let mut info = OsVersionInfoW {
            os_version_info_size: SIZEOF_OSVERSIONINFOW,
            major_version: 0,
            minor_version: 0,
            build_number: 0,
            platform_id: 0,
            csd_version: [0u16; 128],
        };

        let ret = unsafe { GetVersionExW(&mut info) };
        assert_eq!(ret, WinBool::TRUE);
        assert_eq!(info.major_version, 10);
        assert_eq!(info.minor_version, 0);
        assert_eq!(info.build_number, 19045);
        assert_eq!(info.platform_id, VER_PLATFORM_WIN32_NT);
    }

    #[test]
    #[serial]
    fn get_version_ex_w_fills_ex_struct() {
        os::set_version(VersionInfo {
            major: 6,
            minor: 1,
            build: 7601,
            service_pack_major: 1,
            service_pack_minor: 0,
            csd_version: "Service Pack 1".into(),
        });

        let mut info = OsVersionInfoExW {
            os_version_info_size: SIZEOF_OSVERSIONINFOEXW,
            major_version: 0,
            minor_version: 0,
            build_number: 0,
            platform_id: 0,
            csd_version: [0u16; 128],
            service_pack_major: 0,
            service_pack_minor: 0,
            suite_mask: 0,
            product_type: 0,
            reserved: 0,
        };

        let ret = unsafe { GetVersionExW(ptr::from_mut(&mut info).cast()) };
        assert_eq!(ret, WinBool::TRUE);
        assert_eq!(info.major_version, 6);
        assert_eq!(info.minor_version, 1);
        assert_eq!(info.build_number, 7601);
        assert_eq!(info.service_pack_major, 1);
        assert_eq!(info.product_type, VER_NT_WORKSTATION);
        // Check CSD string: "Service Pack 1"
        let csd: String = info
            .csd_version
            .iter()
            .take_while(|&&c| c != 0)
            .map(|&c| char::from(c as u8))
            .collect();
        assert_eq!(csd, "Service Pack 1");
    }

    #[test]
    #[serial]
    fn get_version_ex_a_fills_ex_struct() {
        os::set_version(VersionInfo {
            major: 5,
            minor: 1,
            build: 2600,
            service_pack_major: 3,
            service_pack_minor: 0,
            csd_version: "Service Pack 3".into(),
        });

        let mut info = OsVersionInfoExA {
            os_version_info_size: SIZEOF_OSVERSIONINFOEXA,
            major_version: 0,
            minor_version: 0,
            build_number: 0,
            platform_id: 0,
            csd_version: [0u8; 128],
            service_pack_major: 0,
            service_pack_minor: 0,
            suite_mask: 0,
            product_type: 0,
            reserved: 0,
        };

        let ret = unsafe { GetVersionExA(ptr::from_mut(&mut info).cast()) };
        assert_eq!(ret, WinBool::TRUE);
        assert_eq!(info.major_version, 5);
        assert_eq!(info.minor_version, 1);
        assert_eq!(info.build_number, 2600);
        assert_eq!(info.service_pack_major, 3);
        let csd = std::str::from_utf8(
            &info.csd_version[..info.csd_version.iter().position(|&b| b == 0).unwrap_or(128)],
        )
        .unwrap();
        assert_eq!(csd, "Service Pack 3");
    }
}
