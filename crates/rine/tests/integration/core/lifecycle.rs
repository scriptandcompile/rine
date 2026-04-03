use crate::common::{fixture, run_rine};

#[test]
fn exit_process() {
    let output = run_rine(&fixture("exit_process"), &[]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let code = output.status.code().unwrap_or(-1);

    assert_eq!(code, 7, "ExitProcess(7) should produce exit code 7");
    assert!(
        stdout.contains("before exit"),
        "should see output before ExitProcess"
    );
    assert!(
        !stdout.contains("FAIL: after ExitProcess"),
        "should NOT see output after ExitProcess"
    );
}
