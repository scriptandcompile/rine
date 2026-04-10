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
#[allow(non_snake_case, clippy::too_many_arguments)]
#[unsafe(no_mangle)]
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
        unsafe { Some(common::process::parse_env_block(environment)) }
    };

    unsafe { common::process::create_process(&exe, &args, env, process_info) }
}

/// CreateProcessW — create a child process (wide).
///
/// # Safety
/// All pointer parameters must be null or point to valid memory of the
/// expected layout.
#[allow(non_snake_case, clippy::too_many_arguments)]
#[unsafe(no_mangle)]
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
        unsafe { Some(common::process::parse_env_block_wide(environment)) }
    };

    unsafe { common::process::create_process(&exe, &args, env, process_info) }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
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
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn SetUnhandledExceptionFilter(
    _filter: usize, // LPTOP_LEVEL_EXCEPTION_FILTER
) -> usize {
    0 // NULL — no previous handler
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetCommandLineA() -> *const u8 {
    common::process::cached_cmd_line().ansi.as_ptr().cast()
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetCommandLineW() -> *const u16 {
    common::process::cached_cmd_line().wide.as_ptr()
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetCurrentProcessId() -> u32 {
    std::process::id()
}

/// GetModuleHandleA — retrieve the base address of a loaded module.
///
/// When `module_name` is NULL, returns the base address of the main
/// executable. For now we return NULL (0) as a placeholder — the loader
/// will need to provide the real image base once entry-point execution
/// is wired up.
///
/// # Arguments
/// * `module_name` - A pointer to a null-terminated ANSI string specifying the module name.
///   If NULL, the function returns a handle to the file used to create the calling process (the main executable).
///
/// # Safety
/// `module_name` must be null or a valid null-terminated ANSI string.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetModuleHandleA(module_name: *const u8) -> usize {
    if module_name.is_null() {
        // TODO: return the actual image base once the loader exposes it.
        tracing::debug!("GetModuleHandleA(NULL) — returning 0 (placeholder)");
        return 0;
    }

    unsafe {
        let name = read_cstr(module_name).unwrap_or_default();
        tracing::warn!("GetModuleHandleA({name}) — returning 0 (not implemented)");

        common::process::get_module_handle_a(&name)
    }
}

/// Gets the pseudo-handle for the current process, which is currently always -1 in our implementation.
///
/// # Safety
/// This function is unsafe because it returns a raw handle value that must be used correctly by the caller.
/// The caller must ensure that the returned handle is not misused, as it is a sentinel value representing
/// the current process and not a real handle that can be manipulated or closed.
///
/// # Returns
/// The pseudo-handle for the current process, which is currently always -1.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetCurrentProcess() -> isize {
    common::process::get_current_process()
}

/// Get the last error code for the current thread. Currently always returns 0 (ERROR_SUCCESS).
#[allow(non_snake_case, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetLastError() -> u32 {
    common::process::get_last_error()
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
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
