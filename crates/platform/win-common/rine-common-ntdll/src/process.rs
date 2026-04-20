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
    let handle = process_handle.as_raw();
    // NULL (0) or the current-process pseudo-handle (-1) both mean "self".
    if handle == 0 || handle == -1 {
        std::process::exit(exit_status as i32);
    }

    // Terminating other processes is not yet supported.
    tracing::warn!(
        handle = handle,
        "NtTerminateProcess: terminating other processes not implemented"
    );
    NtStatus::NOT_IMPLEMENTED.0
}

pub fn rtl_init_unicode_string() -> u32 {
    tracing::warn!(
        api = "RtlInitUnicodeString",
        dll = "ntdll",
        "RtlInitUnicodeString stub called. Returned success"
    );
    0
}

pub fn rtl_get_version() -> u32 {
    tracing::warn!(
        api = "RtlGetVersion",
        dll = "ntdll",
        "RtlGetVersion stub called. Returned success"
    );
    0
}
