use rine_common_kernel32 as common;
use rine_types::errors::WinBool;
use rine_types::os::{ProcessInformation, StartupInfoA, StartupInfoW};
use rine_types::strings::{read_cstr, read_wstr};
use rine_types::threading;

use tracing::warn;

/// CreateProcessA — create a child process (ANSI).
///
/// # Safety
/// All pointer parameters must be null or point to valid memory of the
/// expected layout.
#[allow(non_snake_case, clippy::missing_safety_doc, clippy::too_many_arguments)]
pub unsafe extern "stdcall" fn CreateProcessA(
    application_name: *const u8,           // rcx
    command_line: *mut u8,                 // rdx
    _process_attrs: usize,                 // r8
    _thread_attrs: usize,                  // r9
    _inherit_handles: i32,                 // [rsp+0x28]
    _creation_flags: u32,                  // [rsp+0x30]
    environment: *const u8,                // [rsp+0x38]
    _current_directory: *const u8,         // [rsp+0x40]
    _startup_info: *const StartupInfoA,    // [rsp+0x48]
    process_info: *mut ProcessInformation, // [rsp+0x50]
) -> WinBool {
    let app = unsafe { read_cstr(application_name) }.unwrap_or_default();
    let cmd = unsafe { read_cstr(command_line.cast_const()) }.unwrap_or_default();

    let (exe, args) = if !app.is_empty() {
        (app, common::process::split_cmd_line(&cmd))
    } else {
        let tokens = common::process::split_cmd_line(&cmd);
        if tokens.is_empty() {
            warn!("CreateProcessA: no executable specified");
            return WinBool::FALSE;
        }
        (tokens[0].clone(), tokens[1..].to_vec())
    };

    let env = if environment.is_null() {
        None
    } else {
        Some(common::process::parse_env_block(environment))
    };

    common::process::do_create_process(&exe, &args, env, process_info)
}

/// CreateProcessW — create a child process (wide).
///
/// # Safety
/// All pointer parameters must be null or point to valid memory of the
/// expected layout.
#[allow(non_snake_case, clippy::missing_safety_doc, clippy::too_many_arguments)]
pub unsafe extern "stdcall" fn CreateProcessW(
    application_name: *const u16,          // rcx
    command_line: *mut u16,                // rdx
    _process_attrs: usize,                 // r8
    _thread_attrs: usize,                  // r9
    _inherit_handles: i32,                 // [rsp+0x28]
    _creation_flags: u32,                  // [rsp+0x30]
    environment: *const u16,               // [rsp+0x38]
    _current_directory: *const u16,        // [rsp+0x40]
    _startup_info: *const StartupInfoW,    // [rsp+0x48]
    process_info: *mut ProcessInformation, // [rsp+0x50]
) -> WinBool {
    let app = unsafe { read_wstr(application_name) }.unwrap_or_default();
    let cmd = unsafe { read_wstr(command_line.cast_const()) }.unwrap_or_default();

    let (exe, args) = if !app.is_empty() {
        (app, common::process::split_cmd_line(&cmd))
    } else {
        let tokens = common::process::split_cmd_line(&cmd);
        if tokens.is_empty() {
            warn!("CreateProcessW: no executable specified");
            return WinBool::FALSE;
        }
        (tokens[0].clone(), tokens[1..].to_vec())
    };

    let env = if environment.is_null() {
        None
    } else {
        Some(common::process::parse_env_block_wide(environment))
    };

    common::process::do_create_process(&exe, &args, env, process_info)
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn ExitProcess(exit_code: u32) -> ! {
    let tid = unsafe { libc::syscall(libc::SYS_gettid) as u32 };
    rine_types::dev_notify!(on_thread_exited(tid, exit_code));
    rine_types::dev_notify!(on_process_exiting(exit_code as i32));
    std::process::exit(exit_code as i32);
}

/// SetUnhandledExceptionFilter — install a top-level exception filter.
///
/// Stub: returns NULL (no previous handler). Exception handling is not
/// yet implemented.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn SetUnhandledExceptionFilter(
    _filter: usize, // LPTOP_LEVEL_EXCEPTION_FILTER
) -> usize {
    0 // NULL — no previous handler
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn GetCommandLineA() -> *const u8 {
    common::process::cached_cmd_line().ansi.as_ptr().cast()
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn GetCommandLineW() -> *const u16 {
    common::process::cached_cmd_line().wide.as_ptr()
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn GetCurrentProcessId() -> u32 {
    std::process::id()
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn GetCurrentProcess() -> isize {
    -1
}

/// GetLastError — return the last-error code for the calling thread.
///
/// Stub: always returns 0 (ERROR_SUCCESS). A real per-thread last-error
/// store will be added with the threading subsystem.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn GetLastError() -> u32 {
    0
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn GetExitCodeProcess(
    _process_handle: isize,
    exit_code_out: *mut u32,
) -> WinBool {
    if exit_code_out.is_null() {
        return WinBool::FALSE;
    }
    unsafe { *exit_code_out = threading::STILL_ACTIVE };
    WinBool::TRUE
}
