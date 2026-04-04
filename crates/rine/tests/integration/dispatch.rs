use super::common::{
    fixture_arch, make_executable, run_rine_with_env, try_fixture, unique_temp_dir,
    write_minimal_pe32,
};

#[cfg(unix)]
#[test]
fn x86_binary_dispatches_to_rine32_helper() {
    let dir = unique_temp_dir("rine-dispatch-success");
    let pe = dir.join("tiny_x86.exe");
    let helper = dir.join("rine32");
    let marker = dir.join("helper-arg.txt");

    write_minimal_pe32(&pe);
    std::fs::write(
        &helper,
        format!(
            "#!/usr/bin/env sh\nprintf '%s' \"$1\" > '{}'\nexit 0\n",
            marker.display()
        ),
    )
    .expect("failed to write helper script");
    make_executable(&helper);

    let output = run_rine_with_env(
        &pe,
        &[],
        &[
            ("RUST_LOG", "off"),
            ("RINE_RINE32_HELPER", helper.to_string_lossy().as_ref()),
        ],
    );

    assert!(
        output.status.success(),
        "dispatch should succeed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let seen = std::fs::read_to_string(&marker).expect("helper should write marker file");
    assert_eq!(
        seen,
        pe.display().to_string(),
        "helper should receive exe path as first argument"
    );

    let _ = std::fs::remove_dir_all(dir);
}

#[cfg(unix)]
#[test]
fn x86_binary_without_helper_fails_with_hint() {
    let dir = unique_temp_dir("rine-dispatch-missing");
    let pe = dir.join("tiny_x86_missing.exe");
    write_minimal_pe32(&pe);

    let output = run_rine_with_env(
        &pe,
        &[],
        &[
            ("RUST_LOG", "off"),
            ("RINE_RINE32_HELPER", "/definitely/nonexistent/path/rine32"),
        ],
    );

    assert!(
        !output.status.success(),
        "dispatch should fail when helper is missing"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("detected a 32-bit executable")
            && stderr.contains("cargo build --bin rine32"),
        "stderr should include actionable helper hint\nstderr:\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(dir);
}

#[cfg(unix)]
#[test]
fn x86_fixture_dispatches_to_rine32_helper() {
    if fixture_arch() != "x86" {
        return;
    }

    let Some(pe) = try_fixture("exit_zero") else {
        eprintln!("skipping x86 fixture dispatch test: x86 fixtures not found");
        return;
    };

    let dir = unique_temp_dir("rine-dispatch-x86-fixture");
    let helper = dir.join("rine32");
    let marker = dir.join("helper-arg.txt");

    std::fs::write(
        &helper,
        format!(
            "#!/usr/bin/env sh\nprintf '%s' \"$1\" > '{}'\nexit 0\n",
            marker.display()
        ),
    )
    .expect("failed to write helper script");
    make_executable(&helper);

    let output = run_rine_with_env(
        &pe,
        &[],
        &[("RINE_RINE32_HELPER", helper.to_string_lossy().as_ref())],
    );

    assert!(
        output.status.success(),
        "dispatch with x86 fixture should succeed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let seen = std::fs::read_to_string(&marker).expect("helper should write marker file");
    assert_eq!(
        seen,
        pe.display().to_string(),
        "helper should receive x86 fixture path as first argument"
    );

    let _ = std::fs::remove_dir_all(dir);
}

#[cfg(unix)]
#[test]
fn x86_dispatch_exports_window_host_socket() {
    let dir = unique_temp_dir("rine-dispatch-window-host");
    let pe = dir.join("tiny_x86_window_host.exe");
    let helper = dir.join("rine32");
    let marker = dir.join("window-host-socket.txt");

    write_minimal_pe32(&pe);
    std::fs::write(
        &helper,
        format!(
            "#!/usr/bin/env sh\nif [ -S \"$RINE_WINDOW_HOST_SOCKET\" ]; then printf '%s' \"$RINE_WINDOW_HOST_SOCKET\" > '{}'; exit 0; fi\nprintf 'missing' > '{}'; exit 1\n",
            marker.display(),
            marker.display(),
        ),
    )
    .expect("failed to write helper script");
    make_executable(&helper);

    let output = run_rine_with_env(
        &pe,
        &[],
        &[
            ("RUST_LOG", "off"),
            ("RINE_RINE32_HELPER", helper.to_string_lossy().as_ref()),
        ],
    );

    assert!(
        output.status.success(),
        "dispatch should export window host socket\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let seen = std::fs::read_to_string(&marker).expect("helper should write host socket marker");
    assert!(
        seen.trim().ends_with(".sock"),
        "helper should receive socket path, got `{seen}`"
    );

    let _ = std::fs::remove_dir_all(dir);
}
