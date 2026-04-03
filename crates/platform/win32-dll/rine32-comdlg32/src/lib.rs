#![allow(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_void};
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

use rine_common_comdlg32::{
    DialogAdapter, DialogErrorCode, DialogKind, DialogPolicy, last_error, resolve_dialog_policy,
    run_dialog_flow,
};
use rine_dlls::{DllPlugin, Export, as_win_api};
use rine_types::dev_hooks::{DialogOpenTelemetry, DialogResultTelemetry};
use rine_types::dev_notify;
use rine_types::strings::{read_cstr, read_wstr, write_cstr, write_wstr};
use tracing::warn;

#[cfg(not(target_pointer_width = "32"))]
compile_error!(
    "crate `rine32-comdlg32` must be built for a 32-bit target (for example: --target i686-unknown-linux-gnu)"
);

pub struct Comdlg32Plugin32;

static BACKEND_MISSING_WARNED: OnceLock<()> = OnceLock::new();

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

    let mut backend_available = false;

    match pick_with_zenity(kind, title.as_deref(), initial_dir.as_deref()) {
        PickerResult::Selected(path) => return Some(path),
        PickerResult::BackendAvailableNoSelection => backend_available = true,
        PickerResult::BackendUnavailable => {}
    }

    match pick_with_kdialog(kind, title.as_deref(), initial_dir.as_deref()) {
        PickerResult::Selected(path) => return Some(path),
        PickerResult::BackendAvailableNoSelection => backend_available = true,
        PickerResult::BackendUnavailable => {}
    }

    // Only emit a backend-missing warning when no backend was available at all.
    // A user cancel or dialog runtime error should not be reported as "install zenity/kdialog".
    if !backend_available {
        emit_backend_missing_warning_once();
    }

    None
}

enum PickerResult {
    Selected(PathBuf),
    BackendAvailableNoSelection,
    BackendUnavailable,
}

fn emit_backend_missing_warning_once() {
    if BACKEND_MISSING_WARNED.set(()).is_err() {
        return;
    }

    warn!(
        "win32 dialog backend unavailable: neither `zenity` nor `kdialog` could be used; \
         install one of them for 32-bit file picker dialogs"
    );

    let policy = resolve_dialog_policy();
    let fields = rine_common_comdlg32::telemetry::build_result_fields(
        "DialogBackendProbe",
        policy,
        false,
        DialogErrorCode::CderrDialogFailure as u32,
        None,
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

/// Attempt to pick a file path using `zenity`. Returns `None` if `zenity` is not available or the dialog fails/is cancelled.
///
/// We use 'zenity' on GTK desktops when in 32-bit mode since the 'rfd' crate does not support 32-bit Linux targets, and
/// 'zenity' is a widely-available native dialog tool that works well in this scenario.
fn pick_with_zenity(
    kind: DialogKind,
    title: Option<&str>,
    initial_dir: Option<&str>,
) -> PickerResult {
    let mut cmd = Command::new("zenity");
    cmd.arg("--file-selection");

    if matches!(kind, DialogKind::Save) {
        cmd.arg("--save").arg("--confirm-overwrite");
    }

    if let Some(title) = title
        && !title.is_empty()
    {
        cmd.arg("--title").arg(title);
    }

    if let Some(dir) = initial_dir
        && !dir.is_empty()
    {
        cmd.arg("--filename").arg(format!("{dir}/"));
    }

    let output = match cmd.output() {
        Ok(output) => output,
        Err(_) => return PickerResult::BackendUnavailable,
    };

    // `zenity` was found and executed; non-zero status typically means
    // cancel or runtime display error, not missing backend.
    if !output.status.success() {
        return PickerResult::BackendAvailableNoSelection;
    }

    let selected = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if selected.is_empty() {
        return PickerResult::BackendAvailableNoSelection;
    }

    PickerResult::Selected(PathBuf::from(selected))
}

/// Attempt to pick a file path using `kdialog`. Returns `None` if `kdialog` is not available or the dialog fails/is cancelled.
///
/// We use 'kdialog' on KDE desktops when in 32-bit mode since the 'rfd' crate does not support 32-bit Linux targets, and
/// 'kdialog' is a widely-available native dialog tool that works well in this scenario.
fn pick_with_kdialog(
    kind: DialogKind,
    title: Option<&str>,
    initial_dir: Option<&str>,
) -> PickerResult {
    let mut cmd = Command::new("kdialog");

    match kind {
        DialogKind::Open => {
            cmd.arg("--getopenfilename");
        }
        DialogKind::Save => {
            cmd.arg("--getsavefilename");
        }
    }

    cmd.arg(initial_dir.unwrap_or("."));

    if let Some(title) = title
        && !title.is_empty()
    {
        cmd.arg("--title").arg(title);
    }

    let output = match cmd.output() {
        Ok(output) => output,
        Err(_) => return PickerResult::BackendUnavailable,
    };

    // `kdialog` was found and executed; non-zero status typically means
    // cancel or runtime display error, not missing backend.
    if !output.status.success() {
        return PickerResult::BackendAvailableNoSelection;
    }

    let selected = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if selected.is_empty() {
        return PickerResult::BackendAvailableNoSelection;
    }

    PickerResult::Selected(PathBuf::from(selected))
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
    if ofn.is_null() {
        return run_dialog_flow::<OpenFileNameAAdapter<'_>, _, _, _>(
            api_name,
            kind,
            None::<OpenFileNameAAdapter<'_>>,
            pick_path,
            emit_dialog_opened,
            emit_dialog_result,
        );
    }

    // x86 PE callers may provide only byte alignment for the struct address.
    // Read/write it using unaligned operations and work on an aligned local copy.
    let mut local = unsafe { core::ptr::read_unaligned(ofn) };
    let adapter = Some(OpenFileNameAAdapter { ofn: &mut local });
    let result = run_dialog_flow(
        api_name,
        kind,
        adapter,
        pick_path,
        emit_dialog_opened,
        emit_dialog_result,
    );
    unsafe { core::ptr::write_unaligned(ofn, local) };
    result
}

unsafe fn run_w_dialog(ofn: *mut OpenFileNameW, kind: DialogKind) -> i32 {
    let api_name = match kind {
        DialogKind::Open => "GetOpenFileNameW",
        DialogKind::Save => "GetSaveFileNameW",
    };
    if ofn.is_null() {
        return run_dialog_flow::<OpenFileNameWAdapter<'_>, _, _, _>(
            api_name,
            kind,
            None::<OpenFileNameWAdapter<'_>>,
            pick_path,
            emit_dialog_opened,
            emit_dialog_result,
        );
    }

    // x86 PE callers may provide only byte alignment for the struct address.
    // Read/write it using unaligned operations and work on an aligned local copy.
    let mut local = unsafe { core::ptr::read_unaligned(ofn) };
    let adapter = Some(OpenFileNameWAdapter { ofn: &mut local });
    let result = run_dialog_flow(
        api_name,
        kind,
        adapter,
        pick_path,
        emit_dialog_opened,
        emit_dialog_result,
    );
    unsafe { core::ptr::write_unaligned(ofn, local) };
    result
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
