//! ntdll Rtl* utility functions: RtlInitUnicodeString, RtlGetVersion.

use rine_types::os::{
    OsVersionInfoExW, OsVersionInfoW, SIZEOF_OSVERSIONINFOEXW, SIZEOF_OSVERSIONINFOW, get_version,
};
use rine_types::strings::UnicodeString;

/// RtlInitUnicodeString — initialise a UNICODE_STRING from a null-terminated
/// wide-character (UTF-16LE) source string.
///
/// # Safety
/// `source` must either be null or point to a valid null-terminated `u16` array.
/// `dest` must be a valid, writable pointer to a `UNICODE_STRING`.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn RtlInitUnicodeString(dest: *mut UnicodeString, source: *const u16) {
    if dest.is_null() {
        return;
    }

    if source.is_null() {
        unsafe {
            (*dest).length = 0;
            (*dest).maximum_length = 0;
            (*dest).buffer = core::ptr::null_mut();
        }
        return;
    }

    // Count u16 code units up to (but not including) the null terminator.
    let mut len: usize = 0;
    unsafe {
        while *source.add(len) != 0 {
            len += 1;
        }
    }

    // Byte lengths (u16 → 2 bytes each). Cap at u16::MAX.
    let byte_len = (len * 2).min(u16::MAX as usize);
    // maximum_length includes the null terminator (2 extra bytes).
    let max_byte_len = (byte_len + 2).min(u16::MAX as usize);

    unsafe {
        (*dest).length = byte_len as u16;
        (*dest).maximum_length = max_byte_len as u16;
        (*dest).buffer = source as *mut u16;
    }
}

// ---------------------------------------------------------------------------
// RtlGetVersion
// ---------------------------------------------------------------------------

/// `RtlGetVersion` — fill an `OSVERSIONINFOEXW` with the current (spoofed)
/// Windows version. Unlike `GetVersionEx` this function is not subject to
/// application compatibility shims.
///
/// Returns `STATUS_SUCCESS` (0) on success.
///
/// # Safety
/// `info` must point to a writable `OSVERSIONINFOW` or `OSVERSIONINFOEXW`
/// with `dwOSVersionInfoSize` set correctly.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn RtlGetVersion(info: *mut OsVersionInfoW) -> u32 {
    if info.is_null() {
        return 0xC000_000D; // STATUS_INVALID_PARAMETER
    }

    let ver = get_version();
    let size = unsafe { (*info).os_version_info_size };

    match size {
        SIZEOF_OSVERSIONINFOW => {
            tracing::debug!(
                "RtlGetVersion: {}.{}.{} (OSVERSIONINFOW)",
                ver.major,
                ver.minor,
                ver.build
            );
            unsafe { ver.fill_w(info) };
        }
        SIZEOF_OSVERSIONINFOEXW | 0 => {
            // Size 0 is accepted by some callers; treat as Ex.
            tracing::debug!(
                "RtlGetVersion: {}.{}.{} SP{}.{} (OSVERSIONINFOEXW)",
                ver.major,
                ver.minor,
                ver.build,
                ver.service_pack_major,
                ver.service_pack_minor
            );
            let ex = info.cast::<OsVersionInfoExW>();
            unsafe {
                (*ex).os_version_info_size = SIZEOF_OSVERSIONINFOEXW;
                ver.fill_ex_w(ex);
            }
        }
        _ => {
            tracing::warn!("RtlGetVersion: unexpected size {size}");
            return 0xC000_000D; // STATUS_INVALID_PARAMETER
        }
    }

    0 // STATUS_SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

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
