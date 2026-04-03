#![allow(unsafe_op_in_unsafe_fn)]

use core::ffi::c_void;
use core::sync::atomic::{AtomicU32, Ordering};

use rine_dlls::{DllPlugin, Export, as_win_api};

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

unsafe extern "win64" fn get_open_file_name_a(_open_file_name: *mut c_void) -> i32 {
    // First-step behavior: signal user-cancel/no selection.
    LAST_ERROR.store(0, Ordering::Relaxed);
    0
}

unsafe extern "win64" fn get_open_file_name_w(_open_file_name: *mut c_void) -> i32 {
    LAST_ERROR.store(0, Ordering::Relaxed);
    0
}

unsafe extern "win64" fn get_save_file_name_a(_open_file_name: *mut c_void) -> i32 {
    LAST_ERROR.store(0, Ordering::Relaxed);
    0
}

unsafe extern "win64" fn get_save_file_name_w(_open_file_name: *mut c_void) -> i32 {
    LAST_ERROR.store(0, Ordering::Relaxed);
    0
}

unsafe extern "win64" fn comm_dlg_extended_error() -> u32 {
    LAST_ERROR.load(Ordering::Relaxed)
}
