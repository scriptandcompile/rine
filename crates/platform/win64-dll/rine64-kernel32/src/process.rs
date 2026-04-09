//! kernel32 process functions: ExitProcess, CreateProcessA/W,
//! GetCommandLineA/W, GetModuleHandleA/W, GetCurrentProcessId,
//! GetExitCodeProcess.

use std::sync::atomic::Ordering;

use rine_common_kernel32 as common;
use rine_types::errors::WinBool;
use rine_types::handles::{Handle, handle_table};
use rine_types::os::{ProcessInformation, StartupInfoA, StartupInfoW};
use rine_types::strings::{read_cstr, read_wstr};

use tracing::warn;

/// ExitProcess — terminate the current process.
///
/// # Safety
/// Does not return.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn ExitProcess(exit_code: u32) -> ! {
    let tid = unsafe { libc::syscall(libc::SYS_gettid) as u32 };
    rine_types::dev_notify!(on_thread_exited(tid, exit_code));
    rine_types::dev_notify!(on_process_exiting(exit_code as i32));
    std::process::exit(exit_code as i32);
}

/// GetCommandLineA — return a pointer to the ANSI command-line string.
///
/// # Safety
/// The returned pointer is valid for the lifetime of the process.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetCommandLineA() -> *const u8 {
    common::process::cached_cmd_line().ansi.as_ptr().cast()
}

/// GetCommandLineW — return a pointer to the wide command-line string.
///
/// # Safety
/// The returned pointer is valid for the lifetime of the process.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetCommandLineW() -> *const u16 {
    common::process::cached_cmd_line().wide.as_ptr()
}

/// GetModuleHandleA — retrieve the base address of a loaded module.
///
/// When `module_name` is NULL, returns the base address of the main
/// executable. For now we return NULL (0) as a placeholder — the loader
/// will need to provide the real image base once entry-point execution
/// is wired up.
///
/// # Safety
/// `module_name` must be null or a valid null-terminated ANSI string.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetModuleHandleA(module_name: *const u8) -> usize {
    if module_name.is_null() {
        // TODO: return the actual image base once the loader exposes it.
        tracing::debug!("GetModuleHandleA(NULL) — returning 0 (placeholder)");
        return 0;
    }

    let name = unsafe { std::ffi::CStr::from_ptr(module_name.cast()) };
    tracing::warn!(
        ?name,
        "GetModuleHandleA: non-NULL module_name not yet supported"
    );
    0
}

/// GetModuleHandleW — wide variant of `GetModuleHandleA`.
///
/// # Safety
/// `module_name` must be null or a valid null-terminated UTF-16LE string.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetModuleHandleW(module_name: *const u16) -> usize {
    if module_name.is_null() {
        tracing::debug!("GetModuleHandleW(NULL) — returning 0 (placeholder)");
        return 0;
    }

    // Decode for logging only.
    let mut len = 0;
    unsafe {
        while *module_name.add(len) != 0 {
            len += 1;
        }
    }
    let wide_slice = unsafe { core::slice::from_raw_parts(module_name, len) };
    let name = String::from_utf16_lossy(wide_slice);
    tracing::warn!(
        name,
        "GetModuleHandleW: non-NULL module_name not yet supported"
    );
    0
}

/// GetLastError — return the last-error code for the calling thread.
///
/// Stub: always returns 0 (ERROR_SUCCESS). A real per-thread last-error
/// store will be added with the threading subsystem.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn GetLastError() -> u32 {
    0
}

/// SetUnhandledExceptionFilter — install a top-level exception filter.
///
/// Stub: returns NULL (no previous handler). Exception handling is not
/// yet implemented.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn SetUnhandledExceptionFilter(
    _filter: usize, // LPTOP_LEVEL_EXCEPTION_FILTER
) -> usize {
    0 // NULL — no previous handler
}

// ---------------------------------------------------------------------------
// CreateProcessA / CreateProcessW
// ---------------------------------------------------------------------------

/// CreateProcessA — create a child process (ANSI).
///
/// # Safety
/// All pointer parameters must be null or point to valid memory of the
/// expected layout.
#[allow(non_snake_case, clippy::missing_safety_doc, clippy::too_many_arguments)]
pub unsafe extern "win64" fn CreateProcessA(
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
#[allow(non_snake_case, clippy::missing_safety_doc, clippy::too_many_arguments)]
pub unsafe extern "win64" fn CreateProcessW(
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

// ---------------------------------------------------------------------------
// Process info queries
// ---------------------------------------------------------------------------

/// GetCurrentProcessId — return the process ID of the calling process.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn GetCurrentProcessId() -> u32 {
    std::process::id()
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
pub unsafe extern "win64" fn GetCurrentProcess() -> isize {
    common::process::get_current_process()
}

/// GetExitCodeProcess — read the exit code of a process handle.
///
/// Returns `STILL_ACTIVE` (259) if the process has not yet terminated.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn GetExitCodeProcess(process: isize, exit_code: *mut u32) -> WinBool {
    if exit_code.is_null() {
        return WinBool::FALSE;
    }

    let h = Handle::from_raw(process);
    if let Some(rine_types::threading::Waitable::Process(p)) = handle_table().get_waitable(h) {
        unsafe { *exit_code = p.exit_code.load(Ordering::Acquire) };
        return WinBool::TRUE;
    }

    warn!(handle = ?h, "GetExitCodeProcess: invalid handle");
    WinBool::FALSE
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── GetCurrentProcessId / GetCurrentProcess ─────────────────

    #[test]
    fn current_process_id_nonzero() {
        let pid = unsafe { GetCurrentProcessId() };
        assert!(pid > 0);
    }

    #[test]
    fn current_process_pseudo_handle() {
        let h = unsafe { GetCurrentProcess() };
        assert_eq!(h, -1);
    }

    // ── GetExitCodeProcess with null pointer ─────────────────────

    #[test]
    fn exit_code_null_ptr_returns_false() {
        let result = unsafe { GetExitCodeProcess(0x9999, std::ptr::null_mut()) };
        assert_eq!(result, WinBool::FALSE);
    }

    #[test]
    fn exit_code_invalid_handle_returns_false() {
        let mut code: u32 = 0;
        let result = unsafe { GetExitCodeProcess(0x9999, &mut code) };
        assert_eq!(result, WinBool::FALSE);
    }
}
