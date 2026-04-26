use rine_common_user32 as common;
use rine_types::strings::{read_cstr, read_wstr};

/// Registers a new window message with the system and returns its message ID.
///
/// # Arguments
/// * `lpString` - A null-terminated ANSI string that specifies the message to be registered.
///   The string can be any length, but it must be unique from other strings passed to this function.
///   The string can only contain characters in the range 1 through 255; it cannot contain characters in the range 0 or 256 through 65535.
///
/// # Safety
/// * `lpString` must be a valid pointer to a null-terminated ANSI string that meets the requirements specified above.
///
/// # Returns
/// If the function succeeds, the return value is a message identifier in the range 0xC000 through 0xFFFF.
/// This message identifier can be used as the message parameter when sending or posting messages.
/// If the function fails, the return value is zero. To get extended error information
/// call GetLastError.
///
/// # Notes
/// Currently, this is a stub implementation that always returns 0.
/// We do not set `GetLastError` yet.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn RegisterWindowMessageA(lpString: *const u8) -> u32 {
    unsafe {
        let Some(message_kind) = read_cstr(lpString) else {
            // If the pointer is invalid or the string is not null-terminated, we treat it as an empty string.
            // In a real implementation, we would set GetLastError to indicate the error.
            return 0;
        };

        common::register_window_message(&message_kind)
    }
}

/// Registers a new window message with the system and returns its message ID.
///
/// # Arguments
/// * `lpString` - A null-terminated wide string that specifies the message to be registered.
///
/// # Safety
/// * `lpString` must be a valid pointer to a null-terminated wide string that meets the requirements specified above.
///
/// # Returns
/// If the function succeeds, the return value is a message identifier in the range 0xC000 through 0xFFFF.
/// This message identifier can be used as the message parameter when sending or posting messages.
/// If the function fails, the return value is zero. To get extended error information
/// call GetLastError.
///
/// # Notes
/// Currently, this is a stub implementation that always returns 0.
/// We do not set `GetLastError` yet.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn RegisterWindowMessageW(lpString: *const u16) -> u32 {
    unsafe {
        let Some(message_kind) = read_wstr(lpString) else {
            // If the pointer is invalid or the string is not null-terminated, we treat it as an empty string.
            // In a real implementation, we would set GetLastError to indicate the error.
            return 0;
        };

        common::register_window_message(&message_kind)
    }
}
