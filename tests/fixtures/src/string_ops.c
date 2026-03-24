// Tests string/memory operations.
// Tests: strlen, strncmp, memcpy, memset.
#include <string.h>
#include "rine_test.h"

// Manual comparison to avoid pulling in strcmp (not yet in rine).
static int str_eq(const char *a, const char *b) {
    while (*a && *b) {
        if (*a != *b) return 0;
        a++; b++;
    }
    return *a == *b;
}

int main(void) {
    // strlen
    const char *s = "hello";
    if (strlen(s) != 5) {
        puts("FAIL: strlen");
        return 1;
    }

    // strncmp
    if (strncmp("abc", "abd", 2) != 0) {
        puts("FAIL: strncmp equal prefix");
        return 2;
    }
    if (strncmp("abc", "abd", 3) >= 0) {
        puts("FAIL: strncmp differ");
        return 3;
    }

    // memcpy
    char dst[16];
    memcpy(dst, "copied!", 8);  // includes NUL
    if (!str_eq(dst, "copied!")) {
        puts("FAIL: memcpy");
        return 4;
    }

    // memset
    char buf[8];
    memset(buf, 'A', 7);
    buf[7] = '\0';
    if (!str_eq(buf, "AAAAAAA")) {
        puts("FAIL: memset");
        return 5;
    }

    puts("string_ops: ok");
    return 0;
}
