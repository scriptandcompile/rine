// Multiple printf calls with different format specifiers.
// Tests: repeated variadic calls, integer/string/hex formatting.
#include <stdio.h>

int main(void) {
    printf("int: %d\n", 42);
    printf("hex: %x\n", 255);
    printf("str: %s\n", "test");
    printf("multi: %d %s %d\n", 1, "two", 3);
    return 0;
}
