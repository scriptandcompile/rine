use core::sync::atomic::{AtomicU32, Ordering};

static LAST_ERROR: AtomicU32 = AtomicU32::new(DialogErrorCode::None as u32);

/// Common dialog extended error codes (subset).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DialogErrorCode {
    None = 0,
    CderrInitialization = 0x0002,
    CderrDialogFailure = 0xFFFF,
    FnerrBufferTooSmall = 0x3003,
}

/// Retrieves the extended error code from the common dialog operations.
///
/// # Returns
/// The extended error code as a `u32`. This will be one of the values from the `DialogErrorCode` enum,
/// or potentially other values if additional error codes are defined in the future.
/// The caller can compare the returned value against the `DialogErrorCode` enum to determine the
/// specific error that occurred during the common dialog operation.
#[allow(non_snake_case)]
pub fn last_error() -> u32 {
    LAST_ERROR.load(Ordering::Relaxed)
}

/// Sets the extended error code for common dialog operations.
///
/// # Arguments
/// * `code`: The `DialogErrorCode` value to set as the last error.
///
/// This function updates the `LAST_ERROR` atomic variable with the provided error code. It should be
/// called by the common dialog implementations whenever an error occurs, to allow callers to retrieve
/// the extended error information using the `last_error()` function. The error code should be set
/// according to the specific error that occurred, using the values defined in the `DialogErrorCode`
/// enum or any additional error codes that may be defined in the future.
pub fn set_last_error(code: DialogErrorCode) {
    LAST_ERROR.store(code as u32, Ordering::Relaxed);
}
