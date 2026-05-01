use super::common::{assert_run, fixture, run_rine};

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

#[test]
fn create_process() {
    assert_run("create_process", 0, "CreateProcessA ok\n");
}

#[test]
fn sync_primitives() {
    assert_run(
        "sync_primitives",
        0,
        "cs: ok\n\
         events: ok\n\
         events_w: ok\n\
         auto_reset: ok\n\
         auto_reset_w: ok\n\
         mutex: ok\n\
         mutex_w: ok\n\
         mutex_recursive: ok\n\
         mutex_recursive_w: ok\n\
         semaphore: ok\n\
         semaphore_w: ok\n\
         sem_release: ok\n\
         sem_release_w: ok",
    );
}

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

#[test]
fn last_error() {
    assert_run(
        "last_error",
        0,
        "last_error_roundtrip: ok\n\
         last_error_overwrite: ok\n\
         last_error_thread_local: ok",
    );
}

#[test]
fn get_exit_code_process_last_error() {
    assert_run(
        "get_exit_code_process_last_error",
        0,
        "get_exit_code_process_null_out_error: ok\n\
         get_exit_code_process_invalid_handle_error: ok",
    );
}

#[test]
fn set_unhandled_exception_filter() {
    let output = run_rine(&fixture("set_unhandled_exception_filter"), &[]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let code = output.status.code().unwrap_or(-1);

    assert_eq!(
        code, 139,
        "expected SIGSEGV-style exit code 139\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("seh_filter_called: ok"),
        "expected filter marker in stdout\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
}
