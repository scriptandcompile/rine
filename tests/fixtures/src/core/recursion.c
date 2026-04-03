// Tests recursive function calls (stack depth).
// Tests: proper stack frame setup, calling convention.
#include "rine_test.h"

int fibonacci(int n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

int main(void) {
    int result = fibonacci(20);
    put_kv_int("fib(20)", result);
    return 0;
}
