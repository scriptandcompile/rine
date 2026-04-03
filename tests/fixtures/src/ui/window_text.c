// Test Win32 window text operations.
// Tests SetWindowText, GetWindowText, and GetWindowTextLength.

#include <windows.h>
#include <string.h>
#include "rine_test.h"

LRESULT CALLBACK TestWndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    return DefWindowProc(hwnd, msg, wParam, lParam);
}

int main(void) {
    // Register window class
    WNDCLASSEXA wc = {0};
    wc.cbSize = sizeof(WNDCLASSEXA);
    wc.style = CS_HREDRAW;
    wc.lpfnWndProc = TestWndProc;
    wc.hInstance = GetModuleHandle(NULL);
    wc.lpszClassName = "TextTestClass";

    if (!RegisterClassExA(&wc)) {
        puts("FAIL: RegisterClassExA");
        return 1;
    }

    // Create window with initial title
    HWND hwnd = CreateWindowExA(
        0,
        "TextTestClass",
        "Initial Title",
        WS_OVERLAPPEDWINDOW,
        0, 0, 100, 100,
        NULL, NULL, wc.hInstance, NULL
    );

    if (!hwnd) {
        puts("FAIL: CreateWindowExA");
        return 1;
    }
    puts("PASS: CreateWindowExA");

    // Get initial window text length
    int len = GetWindowTextLengthA(hwnd);
    if (len != 13) { // "Initial Title" = 13 chars
        put_kv_int("FAIL: GetWindowTextLengthA expected 13, got", len);
        return 1;
    }
    puts("PASS: GetWindowTextLengthA (initial)");

    // Get initial window text
    char buffer[64];
    int copied = GetWindowTextA(hwnd, buffer, sizeof(buffer));
    if (copied != 13) {
        put_kv_int("FAIL: GetWindowTextA expected 13, got", copied);
        return 1;
    }
    if (strcmp(buffer, "Initial Title") != 0) {
        put_kv_str("FAIL: GetWindowTextA expected 'Initial Title', got", buffer);
        return 1;
    }
    puts("PASS: GetWindowTextA (initial)");

    // Set new window text
    if (!SetWindowTextA(hwnd, "New Title")) {
        puts("FAIL: SetWindowTextA");
        return 1;
    }
    puts("PASS: SetWindowTextA");

    // Get new window text length
    len = GetWindowTextLengthA(hwnd);
    if (len != 9) { // "New Title" = 9 chars
        put_kv_int("FAIL: GetWindowTextLengthA expected 9, got", len);
        return 1;
    }
    puts("PASS: GetWindowTextLengthA (after set)");

    // Get new window text
    memset(buffer, 0, sizeof(buffer));
    copied = GetWindowTextA(hwnd, buffer, sizeof(buffer));
    if (copied != 9) {
        put_kv_int("FAIL: GetWindowTextA expected 9, got", copied);
        return 1;
    }
    if (strcmp(buffer, "New Title") != 0) {
        put_kv_str("FAIL: GetWindowTextA expected 'New Title', got", buffer);
        return 1;
    }
    puts("PASS: GetWindowTextA (after set)");

    // Test buffer too small
    char small_buf[5];
    copied = GetWindowTextA(hwnd, small_buf, sizeof(small_buf));
    if (copied != 4) { // Should only copy 4 chars + null
        put_kv_int("FAIL: GetWindowTextA with small buffer expected 4, got", copied);
        return 1;
    }
    if (strcmp(small_buf, "New ") != 0) {
        put_kv_str("FAIL: GetWindowTextA with small buffer expected 'New ', got", small_buf);
        return 1;
    }
    puts("PASS: GetWindowTextA (buffer truncation)");

    // Cleanup
    DestroyWindow(hwnd);
    UnregisterClassA("TextTestClass", wc.hInstance);

    puts("All window text tests passed");
    return 0;
}
