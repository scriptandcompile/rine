// Tests command-line argument retrieval.
// Tests: GetCommandLineA, __getmainargs, argc/argv propagation.
#include "rine_test.h"

int main(int argc, char *argv[]) {
    put_kv_int("argc", argc);
    for (int i = 0; i < argc; i++) {
        puts(argv[i]);
    }
    return 0;
}
