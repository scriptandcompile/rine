// Tests calloc zero-initialization and realloc growth.
// Tests: calloc, realloc, free.
#include <stdlib.h>
#include "rine_test.h"

int main(void) {
    // calloc should zero-initialize
    int *arr = (int *)calloc(4, sizeof(int));
    if (!arr) {
        puts("FAIL: calloc returned NULL");
        return 1;
    }
    if (arr[0] != 0 || arr[1] != 0 || arr[2] != 0 || arr[3] != 0) {
        puts("FAIL: calloc not zeroed");
        return 2;
    }
    arr[0] = 10;
    arr[1] = 20;

    // realloc to larger
    arr = (int *)realloc(arr, 8 * sizeof(int));
    if (!arr) {
        puts("FAIL: realloc returned NULL");
        return 3;
    }
    // original data preserved
    if (arr[0] != 10 || arr[1] != 20) {
        puts("FAIL: realloc lost data");
        return 4;
    }

    put_kv_int("calloc_realloc[0]", arr[0]);
    put_kv_int("calloc_realloc[1]", arr[1]);
    free(arr);
    return 0;
}
