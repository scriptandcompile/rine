//! Integration tests for the rine PE loader.
//!
//! Each test compiles a small Windows C program (pre-built in tests/fixtures/bin/),
//! runs it through the `rine` binary, and asserts on exit code + stdout.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Path to the workspace root (two levels up from crate manifest).
fn workspace_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(|p| p.parent())
        .expect("could not find workspace root")
        .to_path_buf()
}

/// Path to a pre-built fixture executable.
fn fixture(name: &str) -> PathBuf {
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

/// Run a fixture through rine, returning the process output.
fn run_rine(fixture_path: &Path, extra_args: &[&str]) -> Output {
    let rine = env!("CARGO_BIN_EXE_rine");
    let mut cmd = Command::new(rine);
    cmd.arg(fixture_path);
    for arg in extra_args {
        cmd.arg(arg);
    }
    // Suppress tracing output (goes to stderr) from polluting test output.
    cmd.env("RUST_LOG", "off");
    cmd.output().expect("failed to execute rine")
}

/// Assert a fixture produces the expected exit code and stdout content.
fn assert_run(name: &str, expected_code: i32, expected_stdout: &str) {
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

/// Assert a fixture produces the expected exit code (ignoring stdout).
fn assert_exit_code(name: &str, expected_code: i32) {
    let output = run_rine(&fixture(name), &[]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let code = output.status.code().unwrap_or(-1);

    assert_eq!(
        code, expected_code,
        "\n--- {name} ---\nexpected exit code {expected_code}, got {code}\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
}

// ============================================================================
// Minimal PE loading — exit codes
// ============================================================================

#[test]
fn exit_zero() {
    assert_run("exit_zero", 0, "");
}

#[test]
fn exit_code_42() {
    assert_exit_code("exit_code", 42);
}

// ============================================================================
// CRT I/O — puts, WriteConsoleA, WriteFile
// ============================================================================

#[test]
fn hello_puts() {
    assert_run("hello_puts", 0, "Hello from rine!");
}

#[test]
fn write_console_a() {
    assert_run("write_console", 0, "WriteConsoleA works");
}

#[test]
fn write_file_stdout() {
    assert_run("write_file", 0, "WriteFile works");
}

// ============================================================================
// Heap — malloc, calloc, realloc, free
// ============================================================================

#[test]
fn malloc_free() {
    assert_run("malloc_free", 0, "heap works");
}

#[test]
fn calloc_realloc() {
    assert_run(
        "calloc_realloc",
        0,
        "calloc_realloc[0]: 10\ncalloc_realloc[1]: 20",
    );
}

// ============================================================================
// String/memory functions
// ============================================================================

#[test]
fn string_ops() {
    assert_run("string_ops", 0, "string_ops: ok");
}

// ============================================================================
// Process lifecycle
// ============================================================================

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

// ============================================================================
// Data sections — .data, .bss, relocations
// ============================================================================

#[test]
fn global_data() {
    assert_run(
        "global_data",
        0,
        "init: 42\nbss: 0\nstr: global string\nmod_init: 100\nmod_bss: 200",
    );
}

// ============================================================================
// Stack & calling conventions
// ============================================================================

#[test]
fn large_stack() {
    assert_run("large_stack", 0, "stack_len: 8191");
}

#[test]
fn recursion() {
    assert_run("recursion", 0, "fib(20): 6765");
}

#[test]
fn function_pointers() {
    assert_run("function_pointers", 0, "add: 7\nmul: 12");
}

#[test]
fn struct_layout() {
    assert_run("struct_layout", 0, "area: 4\nsizeof_rect: 16");
}

// ============================================================================
// Command-line arguments
// ============================================================================

#[test]
fn cmdline_no_extra_args() {
    let output = run_rine(&fixture("cmdline"), &[]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let code = output.status.code().unwrap_or(-1);

    assert_eq!(code, 0);
    // argc should be at least 1 (the exe path itself is passed)
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

// ============================================================================
// Process & threading
// ============================================================================

#[test]
fn process_threads() {
    assert_run(
        "process_threads",
        0,
        "pid: ok\n\
         pseudo_handle: ok\n\
         thread_exit: ok\n\
         thread_param: ok\n\
         wait_multiple: ok\n\
         wait_timeout: ok\n\
         sleep: ok",
    );
}

// ============================================================================
// Synchronization primitives
// ============================================================================

#[test]
fn sync_primitives() {
    assert_run(
        "sync_primitives",
        0,
        "cs: ok\n\
         events: ok\n\
         auto_reset: ok\n\
         mutex: ok\n\
         mutex_recursive: ok\n\
         semaphore: ok\n\
         sem_release: ok",
    );
}

// ============================================================================
// Heap management & virtual memory
// ============================================================================

#[test]
fn heap_memory() {
    assert_run(
        "heap_memory",
        0,
        "heap_alloc_free: ok\n\
         heap_zero_memory: ok\n\
         heap_realloc: ok\n\
         heap_create_destroy: ok\n\
         virtual_alloc_free: ok\n\
         virtual_alloc_large: ok\n\
         multiple_allocs: ok",
    );
}

// ============================================================================
// Registry emulation
// ============================================================================

#[test]
fn registry_ops() {
    assert_run(
        "registry_ops",
        0,
        "reg_open_existing: ok\n\
         reg_open_missing: ok\n\
         reg_query_dword: ok\n\
         reg_query_string: ok\n\
         reg_create_set_query: ok\n\
         reg_set_string: ok\n\
         reg_close_predefined: ok",
    );
}

// ============================================================================
// Environment variables
// ============================================================================

#[test]
fn env_ops() {
    assert_run(
        "env_ops",
        0,
        "get_existing: ok\n\
         get_missing: ok\n\
         get_small_buffer: ok\n\
         set_and_get: ok\n\
         set_delete: ok\n\
         case_insensitive: ok\n\
         expand: ok\n\
         expand_undefined: ok\n\
         get_strings_w: ok",
    );
}

// ============================================================================
// Printf (known failing — tracks localeconv/fputc implementation)
// ============================================================================

#[test]
#[ignore = "requires localeconv/fputc stubs (MinGW CRT dependency)"]
fn hello_printf() {
    assert_run("hello_printf", 0, "hello world 2025");
}

#[test]
#[ignore = "requires localeconv/fputc stubs (MinGW CRT dependency)"]
fn printf_multi() {
    assert_run(
        "printf_multi",
        0,
        "int: 42\nhex: ff\nstr: test\nmulti: 1 two 3",
    );
}

// ============================================================================
// Error handling: invalid inputs
// ============================================================================

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
    // Feed rine itself (an ELF binary) as input — should be rejected as not PE.
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
