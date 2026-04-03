// Test Win32 message queue operations.
// Tests PostMessage, PeekMessage, and PostQuitMessage.

#include <windows.h>
#include "rine_test.h"

int main(void) {
    MSG msg = {0};

    // Post a custom message
    if (!PostMessageA(NULL, WM_USER + 1, 42, 100)) {
        puts("FAIL: PostMessageA");
        return 1;
    }
    puts("PASS: PostMessageA");

    // Peek message without removing
    if (!PeekMessageA(&msg, NULL, 0, 0, PM_NOREMOVE)) {
        puts("FAIL: PeekMessageA (PM_NOREMOVE)");
        return 1;
    }
    if (msg.message != WM_USER + 1) {
        puts("FAIL: Wrong message type");
        return 1;
    }
    if (msg.wParam != 42) {
        puts("FAIL: Wrong wParam");
        return 1;
    }
    if (msg.lParam != 100) {
        puts("FAIL: Wrong lParam");
        return 1;
    }
    puts("PASS: PeekMessageA (PM_NOREMOVE)");

    // Peek message with removal
    if (!PeekMessageA(&msg, NULL, 0, 0, PM_REMOVE)) {
        puts("FAIL: PeekMessageA (PM_REMOVE)");
        return 1;
    }
    puts("PASS: PeekMessageA (PM_REMOVE)");

    // Queue should be empty now
    if (PeekMessageA(&msg, NULL, 0, 0, PM_NOREMOVE)) {
        puts("FAIL: Queue should be empty");
        return 1;
    }
    puts("PASS: Queue empty after removal");

    // Post quit message
    PostQuitMessage(99);
    puts("PASS: PostQuitMessage");

    // Should get WM_QUIT
    if (!PeekMessageA(&msg, NULL, 0, 0, PM_NOREMOVE)) {
        puts("FAIL: No WM_QUIT message");
        return 1;
    }
    if (msg.message != WM_QUIT) {
        puts("FAIL: Expected WM_QUIT");
        return 1;
    }
    if (msg.wParam != 99) {
        puts("FAIL: Wrong exit code in WM_QUIT");
        return 1;
    }
    puts("PASS: WM_QUIT received with correct exit code");

    puts("All message tests passed");
    return 0;
}
