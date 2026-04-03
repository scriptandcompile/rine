#![allow(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_void};
use core::sync::atomic::{AtomicU32, Ordering};
use std::path::PathBuf;

use rine_dlls::{DllPlugin, Export, as_win_api};
use rine_types::dev_hooks::{DialogOpenTelemetry, DialogResultTelemetry};
use rine_types::dev_notify;
use rine_types::strings::{read_cstr, read_wstr, write_cstr, write_wstr};

pub struct Comdlg32Plugin;

impl DllPlugin for Comdlg32Plugin {
    fn dll_names(&self) -> &[&str] {
        &["comdlg32.dll"]
    }

    fn exports(&self) -> Vec<Export> {
        vec![
            Export::Func("GetOpenFileNameA", as_win_api!(get_open_file_name_a)),
            Export::Func("GetOpenFileNameW", as_win_api!(get_open_file_name_w)),
            Export::Func("GetSaveFileNameA", as_win_api!(get_save_file_name_a)),
            Export::Func("GetSaveFileNameW", as_win_api!(get_save_file_name_w)),
            Export::Func("CommDlgExtendedError", as_win_api!(comm_dlg_extended_error)),
        ]
    }
}

// Common dialogs store extended error state process-wide in this first pass.
static LAST_ERROR: AtomicU32 = AtomicU32::new(0);

// Common dialog extended error codes (subset).
const CDERR_INITIALIZATION: u32 = 0x0002;
const CDERR_DIALOGFAILURE: u32 = 0xFFFF;
const FNERR_BUFFERTOOSMALL: u32 = 0x3003;

#[allow(non_snake_case)]
#[repr(C)]
struct OpenFileNameA {
    lStructSize: u32,
    hwndOwner: usize,
    hInstance: usize,
    lpstrFilter: *const c_char,
    lpstrCustomFilter: *mut c_char,
    nMaxCustFilter: u32,
    nFilterIndex: u32,
    lpstrFile: *mut c_char,
    nMaxFile: u32,
    lpstrFileTitle: *mut c_char,
    nMaxFileTitle: u32,
    lpstrInitialDir: *const c_char,
    lpstrTitle: *const c_char,
    Flags: u32,
    nFileOffset: u16,
    nFileExtension: u16,
    lpstrDefExt: *const c_char,
    lCustData: isize,
    lpfnHook: usize,
    lpTemplateName: *const c_char,
    pvReserved: *mut c_void,
    dwReserved: u32,
    FlagsEx: u32,
}

#[allow(non_snake_case)]
#[repr(C)]
struct OpenFileNameW {
    lStructSize: u32,
    hwndOwner: usize,
    hInstance: usize,
    lpstrFilter: *const u16,
    lpstrCustomFilter: *mut u16,
    nMaxCustFilter: u32,
    nFilterIndex: u32,
    lpstrFile: *mut u16,
    nMaxFile: u32,
    lpstrFileTitle: *mut u16,
    nMaxFileTitle: u32,
    lpstrInitialDir: *const u16,
    lpstrTitle: *const u16,
    Flags: u32,
    nFileOffset: u16,
    nFileExtension: u16,
    lpstrDefExt: *const u16,
    lCustData: isize,
    lpfnHook: usize,
    lpTemplateName: *const u16,
    pvReserved: *mut c_void,
    dwReserved: u32,
    FlagsEx: u32,
}

#[derive(Clone, Copy)]
enum DialogTheme {
    Native,
    Windows,
}

#[derive(Clone, Copy)]
enum DialogKind {
    Open,
    Save,
}

fn current_theme() -> DialogTheme {
    if let Ok(v) = std::env::var("RINE_DIALOG_THEME") {
        if v.eq_ignore_ascii_case("windows") || v.eq_ignore_ascii_case("emulated") {
            return DialogTheme::Windows;
        }
        return DialogTheme::Native;
    }

    // Backward compatibility for older env key.
    match std::env::var("RINE_DIALOG_MODE") {
        Ok(v) if v.eq_ignore_ascii_case("emulated") || v.eq_ignore_ascii_case("windows") => {
            DialogTheme::Windows
        }
        _ => DialogTheme::Native,
    }
}

fn current_native_backend() -> &'static str {
    match std::env::var("RINE_DIALOG_NATIVE_BACKEND") {
        Ok(v) if v.eq_ignore_ascii_case("gtk") => "gtk",
        Ok(v) if v.eq_ignore_ascii_case("kde") => "kde",
        Ok(v) if v.eq_ignore_ascii_case("portal") || v.eq_ignore_ascii_case("auto") => "portal",
        _ => "portal",
    }
}

