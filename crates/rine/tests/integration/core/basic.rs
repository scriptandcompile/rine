use crate::common::{assert_exit_code, assert_run};

#[test]
fn exit_zero() {
    assert_run("exit_zero", 0, "");
}

#[test]
fn exit_code_42() {
    assert_exit_code("exit_code", 42);
}

#[test]
fn global_data() {
    assert_run(
        "global_data",
        0,
        "init: 42\nbss: 0\nstr: global string\nmod_init: 100\nmod_bss: 200",
    );
}

#[test]
fn string_ops() {
    assert_run("string_ops", 0, "string_ops: ok");
}
