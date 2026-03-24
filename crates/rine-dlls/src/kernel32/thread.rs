//! kernel32 thread functions: TLS, Sleep (minimal Phase 1 stubs).

/// TlsGetValue — returns the value for a TLS index.
///
/// Stub: always returns NULL (no TLS support yet).
pub unsafe extern "win64" fn TlsGetValue(_tls_index: u32) -> usize {
    // SetLastError(ERROR_SUCCESS) would be correct here, but we don't
    // track per-thread last-error yet.
    0 // NULL
}

/// Sleep — suspend execution for the given number of milliseconds.
pub unsafe extern "win64" fn Sleep(milliseconds: u32) {
    let dur = std::time::Duration::from_millis(milliseconds as u64);
    std::thread::sleep(dur);
}
