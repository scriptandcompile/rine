// Allocates a large stack buffer to test stack space.
// Tests: stack setup, TEB StackBase/StackLimit.
#include <string.h>
#include "rine_test.h"

int main(void) {
    char buf[8192];
    memset(buf, 'X', sizeof(buf) - 1);
    buf[sizeof(buf) - 1] = '\0';
    put_kv_int("stack_len", (int)strlen(buf));
    return 0;
}
