use rine_common_user32 as common;
use rine_types::errors::WinBool;
use rine_types::strings::{read_cstr, read_wstr};

/// Set the title bar text for a window.
///
/// # Arguments
/// * `hwnd` - Handle to the window.
/// * `text` - New title text as a C-style string (null-terminated).
///
/// # Safety
/// * `text` must be a valid pointer to a null-terminated string.
/// * The function assumes the caller has the right to modify the window's title.
///
/// # Returns
/// `WinBool::TRUE` on success, `WinBool::FALSE` if the HWND is not found.
///
/// # Notes
/// Currently, this function does not perform any access checks on the window handle (HWND).
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn SetWindowTextA(hwnd: usize, text: *const u8) -> WinBool {
    common::set_window_text(hwnd, read_cstr(text).unwrap_or_default())
}

/// Set the title bar text for a window.
///
/// # Arguments
/// * `hwnd` - Handle to the window.
/// * `text` - New title text as a C-style string (null-terminated).
///
/// # Safety
/// * `text` must be a valid pointer to a null-terminated string.
/// * The function assumes the caller has the right to modify the window's title.
///
/// # Returns
/// `WinBool::TRUE` on success, `WinBool::FALSE` if the HWND is not found.
///
/// # Notes
/// Currently, this function does not perform any access checks on the window handle (HWND).
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn SetWindowTextW(hwnd: usize, text: *const u16) -> WinBool {
    common::set_window_text(hwnd, read_wstr(text).unwrap_or_default())
}

/// Copy the window title into an ANSI buffer.
///
/// # Arguments
/// * `hwnd` - Handle to the window.
/// * `buffer` - Pointer to a buffer that receives the window title as an ANSI string (null-terminated).
/// * `max_count` - Maximum number of characters to copy, including the null terminator.
///
/// # Safety
/// * `buffer` must point to at least `max_count` bytes of writable memory.
/// * The function assumes the caller has the right to read the window's title.
///
/// # Returns
/// The number of characters copied, excluding the null terminator. If the text exceeds this limit,
/// it should be truncated.
/// Returns 0 if the HWND is not found or if `buffer` is null or `max_count` is non-positive.
///
/// # Notes
/// Currently, this function does not perform any access checks on the window handle (HWND).
/// This function should write an error to `GetLastError()` if the HWND is not found, but this is not yet implemented.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetWindowTextA(hwnd: usize, buffer: *mut u8, max_count: i32) -> i32 {
    unsafe { common::get_window_text_a(hwnd, buffer, max_count) }
}

/// Copy the window title into a wide buffer.
///
/// # Arguments
/// * `hwnd` - Handle to the window.
/// * `buffer` - Pointer to a buffer that receives the window title as a wide string (null-terminated).
/// * `max_count` - Maximum number of characters to copy, including the null terminator.
///
/// # Safety
/// * `buffer` must point to at least `max_count` bytes of writable memory.
/// * The function assumes the caller has the right to read the window's title.
///
/// # Returns
/// The number of characters copied, excluding the null terminator. If the text exceeds this limit,
/// it should be truncated.
/// Returns 0 if the HWND is not found or if `buffer` is null or `max_count` is non-positive.
///
/// # Notes
/// Currently, this function does not perform any access checks on the window handle (HWND).
/// This function should write an error to `GetLastError()` if the HWND is not found, but this is not yet implemented.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetWindowTextW(hwnd: usize, buffer: *mut u16, max_count: i32) -> i32 {
    unsafe { common::get_window_text_w(hwnd, buffer, max_count) }
}

/// Get the length of the window title in characters, excluding the null terminator.
///
/// # Arguments
/// * `hwnd` - Handle to the window.
///
/// # Safety
/// The function assumes the caller has the right to read the window's title.
///
/// # Returns
/// The length of the window title in characters, excluding the null terminator. Returns 0 if the HWND is not found.
///
/// # Notes
/// Currently, this function does not perform any access checks on the window handle (HWND).
/// This function should write an error to `GetLastError()` if the HWND is not found, but this is not yet implemented.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetWindowTextLengthA(hwnd: usize) -> i32 {
    common::get_window_text_length(hwnd)
}

/// Get the length of the window title in characters, excluding the null terminator.
///
/// # Arguments
/// * `hwnd` - Handle to the window.
///
/// # Safety
/// The function assumes the caller has the right to read the window's title.
///
/// # Returns
/// The length of the window title in characters, excluding the null terminator. Returns 0 if the HWND is not found.
///
/// # Notes
/// Currently, this function does not perform any access checks on the window handle (HWND).
/// This function should write an error to `GetLastError()` if the HWND is not found, but this is not yet implemented.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetWindowTextLengthW(hwnd: usize) -> i32 {
    common::get_window_text_length(hwnd)
}
