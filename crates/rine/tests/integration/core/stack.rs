use crate::common::assert_run;

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
