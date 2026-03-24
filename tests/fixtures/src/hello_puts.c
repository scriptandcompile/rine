// Prints via puts() (simplest I/O path).
// Tests: CRT init, puts → libc::puts forwarding, stdout capture.
#include <stdio.h>

int main(void) {
    puts("Hello from rine!");
    return 0;
}
