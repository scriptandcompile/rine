use std::process::Command;

use rine_common_kernel32::process::{
    run_with_unhandled_exception_filter, set_unhandled_exception_filter,
};

const CHILD_ENV: &str = "RINE_FAULT_FILTER_CHILD";
const FILTER_MARKER: &[u8] = b"RINE_UNHANDLED_FILTER_CALLED\n";

unsafe extern "system" fn test_top_level_filter(_exception_ptr: usize) -> i32 {
    unsafe {
        libc::write(
            libc::STDERR_FILENO,
            FILTER_MARKER.as_ptr().cast(),
            FILTER_MARKER.len(),
        );
    }
    1
}

#[test]
fn unhandled_exception_filter_invoked_on_fatal_signal_subprocess() {
    if std::env::var_os(CHILD_ENV).is_some() {
        let filter = test_top_level_filter as *const () as usize;
        let _ = set_unhandled_exception_filter(filter);

        run_with_unhandled_exception_filter(|| unsafe {
            libc::raise(libc::SIGSEGV);
        });

        panic!("child process should terminate via SIGSEGV handler before returning");
    }

    let current_test = "unhandled_exception_filter_invoked_on_fatal_signal_subprocess";
    let exe = std::env::current_exe().expect("failed to resolve current test binary path");
    let output = Command::new(exe)
        .arg("--exact")
        .arg(current_test)
        .arg("--nocapture")
        .env(CHILD_ENV, "1")
        .output()
        .expect("failed to launch subprocess for fatal signal test");

    assert!(!output.status.success(), "child unexpectedly succeeded");
    assert_eq!(
        output.status.code(),
        Some(128 + libc::SIGSEGV),
        "child exit code should represent SIGSEGV"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("RINE_UNHANDLED_FILTER_CALLED"),
        "expected filter marker in stderr, got:\n{stderr}"
    );
}
