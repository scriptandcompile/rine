//! ntdll Rtl* utility functions: RtlInitUnicodeString, RtlGetVersion.

use rine_common_ntdll as common;
use rine_types::os::OsVersionInfoW;
use rine_types::strings::UnicodeString;

/// Initialize a `UnicodeString` structure with the given source string.
///
/// # Arguments
/// * `destination_string`: pointer to the `UnicodeString` structure to initialize.
/// * `source_string`: pointer to a null-terminated wide string (PCWSTR) to copy into the `UnicodeString`.
///
/// # Safety
/// All pointer parameters must be valid.
/// The `destination_string` must point to a valid `UnicodeString` structure, and `source_string` must point
/// to a valid null-terminated wide string.
///
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn RtlInitUnicodeString(dest: *mut UnicodeString, source: *const u16) {
    unsafe { common::rtl::rtl_init_unicode_string(dest, source) };
}

/// Fill an `OSVERSIONINFOEXW` with the current (spoofed) Windows version.
/// Unlike `GetVersionEx` this function is not subject to application compatibility shims.
///
/// # Arguments
/// * `info`: pointer to a writable `OSVERSIONINFOW` or `OSVERSIONINFOEXW` structure with
///   `dwOSVersionInfoSize` set correctly.
///
/// # Safety
/// `info` must point to a writable `OSVERSIONINFOW` or `OSVERSIONINFOEXW`
/// with `dwOSVersionInfoSize` set correctly.
///
/// # Returns
/// `STATUS_SUCCESS` (0) on success. `STATUS_INVALID_PARAMETER` (0xC000_000D) if `info` is null or has an unexpected size.
///
/// # Notes
/// This function fills the provided structure with a spoofed Windows version, which can be configured
/// via environment variables. The version information is logged for debugging purposes.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn RtlGetVersion(info: *mut OsVersionInfoW) -> u32 {
    unsafe { common::rtl::rtl_get_version(info) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rine_types::os::{OsVersionInfoExW, SIZEOF_OSVERSIONINFOEXW};

    #[test]
    fn init_unicode_string_from_source() {
        let wide: Vec<u16> = "hello".encode_utf16().chain(std::iter::once(0)).collect();
        let mut us = UnicodeString::empty();

        unsafe { RtlInitUnicodeString(&mut us, wide.as_ptr()) };

        assert_eq!(us.length, 10); // 5 chars × 2 bytes
        assert_eq!(us.maximum_length, 12); // includes null
        assert!(!us.buffer.is_null());
    }

    #[test]
    fn init_unicode_string_null_source() {
        let mut us = UnicodeString::empty();
        unsafe { RtlInitUnicodeString(&mut us, core::ptr::null()) };
        assert_eq!(us.length, 0);
        assert!(us.buffer.is_null());
    }

    #[test]
    #[serial_test::serial]
    fn rtl_get_version_fills_ex_struct() {
        use rine_types::os::{self, VersionInfo};

        os::set_version(VersionInfo {
            major: 10,
            minor: 0,
            build: 22631,
            service_pack_major: 0,
            service_pack_minor: 0,
            csd_version: String::new(),
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

        let status =
            unsafe { RtlGetVersion(core::ptr::from_mut(&mut info).cast::<OsVersionInfoW>()) };
        assert_eq!(status, 0); // STATUS_SUCCESS
        assert_eq!(info.major_version, 10);
        assert_eq!(info.minor_version, 0);
        assert_eq!(info.build_number, 22631);
        assert_eq!(info.product_type, rine_types::os::VER_NT_WORKSTATION);
    }
}
