// Tests registry emulation via advapi32 functions.
// Tests: RegOpenKeyExA, RegQueryValueExA, RegSetValueExA,
//        RegCreateKeyExA, RegCloseKey.
#include <windows.h>
#include "rine_test.h"

// ── RegOpenKeyExA on pre-populated HKLM key ────────────────────

static int test_reg_open_existing(void) {
    HKEY hkey;
    LONG rc = RegOpenKeyExA(
        HKEY_LOCAL_MACHINE,
        "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion",
        0, KEY_READ, &hkey);
    if (rc != ERROR_SUCCESS) return 1;

    RegCloseKey(hkey);
    return 0;
}

// ── RegOpenKeyExA on non-existent key ──────────────────────────

static int test_reg_open_missing(void) {
    HKEY hkey;
    LONG rc = RegOpenKeyExA(
        HKEY_LOCAL_MACHINE,
        "SOFTWARE\\NoSuchVendor\\NoSuchKey",
        0, KEY_READ, &hkey);
    // Should fail with ERROR_FILE_NOT_FOUND (2)
    if (rc == ERROR_FILE_NOT_FOUND) return 0;
    return 1;
}

// ── RegQueryValueExA reading a DWORD ───────────────────────────

static int test_reg_query_dword(void) {
    HKEY hkey;
    LONG rc = RegOpenKeyExA(
        HKEY_LOCAL_MACHINE,
        "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion",
        0, KEY_READ, &hkey);
    if (rc != ERROR_SUCCESS) return 1;

    DWORD value = 0;
    DWORD type = 0;
    DWORD size = sizeof(value);
    rc = RegQueryValueExA(hkey, "CurrentMajorVersionNumber",
                          NULL, &type, (LPBYTE)&value, &size);
    if (rc != ERROR_SUCCESS) return 1;
    if (type != REG_DWORD) return 1;
    if (value != 10) return 1;

    RegCloseKey(hkey);
    return 0;
}

// ── RegQueryValueExA reading a string ──────────────────────────

static int test_reg_query_string(void) {
    HKEY hkey;
    LONG rc = RegOpenKeyExA(
        HKEY_LOCAL_MACHINE,
        "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion",
        0, KEY_READ, &hkey);
    if (rc != ERROR_SUCCESS) return 1;

    char buf[256];
    DWORD type = 0;
    DWORD size = sizeof(buf);
    rc = RegQueryValueExA(hkey, "ProductName",
                          NULL, &type, (LPBYTE)buf, &size);
    if (rc != ERROR_SUCCESS) return 1;
    if (type != REG_SZ) return 1;
    // The pre-populated value is "Windows 10 Pro"
    // Just check it starts with "Windows"
    if (buf[0] != 'W') return 1;

    RegCloseKey(hkey);
    return 0;
}

// ── RegCreateKeyExA + RegSetValueExA + read back ───────────────

static int test_reg_create_set_query(void) {
    HKEY hkey;
    DWORD disposition = 0;
    LONG rc = RegCreateKeyExA(
        HKEY_CURRENT_USER,
        "Software\\RineTest\\SubKey",
        0, NULL, 0, KEY_ALL_ACCESS, NULL, &hkey, &disposition);
    if (rc != ERROR_SUCCESS) return 1;

    // Write a DWORD value.
    DWORD val = 12345;
    rc = RegSetValueExA(hkey, "TestDword", 0, REG_DWORD,
                        (const BYTE *)&val, sizeof(val));
    if (rc != ERROR_SUCCESS) return 1;

    // Read it back.
    DWORD read_val = 0;
    DWORD type = 0;
    DWORD size = sizeof(read_val);
    rc = RegQueryValueExA(hkey, "TestDword", NULL, &type,
                          (LPBYTE)&read_val, &size);
    if (rc != ERROR_SUCCESS) return 1;
    if (type != REG_DWORD) return 1;
    if (read_val != 12345) return 1;

    RegCloseKey(hkey);
    return 0;
}

// ── RegSetValueExA string + read back ──────────────────────────

static int test_reg_set_string(void) {
    HKEY hkey;
    DWORD disposition = 0;
    LONG rc = RegCreateKeyExA(
        HKEY_CURRENT_USER,
        "Software\\RineTest\\Strings",
        0, NULL, 0, KEY_ALL_ACCESS, NULL, &hkey, &disposition);
    if (rc != ERROR_SUCCESS) return 1;

    const char *str_val = "hello rine";
    rc = RegSetValueExA(hkey, "Greeting", 0, REG_SZ,
                        (const BYTE *)str_val, strlen(str_val) + 1);
    if (rc != ERROR_SUCCESS) return 1;

    char buf[64];
    DWORD type = 0;
    DWORD size = sizeof(buf);
    rc = RegQueryValueExA(hkey, "Greeting", NULL, &type,
                          (LPBYTE)buf, &size);
    if (rc != ERROR_SUCCESS) return 1;
    if (type != REG_SZ) return 1;
    if (strcmp(buf, "hello rine") != 0) return 1;

    RegCloseKey(hkey);
    return 0;
}

// ── RegCloseKey on predefined root ─────────────────────────────

static int test_reg_close_predefined(void) {
    // Closing a predefined key should succeed (no-op).
    LONG rc = RegCloseKey(HKEY_LOCAL_MACHINE);
    if (rc != ERROR_SUCCESS) return 1;
    return 0;
}

// ── main ────────────────────────────────────────────────────────

int main(void) {
    puts(test_reg_open_existing()   == 0 ? "reg_open_existing: ok"   : "FAIL: reg_open_existing");
    puts(test_reg_open_missing()    == 0 ? "reg_open_missing: ok"    : "FAIL: reg_open_missing");
    puts(test_reg_query_dword()     == 0 ? "reg_query_dword: ok"     : "FAIL: reg_query_dword");
    puts(test_reg_query_string()    == 0 ? "reg_query_string: ok"    : "FAIL: reg_query_string");
    puts(test_reg_create_set_query()== 0 ? "reg_create_set_query: ok": "FAIL: reg_create_set_query");
    puts(test_reg_set_string()      == 0 ? "reg_set_string: ok"      : "FAIL: reg_set_string");
    puts(test_reg_close_predefined()== 0 ? "reg_close_predefined: ok": "FAIL: reg_close_predefined");
    return 0;
}
