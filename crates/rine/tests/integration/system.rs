use super::common::assert_run;

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
