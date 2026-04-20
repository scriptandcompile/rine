//! msvcrt stdlib functions: exit, _cexit.

/// Terminate the process with the given exit code.
///
/// # Arguments
/// * `code` - The exit code to terminate the process with.
///
/// # Safety
/// Does not return.
pub unsafe fn exit(code: i32) {
    tracing::trace!(code, "msvcrt::exit");
    let tid = unsafe { libc::syscall(libc::SYS_gettid) as u32 };
    rine_types::dev_notify!(on_thread_exited(tid, code as u32));
    rine_types::dev_notify!(on_process_exiting(code));
    std::process::exit(code);
}

/// Perform CRT cleanup without terminating the process.
///
/// # Safety
/// Calls into platform APIs and flushes C stdio buffers, but does not take any pointer arguments.
///
/// # Notes
/// A full implementation would also run atexit handlers and C++ destructors registered with the CRT.
/// Currently, this function only flushes C stdio buffers.
pub unsafe fn _cexit() {
    tracing::trace!("msvcrt::_cexit");
    // Flush all open C stdio streams.
    unsafe { libc::fflush(core::ptr::null_mut()) };
}
