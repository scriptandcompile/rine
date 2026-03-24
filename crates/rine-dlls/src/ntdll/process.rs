//! ntdll process management: NtTerminateProcess.

use rine_types::errors::NtStatus;

/// NtTerminateProcess — terminate the current (or specified) process.
///
/// `process_handle`: if NULL / -1 (current process pseudo-handle), exits
///                   the current process with `exit_status`.
///
/// # Safety
/// Calling this function terminates the process; it does not return.
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn NtTerminateProcess(
    process_handle: isize, // HANDLE — NULL or current-process pseudo-handle
    exit_status: u32,      // NTSTATUS
) -> u32 {
    // NULL (0) or the current-process pseudo-handle (-1) both mean "self".
    if process_handle == 0 || process_handle == -1 {
        std::process::exit(exit_status as i32);
    }

    // Terminating other processes is not yet supported.
    tracing::warn!(
        handle = process_handle,
        "NtTerminateProcess: terminating other processes not implemented"
    );
    NtStatus::NOT_IMPLEMENTED.0
}
