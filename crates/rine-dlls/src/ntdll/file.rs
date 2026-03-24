//! ntdll file I/O: NtWriteFile (minimal — enough for stdout/stderr via kernel32).

use rine_types::errors::NtStatus;
use rine_types::handles::{Handle, handle_to_fd};
use rine_types::structs::IoStatusBlock;

/// NtWriteFile — write data to a file/pipe/device identified by a HANDLE.
///
/// Minimal implementation: translates HANDLE → fd and calls `libc::write`.
/// Ignores ByteOffset, Key, ApcRoutine, ApcContext, and Event (all NULL for
/// simple synchronous console writes).
///
/// # Safety
/// All pointer parameters must be valid for their documented sizes.
/// `buffer` must point to at least `length` readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn NtWriteFile(
    file_handle: isize,  // HANDLE
    _event: isize,       // HANDLE (ignored)
    _apc_routine: usize, // PIO_APC_ROUTINE (ignored)
    _apc_context: usize, // PVOID (ignored)
    io_status_block: *mut IoStatusBlock,
    buffer: *const u8,
    length: u32,
    _byte_offset: *const i64, // PLARGE_INTEGER (ignored)
    _key: *const u32,         // PULONG (ignored)
) -> u32 {
    let handle = Handle::from_raw(file_handle);
    let Some(fd) = handle_to_fd(handle) else {
        return NtStatus::INVALID_HANDLE.0;
    };

    let written = unsafe { libc::write(fd, buffer.cast(), length as usize) };

    if written < 0 {
        if !io_status_block.is_null() {
            unsafe {
                (*io_status_block).status = NtStatus::INVALID_PARAMETER.0;
                (*io_status_block).information = 0;
            }
        }
        return NtStatus::INVALID_PARAMETER.0;
    }

    if !io_status_block.is_null() {
        unsafe {
            (*io_status_block).status = NtStatus::SUCCESS.0;
            (*io_status_block).information = written as usize;
        }
    }
    NtStatus::SUCCESS.0
}
