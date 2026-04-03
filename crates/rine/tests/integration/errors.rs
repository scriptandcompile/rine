use std::process::Command;

#[test]
fn nonexistent_exe() {
    let rine = env!("CARGO_BIN_EXE_rine");
    let output = Command::new(rine)
        .arg("/nonexistent/path.exe")
        .env("RUST_LOG", "off")
        .output()
        .expect("failed to execute rine");

    assert!(
        !output.status.success(),
        "loading a nonexistent file should fail"
    );
}

#[test]
fn not_a_pe_file() {
    let rine = env!("CARGO_BIN_EXE_rine");
    let output = Command::new(rine)
        .arg(rine)
        .env("RUST_LOG", "off")
        .output()
        .expect("failed to execute rine");

    assert!(
        !output.status.success(),
        "loading an ELF binary should fail"
    );
}
