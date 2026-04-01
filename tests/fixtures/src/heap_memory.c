// Tests heap management and virtual memory allocation.
// Tests: GetProcessHeap, HeapAlloc, HeapFree, HeapReAlloc, HeapCreate,
//        HeapDestroy, VirtualAlloc, VirtualFree.
#include <windows.h>
#include "rine_test.h"

// ── HeapAlloc / HeapFree on default heap ────────────────────────

static int test_heap_alloc_free(void) {
    HANDLE heap = GetProcessHeap();
    if (heap == NULL) return 1;

    // Allocate, write, read, free.
    char *buf = (char *)HeapAlloc(heap, 0, 64);
    if (buf == NULL) return 1;

    for (int i = 0; i < 64; i++) buf[i] = (char)(i & 0xFF);
    for (int i = 0; i < 64; i++) {
        if (buf[i] != (char)(i & 0xFF)) return 1;
    }

    if (!HeapFree(heap, 0, buf)) return 1;
    return 0;
}

// ── HeapAlloc with HEAP_ZERO_MEMORY ─────────────────────────────

static int test_heap_zero_memory(void) {
    HANDLE heap = GetProcessHeap();
    char *buf = (char *)HeapAlloc(heap, HEAP_ZERO_MEMORY, 128);
    if (buf == NULL) return 1;

    for (int i = 0; i < 128; i++) {
        if (buf[i] != 0) return 1;
    }

    HeapFree(heap, 0, buf);
    return 0;
}

// ── HeapReAlloc ─────────────────────────────────────────────────

static int test_heap_realloc(void) {
    HANDLE heap = GetProcessHeap();
    char *buf = (char *)HeapAlloc(heap, 0, 16);
    if (buf == NULL) return 1;

    // Fill original 16 bytes.
    for (int i = 0; i < 16; i++) buf[i] = 'A';

    // Grow to 64 bytes.
    char *newbuf = (char *)HeapReAlloc(heap, 0, buf, 64);
    if (newbuf == NULL) return 1;

    // Check original data preserved.
    for (int i = 0; i < 16; i++) {
        if (newbuf[i] != 'A') return 1;
    }

    // Write to extended region.
    for (int i = 16; i < 64; i++) newbuf[i] = 'B';
    if (newbuf[63] != 'B') return 1;

    HeapFree(heap, 0, newbuf);
    return 0;
}

// ── HeapCreate / HeapDestroy ────────────────────────────────────

static int test_heap_create_destroy(void) {
    HANDLE heap = HeapCreate(0, 0, 0);
    if (heap == NULL) return 1;

    // Allocate on the custom heap.
    int *val = (int *)HeapAlloc(heap, HEAP_ZERO_MEMORY, sizeof(int));
    if (val == NULL) return 1;
    if (*val != 0) return 1;

    *val = 42;
    if (*val != 42) return 1;

    HeapFree(heap, 0, val);

    // Destroy the heap.
    if (!HeapDestroy(heap)) return 1;
    return 0;
}

// ── VirtualAlloc / VirtualFree ──────────────────────────────────

static int test_virtual_alloc_free(void) {
    // Allocate 4 KB of read-write memory.
    void *mem = VirtualAlloc(NULL, 4096, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
    if (mem == NULL) return 1;

    // Write and read the entire page.
    char *p = (char *)mem;
    for (int i = 0; i < 4096; i++) p[i] = (char)(i & 0xFF);
    if (p[0] != 0) return 1;
    if (p[255] != (char)255) return 1;
    if (p[4095] != (char)(4095 & 0xFF)) return 1;

    // Free the region.
    if (!VirtualFree(mem, 0, MEM_RELEASE)) return 1;
    return 0;
}

// ── VirtualAlloc large region ───────────────────────────────────

static int test_virtual_alloc_large(void) {
    // Allocate 64 KB.
    SIZE_T size = 64 * 1024;
    char *mem = (char *)VirtualAlloc(NULL, size, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
    if (mem == NULL) return 1;

    // Touch first and last bytes.
    mem[0] = 1;
    mem[size - 1] = 2;
    if (mem[0] != 1) return 1;
    if (mem[size - 1] != 2) return 1;

    if (!VirtualFree(mem, 0, MEM_RELEASE)) return 1;
    return 0;
}

// ── Multiple allocations ────────────────────────────────────────

static int test_multiple_allocs(void) {
    HANDLE heap = GetProcessHeap();
    #define N_ALLOCS 32
    char *ptrs[N_ALLOCS];

    // Allocate many blocks.
    for (int i = 0; i < N_ALLOCS; i++) {
        ptrs[i] = (char *)HeapAlloc(heap, 0, 64 + i * 16);
        if (ptrs[i] == NULL) return 1;
        ptrs[i][0] = (char)i;
    }

    // Verify tags.
    for (int i = 0; i < N_ALLOCS; i++) {
        if (ptrs[i][0] != (char)i) return 1;
    }

    // Free all.
    for (int i = 0; i < N_ALLOCS; i++) {
        if (!HeapFree(heap, 0, ptrs[i])) return 1;
    }
    return 0;
}

// ── Main ────────────────────────────────────────────────────────

int main(void) {
    if (test_heap_alloc_free() != 0) { puts("FAIL: heap_alloc_free"); return 1; }
    puts("heap_alloc_free: ok");

    if (test_heap_zero_memory() != 0) { puts("FAIL: heap_zero_memory"); return 1; }
    puts("heap_zero_memory: ok");

    if (test_heap_realloc() != 0) { puts("FAIL: heap_realloc"); return 1; }
    puts("heap_realloc: ok");

    if (test_heap_create_destroy() != 0) { puts("FAIL: heap_create_destroy"); return 1; }
    puts("heap_create_destroy: ok");

    if (test_virtual_alloc_free() != 0) { puts("FAIL: virtual_alloc_free"); return 1; }
    puts("virtual_alloc_free: ok");

    if (test_virtual_alloc_large() != 0) { puts("FAIL: virtual_alloc_large"); return 1; }
    puts("virtual_alloc_large: ok");

    if (test_multiple_allocs() != 0) { puts("FAIL: multiple_allocs"); return 1; }
    puts("multiple_allocs: ok");

    return 0;
}
