// Test basic Win32 window class registration and window creation.
// Registers a window class, creates a window, and destroys it.

#include <windows.h>
#include "rine_test.h"

LRESULT CALLBACK TestWndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    return DefWindowProc(hwnd, msg, wParam, lParam);
}

int main(void) {
    WNDCLASSEXA wc = {0};
    wc.cbSize = sizeof(WNDCLASSEXA);
    wc.style = CS_HREDRAW | CS_VREDRAW;
    wc.lpfnWndProc = TestWndProc;
    wc.hInstance = GetModuleHandle(NULL);
    wc.lpszClassName = "TestWindowClass";

    // Register window class
    if (!RegisterClassExA(&wc)) {
        puts("FAIL: RegisterClassExA");
        return 1;
    }
    puts("PASS: RegisterClassExA");

    // Create window
    HWND hwnd = CreateWindowExA(
        0,                              // dwExStyle
        "TestWindowClass",              // lpClassName
        "Test Window",                  // lpWindowName
        WS_OVERLAPPEDWINDOW,            // dwStyle
        100, 100,                       // x, y
        640, 480,                       // width, height
        NULL,                           // hWndParent
        NULL,                           // hMenu
        wc.hInstance,                   // hInstance
        NULL                            // lpParam
    );

    if (!hwnd) {
        puts("FAIL: CreateWindowExA");
        return 1;
    }
    puts("PASS: CreateWindowExA");

    // Destroy window
    if (!DestroyWindow(hwnd)) {
        puts("FAIL: DestroyWindow");
        return 1;
    }
    puts("PASS: DestroyWindow");

    // Unregister class
    if (!UnregisterClassA("TestWindowClass", wc.hInstance)) {
        puts("FAIL: UnregisterClassA");
        return 1;
    }
    puts("PASS: UnregisterClassA");

    puts("All window basic tests passed");
    return 0;
}
