use rine_common_kernel32 as common;
use rine_types::errors::WinBool;
use rine_types::os::{ProcessInformation, StartupInfoA, StartupInfoW};
use rine_types::strings::{read_cstr, read_wstr};
use rine_types::threading;

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
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn LoadLibraryA(_file_name: *const u8) -> u32 {
    tracing::warn!(api = "LoadLibraryA", dll = "kernel32", "win32 stub called");
    0
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
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn LoadLibraryW(_file_name: *const u16) -> u32 {
    tracing::warn!(api = "LoadLibraryW", dll = "kernel32", "win32 stub called");
    0
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn GetProcAddress() -> u32 {
    tracing::warn!(
        api = "GetProcAddress",
        dll = "kernel32",
        "win32 stub called"
    );
    0
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn FreeLibrary() -> u32 {
    tracing::warn!(api = "FreeLibrary", dll = "kernel32", "win32 stub called");
    0
}

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
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn ExitProcess(exit_code: u32) -> ! {
    let tid = unsafe { libc::syscall(libc::SYS_gettid) as u32 };
    rine_types::dev_notify!(on_thread_exited(tid, exit_code));
    rine_types::dev_notify!(on_process_exiting(exit_code as i32));
    std::process::exit(exit_code as i32);
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
/// Stub: returns NULL (no previous handler). Exception handling is not
/// yet implemented.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn SetUnhandledExceptionFilter(
    _filter: usize, // LPTOP_LEVEL_EXCEPTION_FILTER
) -> usize {
    common::process::set_unhandled_exception_filter(_filter)
}

/// Gets a pointer to the ANSI command-line string.
///
/// # Safety
/// The returned pointer is valid for the lifetime of the process.
///
/// # Returns
/// A pointer to a null-terminated ANSI string containing the command line for the current process.
/// The caller should not attempt to modify the contents of the string, as it may be shared and is not owned by the caller.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetCommandLineA() -> *const u8 {
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
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetCommandLineW() -> *const u16 {
    common::process::cached_cmd_line().wide.as_ptr()
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetCurrentProcessId() -> u32 {
    std::process::id()
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
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetModuleHandleW(module_name: *const u16) -> usize {
    if module_name.is_null() {
        // TODO: return the actual image base once the loader exposes it.
        tracing::debug!("GetModuleHandleW(NULL) — returning 0 (placeholder)");
        return 0;
    }

    unsafe {
        let name = read_wstr(module_name).unwrap_or_default();
        tracing::warn!("GetModuleHandleW({name}) — returning 0 (stubbed)");

        common::process::get_module_handle(&name)
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

/// Get the last error code for the current thread.
///
/// # Safety
/// This is only unsafe because the caller may need to ensure thread-safety when calling it from multiple threads,
/// as the error code is typically stored in thread-local storage.
/// However, in our current implementation, we always return 0 (ERROR_SUCCESS), so there are no actual safety concerns
/// with the current behavior.
/// The function is marked as unsafe to reflect the typical usage pattern of GetLastError in Windows API, where it is
/// often called after other API functions that may set the error code.
///
/// # Returns
/// Currently always returns 0 (ERROR_SUCCESS).
///
/// # Note
/// Stub implementation which always indicates success.
#[allow(non_snake_case)]
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
