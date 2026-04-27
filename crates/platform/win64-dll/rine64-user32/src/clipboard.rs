use rine_common_user32::clipboard as common;
use rine_types::errors::WinBool;

use tracing::warn;

/// Checks if the specified clipboard format is available.
///
/// # Arguments
/// * `_format` - The clipboard format to check for availability.
///
/// # Safety
/// the `_format` parameter must be a valid clipboard format, otherwise the behavior is undefined.
///
/// # Returns
/// * `WinBool::TRUE` if the specified clipboard format is available, `WinBool::FALSE` otherwise.
///
/// # Notes
/// This function is currently not implemented and will return `WinBool::FALSE` for all formats.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "win64" fn IsClipboardFormatAvailable(_format: u32) -> WinBool {
    if let Ok(format) = common::ClipboardFormat::try_from(_format) {
        common::is_clipboard_format_available(format)
    } else {
        warn!("Invalid clipboard format: {}, returning false", _format);
        WinBool::FALSE
    }
}
