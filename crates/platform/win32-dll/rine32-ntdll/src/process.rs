use rine_common_ntdll::process as common;
use rine_types::handles::HANDLE;

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
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn NtTerminateProcess(
    process_handle: HANDLE,
    exit_status: u32, // NTSTATUS
) -> u32 {
    common::nt_terminate_process(process_handle, exit_status)
}
