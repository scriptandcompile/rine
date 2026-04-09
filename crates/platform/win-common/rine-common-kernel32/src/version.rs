use rine_types::os::get_version;

use tracing::debug;

/// Read the current spoofed version info.
pub fn get_version_packed() -> u32 {
    let ver = get_version();
    debug!("GetVersion: {}.{}.{}", ver.major, ver.minor, ver.build);
    let lo = (ver.major & 0xFF) | ((ver.minor & 0xFF) << 8);
    let hi = ver.build & 0xFFFF;
    (hi << 16) | lo
}
