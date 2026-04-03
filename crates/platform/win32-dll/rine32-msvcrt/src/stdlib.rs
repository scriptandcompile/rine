//! msvcrt stdlib functions: exit, _cexit.

/// exit — terminate the process with the given exit code.
///
/// # Safety
/// Does not return.
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn exit(code: i32) {
    tracing::debug!(code, "msvcrt::exit");
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
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn _cexit() {
    tracing::trace!("msvcrt::_cexit");
    // Flush all open C stdio streams.
    unsafe { libc::fflush(core::ptr::null_mut()) };
}
