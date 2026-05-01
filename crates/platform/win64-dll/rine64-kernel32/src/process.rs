//! kernel32 process functions: ExitProcess, CreateProcessA/W,
//! GetCommandLineA/W, GetModuleHandleA/W, GetCurrentProcessId,
//! GetExitCodeProcess.

use rine_common_kernel32 as common;
use rine_types::errors::{ERROR_INVALID_HANDLE, ERROR_INVALID_PARAMETER, WinBool};
use rine_types::handles::Handle;
use rine_types::os::{ProcessInformation, StartupInfoA, StartupInfoW};
use rine_types::strings::{LPCSTR, LPCWSTR, read_cstr, read_wstr};

use tracing::warn;

/// Load a DLL into the process's address space.
///
/// # Arguments
/// * `_file_name` - A pointer to a null-terminated ANSI string specifying the name of the DLL to load.
///   If the string does not specify an absolute path, the system searches for the DLL in a specific order of directories.
///   If the function fails to find the DLL, it returns NULL (0).
///
/// # Safety
/// This function is unsafe because it involves raw pointer parameters that must be used correctly by the caller.
/// The caller must ensure that the `library_name` parameter is either null or points to a valid null-terminated
/// ANSI string representing the name of the library to load.
/// Additionally, the caller must handle the returned handle correctly, as it is a raw pointer that may need to
/// be closed with `FreeLibrary` when it is no longer needed. Misuse of the returned handle can lead to resource
/// leaks or other unintended consequences.
///
/// # Returns
/// A handle to the loaded DLL module, or NULL (0) if the function fails to find the DLL.
/// The returned handle can be used in subsequent calls to `GetProcAddress` and `FreeLibrary`.
///
/// # Notes
/// Missing implementation features:
/// - No DLL search-path resolution or module loading is performed.
/// - No module handle/reference-count tracking is maintained.
/// - No integration with loader notifications (`DllMain` process/thread attach)
///   exists.
/// - Failure paths do not set Win32-accurate `GetLastError` values.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn LoadLibraryA(_file_name: LPCSTR) -> u32 {
    tracing::warn!(
        api = "LoadLibraryA",
        dll = "kernel32",
        "LoadLibraryA stub called"
    );

    unsafe {
        let file_name = if _file_name.is_null() {
            return 0;
        } else {
            _file_name.read_string().unwrap_or_default()
        };

        common::process::load_library(&file_name)
    }
}

/// Load a DLL into the process's address space.
///
/// # Arguments
/// * `_file_name` - A pointer to a null-terminated UTF-16LE string specifying the name of the DLL to load.
///   If the string does not specify an absolute path, the system searches for the DLL in a specific order of directories.
///   If the function fails to find the DLL, it returns NULL (0).
///
/// # Safety
/// This function is unsafe because it involves raw pointer parameters that must be used correctly by the caller.
/// The caller must ensure that the `library_name` parameter is either null or points to a valid null-terminated
/// UTF-16LE string representing the name of the library to load.
/// Additionally, the caller must handle the returned handle correctly, as it is a raw pointer that may need to
/// be closed with `FreeLibrary` when it is no longer needed. Misuse of the returned handle can lead to resource
/// leaks or other unintended consequences.
///
/// # Returns
/// A handle to the loaded DLL module, or NULL (0) if the function fails to find the DLL.
/// The returned handle can be used in subsequent calls to `GetProcAddress` and `FreeLibrary`.
///
/// # Notes
/// Missing implementation features:
/// - No DLL search-path resolution or module loading is performed.
/// - No module handle/reference-count tracking is maintained.
/// - No integration with loader notifications (`DllMain` process/thread attach)
///   exists.
/// - Failure paths do not set Win32-accurate `GetLastError` values.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn LoadLibraryW(_file_name: LPCWSTR) -> u32 {
    tracing::warn!(
        api = "LoadLibraryW",
        dll = "kernel32",
        "LoadLibraryW stub called"
    );

    unsafe {
        let file_name = if _file_name.is_null() {
            return 0;
        } else {
            _file_name.read_string().unwrap_or_default()
        };

        common::process::load_library(&file_name)
    }
}

