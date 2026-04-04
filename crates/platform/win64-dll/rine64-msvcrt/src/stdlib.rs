//! msvcrt stdlib functions: exit, _cexit.

/// exit — terminate the process with the given exit code.
///
/// # Safety
/// Does not return.
pub unsafe extern "win64" fn exit(code: core::ffi::c_int) {
    tracing::debug!(code, "msvcrt::exit");
    let tid = unsafe { libc::syscall(libc::SYS_gettid) as u32 };
    rine_types::dev_notify!(on_thread_exited(tid, code as u32));
    rine_types::dev_notify!(on_process_exiting(code));
    std::process::exit(code);
}

/// _cexit — perform CRT cleanup without terminating the process.
///
/// Flushes all C stdio buffers. A full implementation would also run
/// atexit handlers and C++ destructors registered with the CRT.
///
/// # Safety
/// No pointer arguments.
pub unsafe extern "win64" fn _cexit() {
    tracing::trace!("msvcrt::_cexit");
    // Flush all open C stdio streams.
    unsafe { libc::fflush(core::ptr::null_mut()) };
}
