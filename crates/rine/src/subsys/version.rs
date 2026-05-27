//! Windows version spoofing initialization.
//!
//! Translates the per-app [`WindowsVersion`](rine_types::config::WindowsVersion)
//! setting into the global [`VersionInfo`] used by `GetVersionEx`,
//! `RtlGetVersion`, etc.

use rine_types::config::WindowsVersion;
use rine_types::os::{VersionInfo, set_version};

/// Populate the global version info from the per-app configuration.
///
/// Must be called before PE entry so that API calls return the
/// configured Windows version.
pub fn init_version(ver: WindowsVersion) {
    let (major, minor, build) = ver.version_triple();

    let (sp_major, sp_minor, csd) = match ver {
        WindowsVersion::WinXP => (3, 0, "Service Pack 3"),
        WindowsVersion::Win7 => (1, 0, "Service Pack 1"),
        WindowsVersion::Win10 | WindowsVersion::Win11 => (0, 0, ""),
    };

    set_version(VersionInfo {
        major,
        minor,
        build,
        service_pack_major: sp_major,
        service_pack_minor: sp_minor,
        csd_version: csd.into(),
    });
}
