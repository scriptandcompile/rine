//! kernel32 version functions: GetVersionExA/W, GetVersion.

use rine_types::os::{
    OsVersionInfoA, OsVersionInfoExA, OsVersionInfoExW, OsVersionInfoW, SIZEOF_OSVERSIONINFOA,
    SIZEOF_OSVERSIONINFOEXA, SIZEOF_OSVERSIONINFOEXW, SIZEOF_OSVERSIONINFOW, get_version,
};
use tracing::{debug, warn};

// ---------------------------------------------------------------------------
// GetVersionExW
// ---------------------------------------------------------------------------

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
pub unsafe extern "win64" fn GetVersionExW(info: *mut OsVersionInfoW) -> i32 {
    if info.is_null() {
        return 0;
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
            1
        }
        SIZEOF_OSVERSIONINFOEXW => {
            debug!(
                "GetVersionExW: {}.{}.{} SP{}.{} (OSVERSIONINFOEXW)",
                ver.major, ver.minor, ver.build, ver.service_pack_major, ver.service_pack_minor
            );
            unsafe { ver.fill_ex_w(info.cast::<OsVersionInfoExW>()) };
            1
        }
        _ => {
            warn!("GetVersionExW: unexpected size {size}");
            0
        }
    }
}

// ---------------------------------------------------------------------------
// GetVersionExA
// ---------------------------------------------------------------------------

/// `GetVersionExA` — ANSI variant of `GetVersionExW`.
///
/// # Safety
/// `info` must point to a valid, writable `OSVERSIONINFOA` or
/// `OSVERSIONINFOEXA`.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetVersionExA(info: *mut OsVersionInfoA) -> i32 {
    if info.is_null() {
        return 0;
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
            1
        }
        SIZEOF_OSVERSIONINFOEXA => {
            debug!(
                "GetVersionExA: {}.{}.{} SP{}.{} (OSVERSIONINFOEXA)",
                ver.major, ver.minor, ver.build, ver.service_pack_major, ver.service_pack_minor
            );
            unsafe { ver.fill_ex_a(info.cast::<OsVersionInfoExA>()) };
            1
        }
        _ => {
            warn!("GetVersionExA: unexpected size {size}");
            0
        }
    }
}

// ---------------------------------------------------------------------------
// GetVersion
// ---------------------------------------------------------------------------

/// `GetVersion` — return a packed `DWORD` encoding the OS version.
///
/// Layout: `LOBYTE(LOWORD)` = major, `HIBYTE(LOWORD)` = minor,
/// `HIWORD` = build number.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetVersion() -> u32 {
    let ver = get_version();
    debug!("GetVersion: {}.{}.{}", ver.major, ver.minor, ver.build);
    let lo = (ver.major & 0xFF) | ((ver.minor & 0xFF) << 8);
    let hi = ver.build & 0xFFFF;
    (hi << 16) | lo
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    use rine_types::os::{self, VER_NT_WORKSTATION, VER_PLATFORM_WIN32_NT, VersionInfo};
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
        assert_eq!(ret, 1);
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
        assert_eq!(ret, 1);
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
        assert_eq!(ret, 1);
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