/// Retrieve the address of an exported function or variable from a loaded DLL module.
///
/// # Arguments
/// * `_module` - A handle to the loaded DLL module that contains the function or variable.
///   This handle must have been returned by a previous call to `LoadLibraryA` or `LoadLibraryW`.
/// * `_proc_name` - A pointer to a null-terminated ANSI string specifying the name of the function or variable to retrieve.
///   If the string specifies an ordinal value, it must be in the form of `#123` where `123` is the ordinal number of the
///   function or variable.
///
/// # Safety
/// This function is unsafe because it involves raw pointer parameters that must be used correctly by the caller.
/// The caller must ensure that the `module` parameter is a valid handle returned by a previous call to
/// `LoadLibraryA` or `LoadLibraryW`, and that it has not already been freed.
/// The caller must also ensure that the `proc_name` parameter is a valid null-terminated ANSI string representing the
/// name of the function or variable to retrieve, or a valid ordinal string in the form of `#123`.
///
/// # Returns
/// If the function succeeds, the return value is the address of the specified function or variable.
/// If the function fails, the return value is NULL (0). To get extended error information, call `GetLastError`.
///
/// # Notes
/// Missing implementation features:
/// - No module-handle validation is performed.
/// - No export lookup by ANSI name is implemented.
/// - No export lookup by ordinal is implemented.
/// - Forwarded-export resolution is not implemented.
/// - Failure paths do not set Win32-accurate `GetLastError` values.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetProcAddress() -> u32 {
    tracing::warn!(
        api = "GetProcAddress",
        dll = "kernel32",
        "GetProcAddress stub called"
    );

    unsafe { common::process::get_proc_address() }
}

/// ExitProcess — terminate the current process.
///
/// # Arguments
/// * `exit_code` - The exit code for the process. This value is returned to the operating system and can be used by
///   other processes to determine the reason for termination.
///   By convention, an exit code of 0 typically indicates successful completion,
///   while non-zero values indicate various error conditions or specific exit statuses defined by the application.
///
/// # Safety
/// Does not return.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn ExitProcess(exit_code: u32) -> ! {
    let tid = unsafe { libc::syscall(libc::SYS_gettid) as u32 };
    rine_types::dev_notify!(on_thread_exited(tid, exit_code));
    rine_types::dev_notify!(on_process_exiting(exit_code as i32));
    std::process::exit(exit_code as i32);
}

/// Free a loaded DLL module.
///
/// # Arguments
/// * `_module` - A handle to the loaded DLL module to be freed.
///   This handle must have been returned by a previous call to `LoadLibraryA` or `LoadLibraryW`.
///
/// # Safety
/// This function is unsafe because it involves raw pointer parameters that must be used correctly by the caller.
/// The caller must ensure that the `module` parameter is a valid handle returned by a previous call
/// to `LoadLibraryA` or `LoadLibraryW`, and that it has not already been freed.
/// Additionally, the caller must handle the return value correctly, as it indicates whether the operation succeeded or failed.
/// Misuse of the returned value can lead to incorrect assumptions about the state of the loaded module and
/// potential resource leaks if the module is not properly freed when it is no longer needed.
///
/// # Returns
/// If the function succeeds, the return value is `WinBool::TRUE`.
/// If the function fails, the return value is `WinBool::FALSE`.
/// To get extended error information, call `GetLastError`.
///
/// # Notes
/// Missing implementation features:
/// - No module-handle validation is performed.
/// - No module reference-count decrement/unload is implemented.
/// - No detach notifications (`DllMain` process/thread detach) are issued.
/// - Failure paths do not set Win32-accurate `GetLastError` values.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn FreeLibrary(_module: u32) -> WinBool {
    common::process::free_library(_module)
}

