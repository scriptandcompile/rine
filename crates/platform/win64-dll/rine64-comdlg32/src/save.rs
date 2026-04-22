use rine_common_comdlg32 as common;

/// GetSaveFileNameA implementation.
///
/// # Arguments
/// - `ofn`: Pointer to an `OPENFILENAMEA` struct that contains information used to initialize the
///   dialog and receives information about the user's selection. Yes, this is the same struct as
///   `GetOpenFileNameA`.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `ofn` is a valid pointer to an `OPENFILENAMEA` struct and that the
/// memory it points to is properly initialized and writable. Additionally, the caller must ensure
/// that the struct's fields are set according to the expected format, as invalid values may lead
/// to undefined behavior. The caller is also responsible for ensuring that the dialog is used in
/// a compatible environment, as certain fields may have specific requirements or may not be
/// supported in all contexts. Finally, the caller must handle the dialog's behavior correctly,
/// as improper use may lead to resource leaks or other issues.
///
/// # Returns
/// Returns a nonzero value if the user clicks the OK button and successfully selects a file,
/// or zero if the user cancels the dialog or an error occurs. If the function fails, extended
/// error information can be retrieved by calling `CommDlgExtendedError`.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetSaveFileNameA(ofn: *mut common::OpenFileNameA) -> i32 {
    let policy = common::telemetry::emit_opened("GetSaveFileNameA");
    let result = common::save::save_file_name_a(ofn);
    common::telemetry::emit_result("GetSaveFileNameA", policy, result);
    result
}

/// GetSaveFileNameW implementation.
///
/// # Arguments
/// - `ofn`: Pointer to an `OPENFILENAMEW` struct that contains information used to initialize the
///   dialog and receives information about the user's selection. Yes, this is the same struct as
///   `GetOpenFileNameW`.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `ofn` is a valid pointer to an `OPENFILENAMEW` struct and that the
/// memory it points to is properly initialized and writable. Additionally, the caller must ensure
/// that the struct's fields are set according to the expected format, as invalid values may lead
/// to undefined behavior. The caller is also responsible for ensuring that the dialog is used in
/// a compatible environment, as certain fields may have specific requirements or may not be
/// supported in all contexts. Finally, the caller must handle the dialog's behavior correctly,
/// as improper use may lead to resource leaks or other issues.
///
/// # Returns
/// Returns a nonzero value if the user clicks the OK button and successfully selects a file,
/// or zero if the user cancels the dialog or an error occurs. If the function fails, extended
/// error information can be retrieved by calling `CommDlgExtendedError`.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetSaveFileNameW(ofn: *mut common::OpenFileNameW) -> i32 {
    let policy = common::telemetry::emit_opened("GetSaveFileNameW");
    let result = common::save::save_file_name_w(ofn);
    common::telemetry::emit_result("GetSaveFileNameW", policy, result);
    result
}
