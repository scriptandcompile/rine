use rine_types::errors::NtStatus;
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
pub fn nt_terminate_process(
    process_handle: Handle,
    exit_status: u32, // NTSTATUS
) -> u32 {
    // NULL (0) or the current-process pseudo-handle (-1) both mean "self".
    if process_handle.is_null() || process_handle.is_invalid() {
        std::process::exit(exit_status as i32);
    }

    // Terminating other processes is not yet supported.
    tracing::warn!(
        handle = process_handle.as_raw(),
        "NtTerminateProcess: terminating other processes not implemented"
    );
    NtStatus::NOT_IMPLEMENTED.0
}