fn current_emulated_theme() -> &'static str {
    match std::env::var("RINE_DIALOG_EMULATED_THEME") {
        Ok(v) if v.eq_ignore_ascii_case("xp") => "xp",
        Ok(v) if v.eq_ignore_ascii_case("win7") => "win7",
        Ok(v) if v.eq_ignore_ascii_case("win10") => "win10",
        Ok(v) if v.eq_ignore_ascii_case("win11") => "win11",
        Ok(v) if v.eq_ignore_ascii_case("windows_version") || v.eq_ignore_ascii_case("auto") => {
            "windows_version"
        }
        _ => "windows_version",
    }
}

fn theme_label(theme: DialogTheme) -> &'static str {
    match theme {
        DialogTheme::Native => "native",
        DialogTheme::Windows => "windows",
    }
}

fn emit_dialog_opened(api: &str, theme: DialogTheme) {
    dev_notify!(on_dialog_opened(DialogOpenTelemetry {
        api,
        theme: theme_label(theme),
        native_backend: current_native_backend(),
        windows_theme: current_emulated_theme(),
    }));
}

fn emit_dialog_result(
    api: &str,
    theme: DialogTheme,
    success: bool,
    error_code: u32,
    selected: Option<&str>,
) {
    dev_notify!(on_dialog_result(DialogResultTelemetry {
        api,
        theme: theme_label(theme),
        native_backend: current_native_backend(),
        windows_theme: current_emulated_theme(),
        success,
        error_code,
        selected_path: selected,
    }));
}

fn pick_path(
    kind: DialogKind,
    title: Option<String>,
    initial_dir: Option<String>,
) -> Option<PathBuf> {
    // Test hook: bypass UI and return a deterministic path when requested.
    if let Ok(path) = std::env::var("RINE_DIALOG_TEST_PATH")
        && !path.is_empty()
    {
        return Some(PathBuf::from(path));
    }

    let mut dialog = rfd::FileDialog::new();
    if let Some(title) = title {
        dialog = dialog.set_title(&title);
    }
    if let Some(dir) = initial_dir {
        let path = PathBuf::from(dir);
        if path.is_dir() {
            dialog = dialog.set_directory(path);
        }
    }

    match kind {
        DialogKind::Open => dialog.pick_file(),
        DialogKind::Save => dialog.save_file(),
    }
}

fn update_offsets(path: &str) -> (u16, u16) {
    let file_offset = path
        .rfind(['/', '\\'])
        .map(|idx| idx + 1)
        .unwrap_or(0)
        .min(u16::MAX as usize) as u16;

    let file_extension = path[file_offset as usize..]
        .rfind('.')
        .map(|dot| file_offset as usize + dot + 1)
        .unwrap_or(0)
        .min(u16::MAX as usize) as u16;

    (file_offset, file_extension)
}

unsafe extern "win64" fn get_open_file_name_a(open_file_name: *mut c_void) -> i32 {
    run_a_dialog(open_file_name as *mut OpenFileNameA, DialogKind::Open)
}

unsafe extern "win64" fn get_open_file_name_w(open_file_name: *mut c_void) -> i32 {
    run_w_dialog(open_file_name as *mut OpenFileNameW, DialogKind::Open)
}

unsafe extern "win64" fn get_save_file_name_a(open_file_name: *mut c_void) -> i32 {
    run_a_dialog(open_file_name as *mut OpenFileNameA, DialogKind::Save)
}

unsafe extern "win64" fn get_save_file_name_w(open_file_name: *mut c_void) -> i32 {
    run_w_dialog(open_file_name as *mut OpenFileNameW, DialogKind::Save)
}

unsafe extern "win64" fn comm_dlg_extended_error() -> u32 {
    LAST_ERROR.load(Ordering::Relaxed)
}

