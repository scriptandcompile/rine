// Prints via printf() with format specifiers.
// Tests: naked asm Win64→SysV ABI thunk, variadic forwarding.
#include <stdio.h>

int main(void) {
    printf("hello %s %d\n", "world", 2025);
    return 0;
}
