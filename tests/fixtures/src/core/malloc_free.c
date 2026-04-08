// Basic heap allocation cycle.
// Tests: malloc/free forwarding to Rust allocator.
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(void) {
    char *buf = (char *)malloc(64);
    if (!buf) {
        puts("FAIL: malloc returned NULL");
        return 1;
    }
    strcpy(buf, "heap ok");
    puts(buf);
    free(buf);
    return 0;
}
