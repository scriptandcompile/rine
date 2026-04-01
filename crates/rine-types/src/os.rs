//! Miscellaneous Windows structures used by DLL implementations.

use std::sync::{LazyLock, RwLock};

// ---------------------------------------------------------------------------
// Version info structures (OSVERSIONINFO / OSVERSIONINFOEX)
// ---------------------------------------------------------------------------

/// `OSVERSIONINFOW` — wide version info returned by `GetVersionExW`.
#[repr(C)]
pub struct OsVersionInfoW {
    pub os_version_info_size: u32,
    pub major_version: u32,
    pub minor_version: u32,
    pub build_number: u32,
    pub platform_id: u32,
    pub csd_version: [u16; 128],
}

/// `OSVERSIONINFOEXW` — extended wide version info returned by `GetVersionExW`
/// and `RtlGetVersion`.
#[repr(C)]
pub struct OsVersionInfoExW {
    pub os_version_info_size: u32,
    pub major_version: u32,
    pub minor_version: u32,
    pub build_number: u32,
    pub platform_id: u32,
    pub csd_version: [u16; 128],
    pub service_pack_major: u16,
    pub service_pack_minor: u16,
    pub suite_mask: u16,
    pub product_type: u8,
    pub reserved: u8,
}

/// `OSVERSIONINFOA` — ANSI version info returned by `GetVersionExA`.
#[repr(C)]
pub struct OsVersionInfoA {
    pub os_version_info_size: u32,
    pub major_version: u32,
    pub minor_version: u32,
    pub build_number: u32,
    pub platform_id: u32,
    pub csd_version: [u8; 128],
}

/// `OSVERSIONINFOEXA` — extended ANSI version info.
#[repr(C)]
pub struct OsVersionInfoExA {
    pub os_version_info_size: u32,
    pub major_version: u32,
    pub minor_version: u32,
    pub build_number: u32,
    pub platform_id: u32,
    pub csd_version: [u8; 128],
    pub service_pack_major: u16,
    pub service_pack_minor: u16,
    pub suite_mask: u16,
    pub product_type: u8,
    pub reserved: u8,
}

// Size constants matching Windows definitions.
/// Size of `OSVERSIONINFOW` (276 bytes).
pub const SIZEOF_OSVERSIONINFOW: u32 = core::mem::size_of::<OsVersionInfoW>() as u32;
/// Size of `OSVERSIONINFOEXW` (284 bytes).
pub const SIZEOF_OSVERSIONINFOEXW: u32 = core::mem::size_of::<OsVersionInfoExW>() as u32;
/// Size of `OSVERSIONINFOA` (148 bytes).
pub const SIZEOF_OSVERSIONINFOA: u32 = core::mem::size_of::<OsVersionInfoA>() as u32;
/// Size of `OSVERSIONINFOEXA` (156 bytes).
pub const SIZEOF_OSVERSIONINFOEXA: u32 = core::mem::size_of::<OsVersionInfoExA>() as u32;

/// `VER_PLATFORM_WIN32_NT`
pub const VER_PLATFORM_WIN32_NT: u32 = 2;
/// `VER_NT_WORKSTATION`
pub const VER_NT_WORKSTATION: u8 = 1;
/// `VER_SUITE_SINGLEUSERTS`
pub const VER_SUITE_SINGLEUSERTS: u16 = 0x0100;

// ---------------------------------------------------------------------------
// Global version state
// ---------------------------------------------------------------------------

/// Spoofed Windows version info, set once at startup from the app config.
#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub major: u32,
    pub minor: u32,
    pub build: u32,
    pub service_pack_major: u16,
    pub service_pack_minor: u16,
    /// CSD version string, e.g. "Service Pack 1" (ANSI).
    pub csd_version: String,
}

impl Default for VersionInfo {
    fn default() -> Self {
        // Default: Windows 11 (10.0.22631)
        Self {
            major: 10,
            minor: 0,
            build: 22631,
            service_pack_major: 0,
            service_pack_minor: 0,
            csd_version: String::new(),
        }
    }
}

static VERSION: LazyLock<RwLock<VersionInfo>> =
    LazyLock::new(|| RwLock::new(VersionInfo::default()));

/// Set the spoofed Windows version. Must be called before PE entry.
pub fn set_version(info: VersionInfo) {
    *VERSION.write().unwrap() = info;
}

/// Read the current spoofed version info.
pub fn get_version() -> VersionInfo {
    VERSION.read().unwrap().clone()
}

impl VersionInfo {
    /// Fill an `OSVERSIONINFOW` (base struct) with the spoofed version.
    ///
    /// # Safety
    /// `info` must be a valid, writable pointer.
    pub unsafe fn fill_w(&self, info: *mut OsVersionInfoW) {
        unsafe {
            (*info).major_version = self.major;
            (*info).minor_version = self.minor;
            (*info).build_number = self.build;
            (*info).platform_id = VER_PLATFORM_WIN32_NT;
            write_csd_wide(&mut (*info).csd_version, &self.csd_version);
        }
    }

