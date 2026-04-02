// Test Win32 window visibility operations.
// Tests ShowWindow and window visibility states.

#include <windows.h>
#include "rine_test.h"

LRESULT CALLBACK TestWndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    return DefWindowProc(hwnd, msg, wParam, lParam);
}

int main(void) {
    // Register window class
    WNDCLASSEXA wc = {0};
    wc.cbSize = sizeof(WNDCLASSEXA);
    wc.style = 0;
    wc.lpfnWndProc = TestWndProc;
    wc.hInstance = GetModuleHandle(NULL);
    wc.lpszClassName = "ShowTestClass";

    if (!RegisterClassExA(&wc)) {
        puts("FAIL: RegisterClassExA");
        return 1;
    }

    // Create window without WS_VISIBLE
    HWND hwnd = CreateWindowExA(
        0,
        "ShowTestClass",
        "Test",
        WS_OVERLAPPEDWINDOW, // Not visible initially
        0, 0, 100, 100,
        NULL, NULL, wc.hInstance, NULL
    );

    if (!hwnd) {
        puts("FAIL: CreateWindowExA");
        return 1;
    }
    puts("PASS: CreateWindowExA");

    // ShowWindow should return 0 (was not visible)
    int was_visible = ShowWindow(hwnd, SW_SHOW);
    if (was_visible != 0) {
        put_kv_int("FAIL: ShowWindow expected 0 (not visible), got", was_visible);
        return 1;
    }
    puts("PASS: ShowWindow with SW_SHOW (was not visible)");

    // ShowWindow again should return non-zero (now visible)
    was_visible = ShowWindow(hwnd, SW_HIDE);
    if (was_visible == 0) {
        puts("FAIL: ShowWindow expected non-zero (was visible)");
        return 1;
    }
    puts("PASS: ShowWindow with SW_HIDE (was visible)");

    // ShowWindow with already hidden should return 0
    was_visible = ShowWindow(hwnd, SW_HIDE);
    if (was_visible != 0) {
        put_kv_int("FAIL: ShowWindow expected 0 (already hidden), got", was_visible);
        return 1;
    }
    puts("PASS: ShowWindow with SW_HIDE (already hidden)");

    // UpdateWindow
    if (!UpdateWindow(hwnd)) {
        puts("FAIL: UpdateWindow");
        return 1;
    }
    puts("PASS: UpdateWindow");

    // Cleanup
    DestroyWindow(hwnd);
    UnregisterClassA("ShowTestClass", wc.hInstance);

    puts("All window show tests passed");
    return 0;
}
