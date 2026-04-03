use crate::common::assert_run;

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
