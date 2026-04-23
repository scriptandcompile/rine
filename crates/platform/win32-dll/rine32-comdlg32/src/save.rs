use rine_common_comdlg32 as common;

/// GetSaveFileNameA — save dialog for ANSI paths.
///
/// # Safety
/// `ofn` must be null or a valid pointer to an `OPENFILENAMEA` whose string
/// fields are null or point to valid NUL-terminated ANSI strings.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetSaveFileNameA(ofn: *mut common::OpenFileNameA) -> i32 {
    let policy = common::telemetry::emit_opened("GetSaveFileNameA");
    let result = common::save::save_file_name_a(ofn);
    common::telemetry::emit_result("GetSaveFileNameA", policy, result);
    result
}

/// GetSaveFileNameW — save dialog for UTF-16 paths.
///
/// # Safety
/// `ofn` must be null or a valid pointer to an `OPENFILENAMEW` whose string
/// fields are null or point to valid NUL-terminated UTF-16 strings.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetSaveFileNameW(ofn: *mut common::OpenFileNameW) -> i32 {
    let policy = common::telemetry::emit_opened("GetSaveFileNameW");
    let result = common::save::save_file_name_w(ofn);
    common::telemetry::emit_result("GetSaveFileNameW", policy, result);
    result
}
