use crate::common::{fixture, run_rine};

#[test]
fn cmdline_no_extra_args() {
    let output = run_rine(&fixture("cmdline"), &[]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let code = output.status.code().unwrap_or(-1);

    assert_eq!(code, 0);
    assert!(
        stdout.contains("argc:"),
        "should print argc line\nstdout:\n{stdout}"
    );
}

#[test]
fn cmdline_with_args() {
    let output = run_rine(&fixture("cmdline"), &["hello", "world"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let code = output.status.code().unwrap_or(-1);

    assert_eq!(code, 0);
    assert!(
        stdout.contains("hello"),
        "argv should contain 'hello'\nstdout:\n{stdout}"
    );
    assert!(
        stdout.contains("world"),
        "argv should contain 'world'\nstdout:\n{stdout}"
    );
}