/// Gets a pointer to the ANSI command-line string.
///
/// # Safety
/// The returned pointer is valid for the lifetime of the process.
///
/// # Returns
/// A pointer to a null-terminated ANSI string containing the command line for the current process.
/// The caller should not attempt to modify the contents of the string, as it may be shared and is not owned by the caller.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetCommandLineA() -> *const u8 {
    common::process::cached_cmd_line().ansi.as_ptr().cast()
}

/// Gets a pointer to the wide command-line string.
///
/// # Safety
/// The returned pointer is valid for the lifetime of the process.
///
/// # Returns
/// A pointer to a null-terminated UTF-16LE string containing the command line for the current process.
/// The caller should not attempt to modify the contents of the string, as it may be shared and is not owned by the caller.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetCommandLineW() -> *const u16 {
    common::process::cached_cmd_line().wide.as_ptr()
}

/// Retrieve the base address of a loaded module.
///
/// # Arguments
/// * `module_name` - A pointer to a null-terminated ANSI string specifying the module name.
///   If NULL, the function returns a handle to the file used to create the calling process (the main executable).
///
/// # Safety
/// `module_name` must be null or a valid null-terminated ANSI string.
///
/// # Returns
/// The base address of the specified module, or NULL (0) if the module is not found.
/// If `module_name` is NULL, returns the base address of the main executable.
///
/// * Note
///   When `module_name` is NULL, returns the base address of the main executable.
///   For now we return NULL (0) as a placeholder — the loader will need to provide the real image base
///   once entry-point execution is wired up.
///
/// # Notes
/// Missing implementation features:
/// - `NULL` input does not yet return the actual main image base.
/// - Name-based module lookup is not implemented.
/// - Case-insensitive Windows module-name matching is not implemented.
/// - No module table integration/reference tracking is performed.
/// - Failure paths do not set Win32-accurate `GetLastError` values.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetModuleHandleA(module_name: LPCSTR) -> usize {
    if module_name.is_null() {
        // TODO: return the actual image base once the loader exposes it.
        tracing::debug!("GetModuleHandleA(NULL) — returning 0 (placeholder)");
        return 0;
    }

    unsafe {
        let name = module_name.read_string().unwrap_or_default();
        tracing::warn!("GetModuleHandleA({name}) — returning 0 (stubbed)");

        common::process::get_module_handle(&name)
    }
}

/// Retrieve the base address of a loaded module.
///
/// # Arguments
/// * `module_name` - A pointer to a null-terminated UTF-16LE string specifying the module name.
///   If NULL, the function returns a handle to the file used to create the calling process (the main executable).
///
/// # Safety
/// `module_name` must be null or a valid null-terminated UTF-16LE string.
///
/// # Returns
/// The base address of the specified module, or NULL (0) if the module is not found.
/// If `module_name` is NULL, returns the base address of the main executable.
///
/// * Note
///   When `module_name` is NULL, returns the base address of the main executable.
///   For now we return NULL (0) as a placeholder — the loader will need to provide the real image base
///   once entry-point execution is wired up.
///
/// # Notes
/// Missing implementation features:
/// - `NULL` input does not yet return the actual main image base.
/// - Name-based module lookup is not implemented.
/// - Case-insensitive Windows module-name matching is not implemented.
/// - No module table integration/reference tracking is performed.
/// - Failure paths do not set Win32-accurate `GetLastError` values.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetModuleHandleW(module_name: LPCWSTR) -> usize {
    if module_name.is_null() {
        // TODO: return the actual image base once the loader exposes it.
        tracing::debug!("GetModuleHandleW(NULL) — returning 0 (placeholder)");
        return 0;
    }

    unsafe {
        let name = module_name.read_string().unwrap_or_default();
        tracing::warn!("GetModuleHandleW({name}) — returning 0 (stubbed)");

        common::process::get_module_handle(&name)
    }
}

