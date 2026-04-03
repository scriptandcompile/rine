use crate::common::{fixture, run_rine_with_env};

#[test]
fn test_dialog_basic_emulated_mode() {
    let output = run_rine_with_env(
        &fixture("dialog_basic"),
        &[],
        &[("RINE_DIALOG_THEME", "windows")],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let code = output.status.code().unwrap_or(-1);

    assert_eq!(
        code, 0,
        "\n--- dialog_basic ---\nexpected exit code 0, got {code}\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(
        stdout.trim(),
        "PASS: GetOpenFileNameA failed as expected\n\
         PASS: CommDlgExtendedError A is CDERR_DIALOGFAILURE\n\
         PASS: GetSaveFileNameW failed as expected\n\
         PASS: CommDlgExtendedError W is CDERR_DIALOGFAILURE\n\
         All dialog basic tests passed",
        "\n--- dialog_basic ---\nstdout mismatch\nstderr:\n{stderr}"
    );
}

#[test]
fn test_dialog_small_buffer_error() {
    let output = run_rine_with_env(
        &fixture("dialog_small_buffer"),
        &[],
        &[
            ("RINE_DIALOG_THEME", "native"),
            (
                "RINE_DIALOG_TEST_PATH",
                "C:/rine/tests/this_path_is_too_long_for_tiny_buffer.exe",
            ),
        ],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let code = output.status.code().unwrap_or(-1);

    assert_eq!(
        code, 0,
        "\n--- dialog_small_buffer ---\nexpected exit code 0, got {code}\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(
        stdout.trim(),
        "PASS: GetOpenFileNameA failed for tiny buffer\n\
         PASS: CommDlgExtendedError is FNERR_BUFFERTOOSMALL\n\
         All dialog small-buffer tests passed",
        "\n--- dialog_small_buffer ---\nstdout mismatch\nstderr:\n{stderr}"
    );
}
