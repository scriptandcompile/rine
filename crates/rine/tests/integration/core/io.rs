use crate::common::assert_run;

#[test]
fn hello_puts() {
    assert_run("hello_puts", 0, "Hello from rine!");
}

#[test]
fn write_console_a() {
    assert_run("write_console", 0, "WriteConsoleA works");
}

#[test]
fn find_close() {
    assert_run("find_close", 0, "FindClose works");
}

#[test]
fn write_file_stdout() {
    assert_run("write_file", 0, "WriteFile works");
}

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
