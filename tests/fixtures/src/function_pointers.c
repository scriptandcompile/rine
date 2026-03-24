// Tests function pointer calls (indirect calls through PE code).
// Tests: correct relocation of function addresses, call semantics.
#include "rine_test.h"

typedef int (*binop)(int, int);

int add(int a, int b) { return a + b; }
int mul(int a, int b) { return a * b; }

int apply(binop fn, int a, int b) {
    return fn(a, b);
}

int main(void) {
    put_kv_int("add", apply(add, 3, 4));
    put_kv_int("mul", apply(mul, 3, 4));
    return 0;
}