    /// Fill an `OSVERSIONINFOEXW` (extended struct) with the spoofed version.
    ///
    /// # Safety
    /// `info` must be a valid, writable pointer.
    pub unsafe fn fill_ex_w(&self, info: *mut OsVersionInfoExW) {
        unsafe {
            (*info).major_version = self.major;
            (*info).minor_version = self.minor;
            (*info).build_number = self.build;
            (*info).platform_id = VER_PLATFORM_WIN32_NT;
            write_csd_wide(&mut (*info).csd_version, &self.csd_version);
            (*info).service_pack_major = self.service_pack_major;
            (*info).service_pack_minor = self.service_pack_minor;
            (*info).suite_mask = VER_SUITE_SINGLEUSERTS;
            (*info).product_type = VER_NT_WORKSTATION;
            (*info).reserved = 0;
        }
    }

    /// Fill an `OSVERSIONINFOA` (ANSI base struct) with the spoofed version.
    ///
    /// # Safety
    /// `info` must be a valid, writable pointer.
    pub unsafe fn fill_a(&self, info: *mut OsVersionInfoA) {
        unsafe {
            (*info).major_version = self.major;
            (*info).minor_version = self.minor;
            (*info).build_number = self.build;
            (*info).platform_id = VER_PLATFORM_WIN32_NT;
            write_csd_ansi(&mut (*info).csd_version, &self.csd_version);
        }
    }

    /// Fill an `OSVERSIONINFOEXA` (ANSI extended struct) with the spoofed version.
    ///
    /// # Safety
    /// `info` must be a valid, writable pointer.
    pub unsafe fn fill_ex_a(&self, info: *mut OsVersionInfoExA) {
        unsafe {
            (*info).major_version = self.major;
            (*info).minor_version = self.minor;
            (*info).build_number = self.build;
            (*info).platform_id = VER_PLATFORM_WIN32_NT;
            write_csd_ansi(&mut (*info).csd_version, &self.csd_version);
            (*info).service_pack_major = self.service_pack_major;
            (*info).service_pack_minor = self.service_pack_minor;
            (*info).suite_mask = VER_SUITE_SINGLEUSERTS;
            (*info).product_type = VER_NT_WORKSTATION;
            (*info).reserved = 0;
        }
    }
}

/// Write a CSD version string into a `[u16; 128]` buffer (wide).
fn write_csd_wide(buf: &mut [u16; 128], csd: &str) {
    *buf = [0u16; 128];
    for (i, unit) in csd.encode_utf16().take(127).enumerate() {
        buf[i] = unit;
    }
}

/// Write a CSD version string into a `[u8; 128]` buffer (ANSI).
fn write_csd_ansi(buf: &mut [u8; 128], csd: &str) {
    *buf = [0u8; 128];
    let bytes = csd.as_bytes();
    let len = bytes.len().min(127);
    buf[..len].copy_from_slice(&bytes[..len]);
}

// ---------------------------------------------------------------------------
// I/O structures
// ---------------------------------------------------------------------------

/// IO_STATUS_BLOCK — returned by NT I/O functions.
#[repr(C)]
pub struct IoStatusBlock {
    /// NTSTATUS or pointer (union in Windows; we use the status variant).
    pub status: u32,
    /// Number of bytes transferred.
    pub information: usize,
}

// ---------------------------------------------------------------------------
// Process creation structures
// ---------------------------------------------------------------------------

/// `STARTUPINFOA` — startup parameters passed to `CreateProcessA`.
///
/// Most fields are ignored by rine; we define the full layout so the PE
/// code's pointer arithmetic lands correctly.
#[repr(C)]
pub struct StartupInfoA {
    pub cb: u32,
    pub reserved: *mut u8,
    pub desktop: *mut u8,
    pub title: *mut u8,
    pub x: u32,
    pub y: u32,
    pub x_size: u32,
    pub y_size: u32,
    pub x_count_chars: u32,
    pub y_count_chars: u32,
    pub fill_attribute: u32,
    pub flags: u32,
    pub show_window: u16,
    pub cb_reserved2: u16,
    pub reserved2: *mut u8,
    pub std_input: isize,
    pub std_output: isize,
    pub std_error: isize,
}

/// `STARTUPINFOW` — wide variant.
#[repr(C)]
pub struct StartupInfoW {
    pub cb: u32,
    pub reserved: *mut u16,
    pub desktop: *mut u16,
    pub title: *mut u16,
    pub x: u32,
    pub y: u32,
    pub x_size: u32,
    pub y_size: u32,
    pub x_count_chars: u32,
    pub y_count_chars: u32,
    pub fill_attribute: u32,
    pub flags: u32,
    pub show_window: u16,
    pub cb_reserved2: u16,
    pub reserved2: *mut u8,
    pub std_input: isize,
    pub std_output: isize,
    pub std_error: isize,
}

/// `PROCESS_INFORMATION` — filled in by `CreateProcessA/W`.
#[repr(C)]
pub struct ProcessInformation {
    pub process: isize,
    pub thread: isize,
    pub process_id: u32,
    pub thread_id: u32,
}
