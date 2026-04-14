pub mod env_policy;
pub mod error;
pub mod open;
pub mod pick;
pub mod save;
pub mod telemetry;

use std::ffi::{c_char, c_void};
//use std::sync::OnceLock;

pub use env_policy::{
    DialogPolicy, DialogTheme, NativeBackend, WindowsTheme, resolve_dialog_policy,
};
pub use telemetry::{DialogOpenFields, DialogResultFields};

//static BACKEND_MISSING_WARNED: OnceLock<()> = OnceLock::new();

#[allow(non_snake_case)]
#[repr(C)]
pub struct OpenFileNameA {
    pub lStructSize: u32,
    pub hwndOwner: usize,
    pub hInstance: usize,
    pub lpstrFilter: *const c_char,
    pub lpstrCustomFilter: *mut c_char,
    pub nMaxCustFilter: u32,
    pub nFilterIndex: u32,
    pub lpstrFile: *mut c_char,
    pub nMaxFile: u32,
    pub lpstrFileTitle: *mut c_char,
    pub nMaxFileTitle: u32,
    pub lpstrInitialDir: *const c_char,
    pub lpstrTitle: *const c_char,
    pub Flags: u32,
    pub nFileOffset: u16,
    pub nFileExtension: u16,
    pub lpstrDefExt: *const c_char,
    pub lCustData: isize,
    pub lpfnHook: usize,
    pub lpTemplateName: *const c_char,
    pub pvReserved: *mut c_void,
    pub dwReserved: u32,
    pub FlagsEx: u32,
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct OpenFileNameW {
    pub lStructSize: u32,
    pub hwndOwner: usize,
    pub hInstance: usize,
    pub lpstrFilter: *const u16,
    pub lpstrCustomFilter: *mut u16,
    pub nMaxCustFilter: u32,
    pub nFilterIndex: u32,
    pub lpstrFile: *mut u16,
    pub nMaxFile: u32,
    pub lpstrFileTitle: *mut u16,
    pub nMaxFileTitle: u32,
    pub lpstrInitialDir: *const u16,
    pub lpstrTitle: *const u16,
    pub Flags: u32,
    pub nFileOffset: u16,
    pub nFileExtension: u16,
    pub lpstrDefExt: *const u16,
    pub lCustData: isize,
    pub lpfnHook: usize,
    pub lpTemplateName: *const u16,
    pub pvReserved: *mut c_void,
    pub dwReserved: u32,
    pub FlagsEx: u32,
}

/// Updates the `nFileOffset` and `nFileExtension` fields of the given
/// `OPENFILENAME` struct based on the current value of the `lpstrFile` field.
///
/// # Arguments
/// * `path`: The file path string from which to compute the offsets.
///   This should be the same string that is pointed to by the `lpstrFile` field of the `OPENFILENAME` struct.
///
/// # Returns
/// A tuple containing the computed `nFileOffset` and `nFileExtension` values.
/// The `nFileOffset` is the index of the last path separator (`\` or `/`) plus one, or zero if no path
/// separator is found.
/// The `nFileExtension` is the index of the last dot (`.`) in the file name (after the last path separator)
/// plus one, or zero if no dot is found. Both values are capped at `u16::MAX` to fit within the `u16`
/// fields of the struct.
pub(crate) fn update_offsets(path: &str) -> (u16, u16) {
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
