use core::ptr;

use rine_types::strings::{read_cstr, read_wstr, write_cstr, write_wstr};

use crate::env_policy::{DialogTheme, resolve_dialog_policy};
use crate::error::{DialogErrorCode, set_last_error};
use crate::pick::pick_open;
use crate::update_offsets;
use crate::{OpenFileNameA, OpenFileNameW};

/// GetOpenFileNameA — open dialog for ANSI paths.
///
/// # Arguments
/// * `ofn`: A pointer to an `OPENFILENAMEA` struct that contains information used to initialize
///   the dialog and receives the results of the user's selection.
///
/// # Safety
/// `ofn` must be null or a valid pointer to an `OPENFILENAMEA` whose string
/// fields are null or point to valid NUL-terminated strings.
pub unsafe fn open_file_name_a(ofn: *mut OpenFileNameA) -> i32 {
    if ofn.is_null() {
        set_last_error(DialogErrorCode::CderrInitialization);
        return 0;
    }

    // Use read_unaligned so 32-bit callers with byte-aligned structs are safe.
    let mut local: OpenFileNameA = unsafe { ptr::read_unaligned(ofn) };

    if (local.lStructSize as usize) < core::mem::size_of::<OpenFileNameA>() {
        set_last_error(DialogErrorCode::CderrInitialization);
        return 0;
    }

    if matches!(resolve_dialog_policy().theme, DialogTheme::Windows) {
        set_last_error(DialogErrorCode::CderrDialogFailure);
        return 0;
    }

    let title = unsafe { read_cstr(local.lpstrTitle.cast()) };
    let initial_dir = unsafe { read_cstr(local.lpstrInitialDir.cast()) };

    let Some(path) = pick_open(title, initial_dir) else {
        set_last_error(DialogErrorCode::None);
        return 0;
    };
    let path = path.to_string_lossy().into_owned();

    if local.lpstrFile.is_null() || local.nMaxFile == 0 {
        set_last_error(DialogErrorCode::CderrInitialization);
        return 0;
    }

    let written = unsafe { write_cstr(local.lpstrFile.cast(), local.nMaxFile, &path) };
    if written + 1 > local.nMaxFile {
        set_last_error(DialogErrorCode::FnerrBufferTooSmall);
        unsafe { ptr::write_unaligned(ofn, local) };
        return 0;
    }

    let (off, ext) = update_offsets(&path);
    local.nFileOffset = off;
    local.nFileExtension = ext;

    unsafe { ptr::write_unaligned(ofn, local) };
    set_last_error(DialogErrorCode::None);
    1
}

/// GetOpenFileNameW — open dialog for UTF-16 paths.
///
/// # Arguments
/// * `ofn`: A pointer to an `OPENFILENAMEW` struct that contains information used to initialize
///   the dialog and receives the results of the user's selection.
///
/// # Safety
/// `ofn` must be null or a valid pointer to an `OPENFILENAMEW` whose string
/// fields are null or point to valid NUL-terminated UTF-16 strings.
pub unsafe fn open_file_name_w(ofn: *mut OpenFileNameW) -> i32 {
    if ofn.is_null() {
        set_last_error(DialogErrorCode::CderrInitialization);
        return 0;
    }

    let mut local: OpenFileNameW = unsafe { ptr::read_unaligned(ofn) };

    if (local.lStructSize as usize) < core::mem::size_of::<OpenFileNameW>() {
        set_last_error(DialogErrorCode::CderrInitialization);
        return 0;
    }

    if matches!(resolve_dialog_policy().theme, DialogTheme::Windows) {
        set_last_error(DialogErrorCode::CderrDialogFailure);
        return 0;
    }

    let title = unsafe { read_wstr(local.lpstrTitle) };
    let initial_dir = unsafe { read_wstr(local.lpstrInitialDir) };

    let Some(path) = pick_open(title, initial_dir) else {
        set_last_error(DialogErrorCode::None);
        return 0;
    };
    let path = path.to_string_lossy().into_owned();

    if local.lpstrFile.is_null() || local.nMaxFile == 0 {
        set_last_error(DialogErrorCode::CderrInitialization);
        return 0;
    }

    let written = unsafe { write_wstr(local.lpstrFile, local.nMaxFile, &path) };
    if written + 1 > local.nMaxFile {
        set_last_error(DialogErrorCode::FnerrBufferTooSmall);
        unsafe { ptr::write_unaligned(ofn, local) };
        return 0;
    }

    let (off, ext) = update_offsets(&path);
    local.nFileOffset = off;
    local.nFileExtension = ext;

    unsafe { ptr::write_unaligned(ofn, local) };
    set_last_error(DialogErrorCode::None);
    1
}
