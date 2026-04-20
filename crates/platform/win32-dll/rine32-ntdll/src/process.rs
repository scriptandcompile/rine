use rine_common_ntdll::process as common;
use rine_types::handles::Handle;

/// Terminate the current (or specified) process.
///
/// # Arguments
/// * `process_handle`: if NULL or -1 (current process pseudo-handle), exits
///   the current process with `exit_status`.
/// * `exit_status`: the exit status to use if terminating the current process.
///
/// # Safety
/// Calling this function terminates the process it does not return currently
/// because terminating other processes is not yet implemented.
///
/// # Returns
/// If `process_handle` is not NULL or -1, returns `STATUS_NOT_IMPLEMENTED`.
///
/// # Note
/// This is a partial implementation that only supports terminating the current process.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn NtTerminateProcess(
    process_handle: isize, // HANDLE — NULL or current-process pseudo-handle
    exit_status: u32,      // NTSTATUS
) -> u32 {
    let handle = Handle::from_raw(process_handle);
    common::nt_terminate_process(handle, exit_status)
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn RtlInitUnicodeString() -> u32 {
    tracing::warn!(
        api = "RtlInitUnicodeString",
        dll = "ntdll",
        "win32 stub called"
    );
    0
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn RtlGetVersion() -> u32 {
    tracing::warn!(api = "RtlGetVersion", dll = "ntdll", "win32 stub called");
    0
}
