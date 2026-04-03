use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn workspace_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(|p| p.parent())
        .expect("could not find workspace root")
        .to_path_buf()
}

pub fn fixture(name: &str) -> PathBuf {
    let exe = workspace_root()
        .join("tests/fixtures/bin")
        .join(format!("{name}.exe"));
    assert!(
        exe.exists(),
        "fixture not found: {}\nRun `tests/build_fixtures.sh` to build.",
        exe.display()
    );
    exe
}

pub fn run_rine(fixture_path: &Path, extra_args: &[&str]) -> Output {
    let rine = env!("CARGO_BIN_EXE_rine");
    let mut cmd = Command::new(rine);
    cmd.arg(fixture_path);
    for arg in extra_args {
        cmd.arg(arg);
    }
    cmd.env("RUST_LOG", "off");
    cmd.output().expect("failed to execute rine")
}

pub fn run_rine_with_env(
    fixture_path: &Path,
    extra_args: &[&str],
    envs: &[(&str, &str)],
) -> Output {
    let rine = env!("CARGO_BIN_EXE_rine");
    let mut cmd = Command::new(rine);
    cmd.arg(fixture_path);
    for arg in extra_args {
        cmd.arg(arg);
    }
    cmd.env("RUST_LOG", "off");
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("failed to execute rine")
}

pub fn unique_temp_dir(prefix: &str) -> PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("{prefix}-{ts}"));
    std::fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

pub fn write_minimal_pe32(path: &Path) {
    let mut bytes = vec![0u8; 0x90];
    bytes[0..2].copy_from_slice(b"MZ");
    bytes[0x3c..0x40].copy_from_slice(&(0x80u32).to_le_bytes());
    bytes[0x80..0x84].copy_from_slice(b"PE\0\0");
    bytes[0x84..0x86].copy_from_slice(&(0x014cu16).to_le_bytes());
    std::fs::write(path, bytes).expect("failed to write minimal pe32 fixture");
}

pub fn assert_run(name: &str, expected_code: i32, expected_stdout: &str) {
    let output = run_rine(&fixture(name), &[]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let code = output.status.code().unwrap_or(-1);

    assert_eq!(
        code, expected_code,
        "\n--- {name} ---\nexpected exit code {expected_code}, got {code}\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(
        stdout.trim(),
        expected_stdout.trim(),
        "\n--- {name} ---\nstdout mismatch\nstderr:\n{stderr}"
    );
}

pub fn assert_exit_code(name: &str, expected_code: i32) {
    let output = run_rine(&fixture(name), &[]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let code = output.status.code().unwrap_or(-1);

    assert_eq!(
        code, expected_code,
        "\n--- {name} ---\nexpected exit code {expected_code}, got {code}\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
}

#[cfg(unix)]
pub fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let metadata = std::fs::metadata(path).expect("failed to stat helper script");
    let mut perms = metadata.permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("failed to chmod helper script");
}