unsafe fn run_a_dialog(ofn: *mut OpenFileNameA, kind: DialogKind) -> i32 {
    let api_name = match kind {
        DialogKind::Open => "GetOpenFileNameA",
        DialogKind::Save => "GetSaveFileNameA",
    };
    let theme = current_theme();
    emit_dialog_opened(api_name, theme);

    if ofn.is_null() {
        LAST_ERROR.store(CDERR_INITIALIZATION, Ordering::Relaxed);
        emit_dialog_result(api_name, theme, false, CDERR_INITIALIZATION, None);
        return 0;
    }
    // SAFETY: Pointer is checked for null above.
    let ofn = unsafe { &mut *ofn };
    if (ofn.lStructSize as usize) < core::mem::size_of::<OpenFileNameA>() {
        LAST_ERROR.store(CDERR_INITIALIZATION, Ordering::Relaxed);
        emit_dialog_result(api_name, theme, false, CDERR_INITIALIZATION, None);
        return 0;
    }

    if matches!(theme, DialogTheme::Windows) {
        LAST_ERROR.store(CDERR_DIALOGFAILURE, Ordering::Relaxed);
        emit_dialog_result(api_name, theme, false, CDERR_DIALOGFAILURE, None);
        return 0;
    }

    // SAFETY: OPENFILENAME fields are null or valid NUL-terminated strings.
    let chosen = pick_path(kind, unsafe { read_cstr(ofn.lpstrTitle.cast()) }, unsafe {
        read_cstr(ofn.lpstrInitialDir.cast())
    });
    let Some(path) = chosen else {
        LAST_ERROR.store(0, Ordering::Relaxed);
        emit_dialog_result(api_name, theme, false, 0, None);
        return 0;
    };
    let path = path.to_string_lossy().into_owned();

    if ofn.lpstrFile.is_null() || ofn.nMaxFile == 0 {
        LAST_ERROR.store(CDERR_INITIALIZATION, Ordering::Relaxed);
        emit_dialog_result(api_name, theme, false, CDERR_INITIALIZATION, None);
        return 0;
    }

    // SAFETY: lpstrFile points to caller-owned buffer of nMaxFile bytes.
    let written = unsafe { write_cstr(ofn.lpstrFile.cast(), ofn.nMaxFile, &path) };
    if written + 1 > ofn.nMaxFile {
        LAST_ERROR.store(FNERR_BUFFERTOOSMALL, Ordering::Relaxed);
        emit_dialog_result(api_name, theme, false, FNERR_BUFFERTOOSMALL, None);
        return 0;
    }

    let (off, ext) = update_offsets(&path);
    ofn.nFileOffset = off;
    ofn.nFileExtension = ext;
    LAST_ERROR.store(0, Ordering::Relaxed);
    emit_dialog_result(api_name, theme, true, 0, Some(&path));
    1
}

unsafe fn run_w_dialog(ofn: *mut OpenFileNameW, kind: DialogKind) -> i32 {
    let api_name = match kind {
        DialogKind::Open => "GetOpenFileNameW",
        DialogKind::Save => "GetSaveFileNameW",
    };
    let theme = current_theme();
    emit_dialog_opened(api_name, theme);

    if ofn.is_null() {
        LAST_ERROR.store(CDERR_INITIALIZATION, Ordering::Relaxed);
        emit_dialog_result(api_name, theme, false, CDERR_INITIALIZATION, None);
        return 0;
    }
    // SAFETY: Pointer is checked for null above.
    let ofn = unsafe { &mut *ofn };
    if (ofn.lStructSize as usize) < core::mem::size_of::<OpenFileNameW>() {
        LAST_ERROR.store(CDERR_INITIALIZATION, Ordering::Relaxed);
        emit_dialog_result(api_name, theme, false, CDERR_INITIALIZATION, None);
        return 0;
    }

    if matches!(theme, DialogTheme::Windows) {
        LAST_ERROR.store(CDERR_DIALOGFAILURE, Ordering::Relaxed);
        emit_dialog_result(api_name, theme, false, CDERR_DIALOGFAILURE, None);
        return 0;
    }

    // SAFETY: OPENFILENAME fields are null or valid NUL-terminated UTF-16 strings.
    let chosen = pick_path(kind, unsafe { read_wstr(ofn.lpstrTitle) }, unsafe {
        read_wstr(ofn.lpstrInitialDir)
    });
    let Some(path) = chosen else {
        LAST_ERROR.store(0, Ordering::Relaxed);
        emit_dialog_result(api_name, theme, false, 0, None);
        return 0;
    };
    let path = path.to_string_lossy().into_owned();

    if ofn.lpstrFile.is_null() || ofn.nMaxFile == 0 {
        LAST_ERROR.store(CDERR_INITIALIZATION, Ordering::Relaxed);
        emit_dialog_result(api_name, theme, false, CDERR_INITIALIZATION, None);
        return 0;
    }

    // SAFETY: lpstrFile points to caller-owned buffer of nMaxFile UTF-16 units.
    let written = unsafe { write_wstr(ofn.lpstrFile, ofn.nMaxFile, &path) };
    if written + 1 > ofn.nMaxFile {
        LAST_ERROR.store(FNERR_BUFFERTOOSMALL, Ordering::Relaxed);
        emit_dialog_result(api_name, theme, false, FNERR_BUFFERTOOSMALL, None);
        return 0;
    }

    let (off, ext) = update_offsets(&path);
    ofn.nFileOffset = off;
    ofn.nFileExtension = ext;
    LAST_ERROR.store(0, Ordering::Relaxed);
    emit_dialog_result(api_name, theme, true, 0, Some(&path));
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offsets_are_computed() {
        let (off, ext) = update_offsets("C:\\games\\foo.exe");
        assert_eq!(off, 9);
        assert_eq!(ext, 13);
    }
}
