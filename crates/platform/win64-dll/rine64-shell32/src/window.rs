use rine_common_shell32::window as common;

use rine_types::errors::WinBool;
use rine_types::strings::{LPSTR, LPWSTR};
use rine_types::windows::HDROP;
use rine_types::windows::Hwnd;

/// Enables or disables file-drop acceptance for a window.
///
/// # Arguments
/// * `hwnd` - Handle to the target window.
/// * `f_accept` - Nonzero enables file drops, zero disables them.
///
/// # Safety
/// The caller must ensure `hwnd` refers to a valid window handle for this process context.
///
/// # Return
/// This function returns no value.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn DragAcceptFiles(hwnd: Hwnd, f_accept: WinBool) {
    common::drag_accept_files(hwnd, f_accept);
}

/// Queries file paths from an `HDROP` handle.
///
/// # Arguments
/// * `hDrop` - Handle identifying the dropped-file list.
/// * `iFile` - File index, or `0xFFFFFFFF` to query file count.
/// * `lpszFile` - Optional output buffer for the file path.
/// * `cch` - Output buffer size in bytes.
///
/// # Safety
/// `hDrop` must refer to a valid DROPFILES memory block. When `lpszFile`
/// is non-null, it must reference writable memory of at least `cch` bytes.
///
/// # Return
/// Returns file count when `iFile == 0xFFFFFFFF`; otherwise returns the selected
/// path length in bytes excluding the null terminator. Returns `0` on failure.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn DragQueryFileA(
    hDrop: HDROP,
    iFile: u32,
    lpszFile: LPSTR,
    cch: u32,
) -> u32 {
    unsafe {
        common::drag_query_file(
            hDrop,
            iFile,
            common::DragQueryFileBuffer::Ansi {
                lpsz_file: lpszFile,
                cch,
            },
        )
    }
}

/// Queries file paths from an `HDROP` handle.
///
/// # Arguments
/// * `hDrop` - Handle identifying the dropped-file list.
/// * `iFile` - File index, or `0xFFFFFFFF` to query file count.
/// * `lpszFile` - Optional output buffer for the file path.
/// * `cch` - Output buffer size in bytes.
///
/// # Safety
/// `hDrop` must refer to a valid DROPFILES memory block. When `lpszFile`
/// is non-null, it must reference writable memory of at least `cch` bytes.
///
/// # Return
/// Returns file count when `iFile == 0xFFFFFFFF`; otherwise returns the selected
/// path length in bytes excluding the null terminator. Returns `0` on failure.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn DragQueryFileW(
    hDrop: HDROP,
    iFile: u32,
    lpszFile: LPWSTR,
    cch: u32,
) -> u32 {
    unsafe {
        common::drag_query_file(
            hDrop,
            iFile,
            common::DragQueryFileBuffer::Wide {
                lpsz_file: lpszFile,
                cch,
            },
        )
    }
}