/// Get the last error code for the current thread.
///
/// # Safety
/// This is only unsafe because the caller may need to ensure thread-safety when calling it from multiple threads,
/// as the error code is typically stored in thread-local storage.
/// The function is marked as unsafe to reflect the typical usage pattern of GetLastError in Windows API, where it is
/// often called after other API functions that may set the error code.
///
/// # Returns
/// The current thread's last-error code.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetLastError() -> u32 {
    common::process::get_last_error()
}

/// Set the last error code for the current thread.
///
/// # Arguments
/// * `error_code` - The error code to store for this thread.
///
/// # Safety
/// This function is unsafe because it is an FFI entry point.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn SetLastError(error_code: u32) {
    common::process::set_last_error(error_code)
}

/// Install a top-level exception filter.
///
/// # Arguments
/// * `_filter` - A pointer to a function that will be called when an unhandled exception occurs in the process.
///   The function should match the `LPTOP_LEVEL_EXCEPTION_FILTER` type, which takes a pointer to an `EXCEPTION_POINTERS`
///   structure and returns a `LONG` value indicating how the exception should be handled.
///
/// # Safety
/// This function is unsafe because it involves raw pointer parameters that must be used correctly by the caller.
///
/// # Returns
/// The SetUnhandledExceptionFilter function returns the address of the previous exception filter established with the function.
/// A NULL return value means that there is no current top-level exception handler.
///
/// # Notes
/// Missing implementation features:
/// - No process-wide top-level exception filter is stored.
/// - The previous filter is not tracked/returned.
/// - No integration with structured exception handling dispatch exists.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn SetUnhandledExceptionFilter(
    _filter: usize, // LPTOP_LEVEL_EXCEPTION_FILTER
) -> usize {
    common::process::set_unhandled_exception_filter(_filter)
}

// ---------------------------------------------------------------------------
// CreateProcessA / CreateProcessW
// ---------------------------------------------------------------------------

