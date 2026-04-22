use rine_common_comdlg32 as common;

/// CommDlgExtendedError — get extended error code from last dialog operation.
///
/// # Safety
/// This function is unsafe because it may be called in contexts where the common
/// dialog operations have not been properly initialized or used, which could lead
/// to undefined behavior.
/// The caller must ensure that the common dialog functions have been called and
/// that any necessary setup has been performed before calling this function.
/// Additionally, the caller should be aware that the returned error code may not
/// be meaningful if the common dialog operations were not used correctly, and
/// should handle the returned value accordingly to avoid potential issues.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn CommDlgExtendedError() -> u32 {
    common::error::last_error()
}
