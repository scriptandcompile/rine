// Tests environment variable functions from kernel32.
// Tests: GetEnvironmentVariableA, SetEnvironmentVariableA,
//        ExpandEnvironmentStringsA, GetEnvironmentStringsW,
//        FreeEnvironmentStringsW.
#include <windows.h>
#include "rine_test.h"

// ── GetEnvironmentVariableA for a pre-seeded variable ──────────

static int test_get_existing(void) {
    char buf[32];
    DWORD ret = GetEnvironmentVariableA("OS", buf, sizeof(buf));
    if (ret == 0) return 1;
    if (strcmp(buf, "Windows_NT") != 0) return 1;
    return 0;
}

// ── GetEnvironmentVariableA for a variable that doesn't exist ──

static int test_get_missing(void) {
    char buf[32];
    DWORD ret = GetEnvironmentVariableA("RINE_NO_SUCH_VAR_XYZ", buf, sizeof(buf));
    // Should return 0 for missing variable.
    if (ret != 0) return 1;
    return 0;
}

// ── GetEnvironmentVariableA with too-small buffer ──────────────

static int test_get_small_buffer(void) {
    // "Windows_NT" is 10 chars, needs 11 bytes with NUL.
    char buf[4];
    DWORD ret = GetEnvironmentVariableA("OS", buf, sizeof(buf));
    // Should return required size (including NUL terminator = 11).
    if (ret < 11) return 1;
    return 0;
}

// ── SetEnvironmentVariableA + read back ────────────────────────

static int test_set_and_get(void) {
    BOOL ok = SetEnvironmentVariableA("RINE_TEST_VAR", "hello_rine");
    if (!ok) return 1;

    char buf[64];
    DWORD ret = GetEnvironmentVariableA("RINE_TEST_VAR", buf, sizeof(buf));
    if (ret == 0) return 1;
    if (strcmp(buf, "hello_rine") != 0) return 1;
    return 0;
}

// ── SetEnvironmentVariableA with NULL deletes the variable ─────

static int test_set_delete(void) {
    SetEnvironmentVariableA("RINE_DEL_VAR", "temp_value");

    // Delete by passing NULL.
    BOOL ok = SetEnvironmentVariableA("RINE_DEL_VAR", NULL);
    if (!ok) return 1;

    char buf[64];
    DWORD ret = GetEnvironmentVariableA("RINE_DEL_VAR", buf, sizeof(buf));
    // Should return 0 — variable was deleted.
    if (ret != 0) return 1;
    return 0;
}

// ── Case-insensitive lookup ────────────────────────────────────

static int test_case_insensitive(void) {
    SetEnvironmentVariableA("RineCase", "CaseVal");

    char buf[32];
    DWORD ret = GetEnvironmentVariableA("RINECASE", buf, sizeof(buf));
    if (ret == 0) return 1;
    if (strcmp(buf, "CaseVal") != 0) return 1;
    return 0;
}

// ── ExpandEnvironmentStringsA ──────────────────────────────────

static int test_expand(void) {
    // %OS% should expand to "Windows_NT".
    char buf[128];
    DWORD ret = ExpandEnvironmentStringsA("val=%OS%", buf, sizeof(buf));
    if (ret == 0) return 1;
    if (strcmp(buf, "val=Windows_NT") != 0) return 1;
    return 0;
}

// ── ExpandEnvironmentStringsA with undefined variable ──────────

static int test_expand_undefined(void) {
    // Undefined variables should pass through unchanged.
    char buf[128];
    DWORD ret = ExpandEnvironmentStringsA("%NOSUCHVAR%", buf, sizeof(buf));
    if (ret == 0) return 1;
    if (strcmp(buf, "%NOSUCHVAR%") != 0) return 1;
    return 0;
}

// ── GetEnvironmentStringsW returns a non-null block ────────────

static int test_get_strings_w(void) {
    LPWCH block = GetEnvironmentStringsW();
    if (block == NULL) return 1;

    // The first entry should be non-empty (at least one var is set).
    if (block[0] == 0) return 1;

    // Walk the block and count entries.
    int count = 0;
    LPWCH p = block;
    while (*p != 0) {
        count++;
        while (*p != 0) p++;
        p++; // skip NUL separator
    }
    // We seed ~20 variables; just check we got a reasonable number.
    if (count < 5) return 1;

    FreeEnvironmentStringsW(block);
    return 0;
}

// ── main ────────────────────────────────────────────────────────

int main(void) {
    puts(test_get_existing()     == 0 ? "get_existing: ok"     : "FAIL: get_existing");
    puts(test_get_missing()      == 0 ? "get_missing: ok"      : "FAIL: get_missing");
    puts(test_get_small_buffer() == 0 ? "get_small_buffer: ok" : "FAIL: get_small_buffer");
    puts(test_set_and_get()      == 0 ? "set_and_get: ok"      : "FAIL: set_and_get");
    puts(test_set_delete()       == 0 ? "set_delete: ok"       : "FAIL: set_delete");
    puts(test_case_insensitive() == 0 ? "case_insensitive: ok" : "FAIL: case_insensitive");
    puts(test_expand()           == 0 ? "expand: ok"           : "FAIL: expand");
    puts(test_expand_undefined() == 0 ? "expand_undefined: ok" : "FAIL: expand_undefined");
    puts(test_get_strings_w()    == 0 ? "get_strings_w: ok"    : "FAIL: get_strings_w");
    return 0;
}
