use rine_common_kernel32 as common;

/// `GetVersion` — return a packed `DWORD` encoding the OS version.
///
/// Layout: `LOBYTE(LOWORD)` = major, `HIBYTE(LOWORD)` = minor,
/// `HIWORD` = build number.
///
/// # Safety
///
/// Called from PE code via the Windows ABI. The caller must ensure the
/// global version info has been initialised before entry.
#[allow(non_snake_case)]
pub unsafe extern "stdcall" fn GetVersion() -> u32 {
    common::version::get_version_packed()
}