/// Create a child process (ANSI).
///
/// # Arguments
/// * `application_name` - A pointer to a null-terminated ANSI string specifying the module to execute.
///   If NULL, the module name must be the first token in `command_line`.
/// * `command_line` - A pointer to a null-terminated ANSI string specifying the command line to execute.
///   The module name must be the first token if `application_name` is NULL.
/// * `_process_attrs` - A pointer to a `SECURITY_ATTRIBUTES` structure that determines whether the returned
///   handle to the new process object can be inherited by child processes. Can be NULL.
/// * `_thread_attrs` - A pointer to a `SECURITY_ATTRIBUTES` structure that determines whether the returned
///   handle to the new thread object can be inherited by child processes. Can be NULL.
/// * `_inherit_handles` - If TRUE, each inheritable handle in the calling process is inherited by the new process.
///   If FALSE, the handles are not inherited.
/// * `_creation_flags` - The flags that control the priority class and the creation of the process.
/// * `environment` - A pointer to the environment block for the new process. Can be NULL.
/// * `_current_directory` - The full path to the current directory for the process. Can be NULL.
/// * `_startup_info` - A pointer to a `STARTUPINFOA` structure.
/// * `process_info` - A pointer to a `PROCESS_INFORMATION` structure that receives identification information about the new process.
///
/// # Safety
/// All pointer parameters must be null or point to valid memory of the expected layout.
///
/// # Returns
/// If the function succeeds, the return value is nonzero (TRUE). If the function fails, the return value is zero (FALSE).
/// To get extended error information, call `GetLastError`.
///
/// # Notes
/// Missing implementation features:
/// - `_process_attrs` and `_thread_attrs` semantics are ignored.
/// - `_inherit_handles` behavior is ignored.
/// - `_creation_flags` behavior is not implemented (for example: suspended
///   start, process group/new console behavior, priority/debug flags).
/// - `_startup_info` semantics are not implemented (for example std handle
///   routing, show-window flags, desktop/title fields).
/// - `_current_directory` is ignored.
/// - The returned thread handle/ID are placeholders and do not model a
///   distinct primary-thread object.
/// - No Win32-accurate `GetLastError` mapping is provided for all failure modes.
#[rine_dlls::partial]
#[allow(non_snake_case, clippy::too_many_arguments)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn CreateProcessA(
    application_name: LPCSTR,              // rcx
    command_line: *mut u8,                 // rdx
    _process_attrs: usize,                 // r8
    _thread_attrs: usize,                  // r9
    _inherit_handles: i32,                 // [rsp+0x28]
    _creation_flags: u32,                  // [rsp+0x30]
    environment: *const u8,                // [rsp+0x38]
    _current_directory: LPCSTR,            // [rsp+0x40]
    _startup_info: *const StartupInfoA,    // [rsp+0x48]
    process_info: *mut ProcessInformation, // [rsp+0x50]
) -> WinBool {
    let app = unsafe { application_name.read_string() }.unwrap_or_default();
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

/// Create a child process (wide).
///
/// # Arguments
/// * `application_name` - A pointer to a null-terminated wide string specifying the module to execute.
///   If NULL, the module name must be the first token in `command_line`.
/// * `command_line` - A pointer to a null-terminated wide string specifying the command line to execute.
///   The module name must be the first token if `application_name` is NULL.
/// * `_process_attrs` - A pointer to a `SECURITY_ATTRIBUTES` structure that determines whether the returned
///   handle to the new process object can be inherited by child processes. Can be NULL.
/// * `_thread_attrs` - A pointer to a `SECURITY_ATTRIBUTES` structure that determines whether the returned
///   handle to the new thread object can be inherited by child processes. Can be NULL.
/// * `_inherit_handles` - If TRUE, each inheritable handle in the calling process is inherited by the new process.
///   If FALSE, the handles are not inherited.
/// * `_creation_flags` - The flags that control the priority class and the creation of the process.
/// * `environment` - A pointer to the environment block for the new process. Can be NULL.
/// * `_current_directory` - The full path to the current directory for the process. Can be NULL.
/// * `_startup_info` - A pointer to a `STARTUPINFOW` structure.
/// * `process_info` - A pointer to a `PROCESS_INFORMATION` structure that receives identification information about the new process.
///
/// # Safety
/// All pointer parameters must be null or point to valid memory of the expected layout.
///
/// # Returns
/// If the function succeeds, the return value is nonzero (TRUE). If the function fails, the return value is zero (FALSE).
/// To get extended error information, call `GetLastError`.
///
/// # Notes
/// Missing implementation features:
/// - `_process_attrs` and `_thread_attrs` semantics are ignored.
/// - `_inherit_handles` behavior is ignored.
/// - `_creation_flags` behavior is not implemented (for example: suspended
///   start, process group/new console behavior, priority/debug flags).
/// - `_startup_info` semantics are not implemented (for example std handle
///   routing, show-window flags, desktop/title fields).
/// - `_current_directory` is ignored.
/// - The returned thread handle/ID are placeholders and do not model a
///   distinct primary-thread object.
/// - No Win32-accurate `GetLastError` mapping is provided for all failure modes.
#[rine_dlls::partial]
#[allow(non_snake_case, clippy::too_many_arguments)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn CreateProcessW(
    application_name: LPCSTR,              // rcx
    command_line: *mut u16,                // rdx
    _process_attrs: usize,                 // r8
    _thread_attrs: usize,                  // r9
    _inherit_handles: i32,                 // [rsp+0x28]
    _creation_flags: u32,                  // [rsp+0x30]
    environment: *const u16,               // [rsp+0x38]
    _current_directory: LPCSTR,            // [rsp+0x40]
    _startup_info: *const StartupInfoW,    // [rsp+0x48]
    process_info: *mut ProcessInformation, // [rsp+0x50]
) -> WinBool {
    let app = unsafe { application_name.read_string() }.unwrap_or_default();
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

/// Gets the process ID of the calling process.
///
/// # Safety
/// This function is unsafe because it is a raw FFI function that can be called from C code.
/// However, there are no specific safety concerns with the current implementation, as it simply
/// returns the process ID using Rust's standard library.
/// The function is marked as unsafe to reflect the typical usage pattern of Windows API functions,
/// which are often unsafe due to their FFI nature and potential for misuse by callers.
///
/// # Returns
/// The process ID of the calling process.
/// This value is a non-negative integer that uniquely identifies the process within the system.
/// The process ID can be used in various API calls that require a process identifier, such as
/// `OpenProcess` or `WaitForSingleObject`.
///
/// # Note
/// Process IDs can be reused by the system after a process terminates, so they should not
/// be assumed to be unique over time.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetCurrentProcessId() -> u32 {
    common::process::get_current_process_id()
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
///
/// # Notes
/// Missing implementation features:
/// - The pseudo-handle is not currently mapped to a concrete process entry in
///   the internal handle table.
/// - APIs expecting a queryable process handle may still reject this pseudo-
///   handle instead of treating it as `GetCurrentProcess()`.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetCurrentProcess() -> Handle {
    common::process::get_current_process()
}

