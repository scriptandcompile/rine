// Tests initialized and uninitialized global data.
// Tests: .data and .bss section mapping, relocation correctness.
#include "rine_test.h"

int initialized = 42;
int uninitialized;          // .bss, should be zero
const char message[] = "global string";

int main(void) {
    put_kv_int("init", initialized);
    put_kv_int("bss", uninitialized);
    put_kv_str("str", message);
    // Modify globals
    initialized = 100;
    uninitialized = 200;
    put_kv_int("mod_init", initialized);
    put_kv_int("mod_bss", uninitialized);
    return 0;
}
