#![allow(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_void};
use std::path::PathBuf;

use rine_common_comdlg32::{
    DialogAdapter, DialogErrorCode, DialogKind, DialogPolicy, last_error, run_dialog_flow,
};
use rine_dlls::{DllPlugin, Export, as_win_api};
use rine_types::dev_hooks::{DialogOpenTelemetry, DialogResultTelemetry};
use rine_types::dev_notify;
use rine_types::strings::{read_cstr, read_wstr, write_cstr, write_wstr};

#[cfg(not(target_pointer_width = "32"))]
compile_error!(
    "crate `rine32-comdlg32` must be built for a 32-bit target (for example: --target i686-unknown-linux-gnu)"
);

pub struct Comdlg32Plugin32;

impl DllPlugin for Comdlg32Plugin32 {
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

fn emit_dialog_opened(api: &'static str, policy: DialogPolicy) {
    let fields = rine_common_comdlg32::telemetry::build_open_fields(api, policy);
    dev_notify!(on_dialog_opened(DialogOpenTelemetry {
        api: fields.api,
        theme: fields.theme,
        native_backend: fields.native_backend,
        windows_theme: fields.windows_theme,
    }));
}

fn emit_dialog_result(
    api: &'static str,
    policy: DialogPolicy,
    success: bool,
    error_code: u32,
    selected: Option<&str>,
) {
    let fields = rine_common_comdlg32::telemetry::build_result_fields(
        api,
        policy,
        success,
        error_code,
        selected.map(str::to_owned),
    );
    dev_notify!(on_dialog_result(DialogResultTelemetry {
        api: fields.api,
        theme: fields.theme,
        native_backend: fields.native_backend,
        windows_theme: fields.windows_theme,
        success: fields.success,
        error_code: fields.error_code,
        selected_path: fields.selected_path.as_deref(),
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

unsafe extern "C" fn get_open_file_name_a(open_file_name: *mut c_void) -> i32 {
    run_a_dialog(open_file_name as *mut OpenFileNameA, DialogKind::Open)
}

unsafe extern "C" fn get_open_file_name_w(open_file_name: *mut c_void) -> i32 {
    run_w_dialog(open_file_name as *mut OpenFileNameW, DialogKind::Open)
}

unsafe extern "C" fn get_save_file_name_a(open_file_name: *mut c_void) -> i32 {
    run_a_dialog(open_file_name as *mut OpenFileNameA, DialogKind::Save)
}

unsafe extern "C" fn get_save_file_name_w(open_file_name: *mut c_void) -> i32 {
    run_w_dialog(open_file_name as *mut OpenFileNameW, DialogKind::Save)
}

unsafe extern "C" fn comm_dlg_extended_error() -> u32 {
    last_error()
}

unsafe fn run_a_dialog(ofn: *mut OpenFileNameA, kind: DialogKind) -> i32 {
    let api_name = match kind {
        DialogKind::Open => "GetOpenFileNameA",
        DialogKind::Save => "GetSaveFileNameA",
    };
    let adapter = if ofn.is_null() {
        None
    } else {
        // SAFETY: pointer is checked for null above.
        Some(OpenFileNameAAdapter {
            ofn: unsafe { &mut *ofn },
        })
    };

    run_dialog_flow(
        api_name,
        kind,
        adapter,
        pick_path,
        emit_dialog_opened,
        emit_dialog_result,
    )
}

unsafe fn run_w_dialog(ofn: *mut OpenFileNameW, kind: DialogKind) -> i32 {
    let api_name = match kind {
        DialogKind::Open => "GetOpenFileNameW",
        DialogKind::Save => "GetSaveFileNameW",
    };
    let adapter = if ofn.is_null() {
        None
    } else {
        // SAFETY: pointer is checked for null above.
        Some(OpenFileNameWAdapter {
            ofn: unsafe { &mut *ofn },
        })
    };

    run_dialog_flow(
        api_name,
        kind,
        adapter,
        pick_path,
        emit_dialog_opened,
        emit_dialog_result,
    )
}

struct OpenFileNameAAdapter<'a> {
    ofn: &'a mut OpenFileNameA,
}

impl DialogAdapter for OpenFileNameAAdapter<'_> {
    fn struct_size_valid(&self) -> bool {
        (self.ofn.lStructSize as usize) >= core::mem::size_of::<OpenFileNameA>()
    }

    fn title(&self) -> Option<String> {
        // SAFETY: OPENFILENAME fields are null or valid NUL-terminated strings.
        unsafe { read_cstr(self.ofn.lpstrTitle.cast()) }
    }

    fn initial_dir(&self) -> Option<String> {
        // SAFETY: OPENFILENAME fields are null or valid NUL-terminated strings.
        unsafe { read_cstr(self.ofn.lpstrInitialDir.cast()) }
    }

    fn has_output_buffer(&self) -> bool {
        !self.ofn.lpstrFile.is_null() && self.ofn.nMaxFile > 0
    }

    fn write_selected_path(&mut self, path: &str) -> Result<(), DialogErrorCode> {
        // SAFETY: lpstrFile points to caller-owned buffer of nMaxFile bytes.
        let written = unsafe { write_cstr(self.ofn.lpstrFile.cast(), self.ofn.nMaxFile, path) };
        if written + 1 > self.ofn.nMaxFile {
            return Err(DialogErrorCode::FnerrBufferTooSmall);
        }
        Ok(())
    }

    fn set_name_offsets(&mut self, file_offset: u16, file_extension: u16) {
        self.ofn.nFileOffset = file_offset;
        self.ofn.nFileExtension = file_extension;
    }
}

struct OpenFileNameWAdapter<'a> {
    ofn: &'a mut OpenFileNameW,
}

impl DialogAdapter for OpenFileNameWAdapter<'_> {
    fn struct_size_valid(&self) -> bool {
        (self.ofn.lStructSize as usize) >= core::mem::size_of::<OpenFileNameW>()
    }

    fn title(&self) -> Option<String> {
        // SAFETY: OPENFILENAME fields are null or valid NUL-terminated UTF-16 strings.
        unsafe { read_wstr(self.ofn.lpstrTitle) }
    }

    fn initial_dir(&self) -> Option<String> {
        // SAFETY: OPENFILENAME fields are null or valid NUL-terminated UTF-16 strings.
        unsafe { read_wstr(self.ofn.lpstrInitialDir) }
    }

    fn has_output_buffer(&self) -> bool {
        !self.ofn.lpstrFile.is_null() && self.ofn.nMaxFile > 0
    }

    fn write_selected_path(&mut self, path: &str) -> Result<(), DialogErrorCode> {
        // SAFETY: lpstrFile points to caller-owned buffer of nMaxFile UTF-16 units.
        let written = unsafe { write_wstr(self.ofn.lpstrFile, self.ofn.nMaxFile, path) };
        if written + 1 > self.ofn.nMaxFile {
            return Err(DialogErrorCode::FnerrBufferTooSmall);
        }
        Ok(())
    }

    fn set_name_offsets(&mut self, file_offset: u16, file_extension: u16) {
        self.ofn.nFileOffset = file_offset;
        self.ofn.nFileExtension = file_extension;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn offsets_are_computed() {
        let (off, ext) = rine_common_comdlg32::update_offsets("C:\\games\\foo.exe");
        assert_eq!(off, 9);
        assert_eq!(ext, 13);
    }
}