/// Gets the exit code of a process handle.
///
/// # Arguments
/// * `process` - A handle to the process.
///   This handle must have the `PROCESS_QUERY_INFORMATION` or `PROCESS_QUERY_LIMITED_INFORMATION` access right.
/// * `exit_code` - A pointer to a variable that receives the process's exit code.
///   If the function succeeds, the exit code is stored in the variable pointed to by `exit_code`.
///   If the function fails, the contents of the variable pointed to by `exit_code` are undefined.
///   A process that is still active returns the `STILL_ACTIVE` (259) exit code.
///
/// # Safety
/// The caller must ensure that the `process` handle is valid and has the appropriate access rights to query
/// information about the process.
/// The caller must also ensure that the `exit_code` pointer is valid and points to a writable memory location.
///
/// # Returns
/// If the function succeeds, the return value is `WinBool::TRUE`.
/// If the function fails, the return value is `WinBool::FALSE`.
///
/// # Notes
/// We do not currently handle the error case where the handle does not have the
/// PROCESS_QUERY_INFORMATION or PROCESS_QUERY_LIMITED_INFORMATION access right, and instead just
/// return `WinBool::FALSE` with ERROR_INVALID_HANDLE.
///
/// We also do not currently distinguish all invalid-handle sub-cases with
/// finer-grained Win32 error codes.
///
/// Additional missing features:
/// - No explicit access-right checks are enforced against per-handle granted
///   permissions.
/// - Pseudo-handle semantics (`GetCurrentProcess`) are not normalized here.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetExitCodeProcess(process: Handle, exit_code: *mut u32) -> WinBool {
    if exit_code.is_null() {
        common::process::set_last_error(ERROR_INVALID_PARAMETER);
        return WinBool::FALSE;
    }

    if let Some(code) = common::process::get_exit_code_process(process) {
        unsafe { *exit_code = code };
        return WinBool::TRUE;
    };

    common::process::set_last_error(ERROR_INVALID_HANDLE);
    warn!(handle = ?process.as_raw(), "GetExitCodeProcess: invalid handle");
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
        assert_eq!(h, Handle::from_raw(-1));
    }

    // ── GetExitCodeProcess with null pointer ─────────────────────

    #[test]
    fn exit_code_null_ptr_returns_false() {
        let result = unsafe { GetExitCodeProcess(Handle::from_raw(0x9999), std::ptr::null_mut()) };
        assert_eq!(result, WinBool::FALSE);
    }

    #[test]
    fn exit_code_invalid_handle_returns_false() {
        let mut code: u32 = 0;
        let result = unsafe { GetExitCodeProcess(Handle::from_raw(0x9999), &mut code) };
        assert_eq!(result, WinBool::FALSE);
    }
}
