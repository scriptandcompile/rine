use rine_types::{
    errors::BOOL,
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
/// Returns the version as a packed `u32` on success, or `BOOL::FALSE` (0) on failure.
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
pub unsafe fn get_version_ex_w(info: *mut OsVersionInfoW) -> BOOL {
    if info.is_null() {
        return BOOL::FALSE;
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
            BOOL::TRUE
        }
        SIZEOF_OSVERSIONINFOEXW => {
            debug!(
                "GetVersionExW: {}.{}.{} SP{}.{} (OSVERSIONINFOEXW)",
                ver.major, ver.minor, ver.build, ver.service_pack_major, ver.service_pack_minor
            );
            unsafe { ver.fill_ex_w(info.cast::<OsVersionInfoExW>()) };
            BOOL::TRUE
        }
        _ => {
            warn!("GetVersionExW: unexpected size {size}");
            BOOL::FALSE
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
/// `BOOL::TRUE` on success, `BOOL::FALSE` (0) on failure.
pub unsafe fn get_version_ex_a(info: *mut OsVersionInfoA) -> BOOL {
    if info.is_null() {
        return BOOL::FALSE;
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
            BOOL::TRUE
        }
        SIZEOF_OSVERSIONINFOEXA => {
            debug!(
                "GetVersionExA: {}.{}.{} SP{}.{} (OSVERSIONINFOEXA)",
                ver.major, ver.minor, ver.build, ver.service_pack_major, ver.service_pack_minor
            );
            unsafe { ver.fill_ex_a(info.cast::<OsVersionInfoExA>()) };
            BOOL::TRUE
        }
        _ => {
            warn!("GetVersionExA: unexpected size {size}");
            BOOL::FALSE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rine_types::os::{
        VER_NT_WORKSTATION, VER_PLATFORM_WIN32_NT, VER_SUITE_SINGLEUSERTS, VersionInfo, set_version,
    };
    use std::sync::{LazyLock, Mutex};

    static TEST_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    fn pack(major: u32, minor: u32, build: u32) -> u32 {
        let lo = (major & 0xFF) | ((minor & 0xFF) << 8);
        let hi = build & 0xFFFF;
        (hi << 16) | lo
    }

    // ── get_version_packed ──────────────────────────────────────

    #[test]
    fn packed_default_version() {
        let _guard = TEST_MUTEX.lock().unwrap();
        set_version(VersionInfo::default());
        assert_eq!(get_version_packed(), pack(10, 0, 22631));
    }

    #[test]
    fn packed_windows_7() {
        let _guard = TEST_MUTEX.lock().unwrap();
        set_version(VersionInfo {
            major: 6,
            minor: 1,
            build: 7601,
            ..Default::default()
        });
        assert_eq!(get_version_packed(), pack(6, 1, 7601));
    }

    #[test]
    fn packed_windows_xp() {
        let _guard = TEST_MUTEX.lock().unwrap();
        set_version(VersionInfo {
            major: 5,
            minor: 1,
            build: 2600,
            ..Default::default()
        });
        assert_eq!(get_version_packed(), pack(5, 1, 2600));
    }

    #[test]
    fn packed_all_zeros() {
        let _guard = TEST_MUTEX.lock().unwrap();
        set_version(VersionInfo {
            major: 0,
            minor: 0,
            build: 0,
            ..Default::default()
        });
        assert_eq!(get_version_packed(), 0);
    }

    #[test]
    fn packed_max_values() {
        let _guard = TEST_MUTEX.lock().unwrap();
        set_version(VersionInfo {
            major: 255,
            minor: 255,
            build: 65535,
            ..Default::default()
        });
        assert_eq!(get_version_packed(), 0xFFFFFFFF);
    }

    // ── get_version_ex_w ────────────────────────────────────────

    #[test]
    fn version_ex_w_null() {
        let _guard = TEST_MUTEX.lock().unwrap();
        assert_eq!(
            unsafe { get_version_ex_w(std::ptr::null_mut()) },
            BOOL::FALSE
        );
    }

    #[test]
    fn version_ex_w_base() {
        let _guard = TEST_MUTEX.lock().unwrap();
        set_version(VersionInfo {
            major: 10,
            minor: 0,
            build: 22631,
            csd_version: "Test".into(),
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
        let result = unsafe { get_version_ex_w(&mut info) };
        assert_eq!(result, BOOL::TRUE);
        assert_eq!(info.major_version, 10);
        assert_eq!(info.minor_version, 0);
        assert_eq!(info.build_number, 22631);
        assert_eq!(info.platform_id, VER_PLATFORM_WIN32_NT);
        assert_eq!(
            String::from_utf16_lossy(&info.csd_version).trim_end_matches('\0'),
            "Test"
        );
    }

    #[test]
    fn version_ex_w_extended() {
        let _guard = TEST_MUTEX.lock().unwrap();
        set_version(VersionInfo {
            major: 10,
            minor: 0,
            build: 22631,
            service_pack_major: 1,
            service_pack_minor: 2,
            ..Default::default()
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
        let ptr = &mut info as *mut OsVersionInfoExW as *mut OsVersionInfoW;
        let result = unsafe { get_version_ex_w(ptr) };
        assert_eq!(result, BOOL::TRUE);
        assert_eq!(info.major_version, 10);
        assert_eq!(info.minor_version, 0);
        assert_eq!(info.build_number, 22631);
        assert_eq!(info.service_pack_major, 1);
        assert_eq!(info.service_pack_minor, 2);
        assert_eq!(info.suite_mask, VER_SUITE_SINGLEUSERTS);
        assert_eq!(info.product_type, VER_NT_WORKSTATION);
    }

    #[test]
    fn version_ex_w_invalid_size() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let mut info = OsVersionInfoW {
            os_version_info_size: 0xDEAD,
            major_version: 0,
            minor_version: 0,
            build_number: 0,
            platform_id: 0,
            csd_version: [0u16; 128],
        };
        assert_eq!(unsafe { get_version_ex_w(&mut info) }, BOOL::FALSE);
    }

    // ── get_version_ex_a ────────────────────────────────────────

    #[test]
    fn version_ex_a_null() {
        let _guard = TEST_MUTEX.lock().unwrap();
        assert_eq!(
            unsafe { get_version_ex_a(std::ptr::null_mut()) },
            BOOL::FALSE
        );
    }

    #[test]
    fn version_ex_a_base() {
        let _guard = TEST_MUTEX.lock().unwrap();
        set_version(VersionInfo {
            major: 6,
            minor: 1,
            build: 7601,
            csd_version: "SP1".into(),
            ..Default::default()
        });
        let mut info = OsVersionInfoA {
            os_version_info_size: SIZEOF_OSVERSIONINFOA,
            major_version: 0,
            minor_version: 0,
            build_number: 0,
            platform_id: 0,
            csd_version: [0u8; 128],
        };
        let result = unsafe { get_version_ex_a(&mut info) };
        assert_eq!(result, BOOL::TRUE);
        assert_eq!(info.major_version, 6);
        assert_eq!(info.minor_version, 1);
        assert_eq!(info.build_number, 7601);
        assert_eq!(info.platform_id, VER_PLATFORM_WIN32_NT);
        let csd = String::from_utf8_lossy(&info.csd_version);
        assert_eq!(csd.trim_end_matches('\0'), "SP1");
    }

    #[test]
    fn version_ex_a_extended() {
        let _guard = TEST_MUTEX.lock().unwrap();
        set_version(VersionInfo {
            major: 6,
            minor: 1,
            build: 7601,
            service_pack_major: 1,
            service_pack_minor: 0,
            ..Default::default()
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
        let ptr = &mut info as *mut OsVersionInfoExA as *mut OsVersionInfoA;
        let result = unsafe { get_version_ex_a(ptr) };
        assert_eq!(result, BOOL::TRUE);
        assert_eq!(info.major_version, 6);
        assert_eq!(info.minor_version, 1);
        assert_eq!(info.build_number, 7601);
        assert_eq!(info.service_pack_major, 1);
        assert_eq!(info.service_pack_minor, 0);
        assert_eq!(info.suite_mask, VER_SUITE_SINGLEUSERTS);
        assert_eq!(info.product_type, VER_NT_WORKSTATION);
    }

    #[test]
    fn version_ex_a_invalid_size() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let mut info = OsVersionInfoA {
            os_version_info_size: 0xDEAD,
            major_version: 0,
            minor_version: 0,
            build_number: 0,
            platform_id: 0,
            csd_version: [0u8; 128],
        };
        assert_eq!(unsafe { get_version_ex_a(&mut info) }, BOOL::FALSE);
    }
}
