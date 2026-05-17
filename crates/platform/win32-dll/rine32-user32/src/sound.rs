use rine_common_user32::sound as common;

use rine_types::errors::BOOL;

/// Plays a specific system sound.
///
/// # Arguments
/// * `utype` - The sound to be played. This parameter can be one of the following values:
///  - `MB_SIMPLEBEEP`: A simple beep. This is the default sound.
/// - `MB_OK`: The sound played when the user clicks the OK button of a message box.
/// - `MB_ICONERROR` or `MB_ICONSTOP`: The sound played when the user clicks the Cancel button of a message box,
///   or when a message box is displayed that has the MB_ICONERROR, MB_ICONSTOP, or MB_ICONHAND style.
/// - `MB_ICONQUESTION`: The sound played when the user clicks the Help button of a message box, or when a message box
///   is displayed that has the MB_ICONQUESTION style.
/// - `MB_ICONEXCLAMATION` or `MB_ICONWARNING`: The sound played when the user clicks the Retry button of a message box,
///   or when a message box is displayed that has the MB_ICONEXCLAMATION or MB_ICONWARNING style.
/// - `MB_ICONASTERISK` or `MB_ICONINFORMATION`: The sound played when the user clicks the Ignore button of a message box,
///   or when a message box is displayed that has the MB_ICONASTERISK or MB_ICONINFORMATION style.
///
/// # Returns
/// On success, returns `BOOL::TRUE`. On failure, returns `BOOL::FALSE`.
///
/// # Notes
/// Currently this function is a stub and does not actually play any sound. It always returns `BOOL::TRUE`.
/// Further, this function does not validate the `utype` parameter, nor does it set the last error code on failure.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn MessageBeep(utype: u32) -> BOOL {
    common::message_beep(utype)
}
