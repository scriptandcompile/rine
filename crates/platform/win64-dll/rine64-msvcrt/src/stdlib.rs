//! msvcrt stdlib functions: exit, _cexit.

use rine_common_msvcrt as common;

/// Terminate the process with the given exit code.
///
/// # Arguments
/// * `code` - The exit code to terminate the process with.
///
/// # Safety
/// Does not return.
pub unsafe extern "win64" fn exit(code: core::ffi::c_int) {
    unsafe { common::exit(code) };
}

/// Perform CRT cleanup without terminating the process.
///
/// # Safety
/// Calls into platform APIs and flushes C stdio buffers, but does not take any pointer arguments.
///
/// # Notes
/// A full implementation would also run atexit handlers and C++ destructors registered with the CRT.
/// Currently, this function only flushes C stdio buffers.
pub unsafe extern "win64" fn _cexit() {
    unsafe { common::_cexit() };
}
