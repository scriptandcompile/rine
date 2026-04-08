// Uses Win32 API directly (no CRT I/O).
// Tests: GetStdHandle, WriteConsoleA, kernel32 import resolution.
#include <windows.h>

int main(void) {
    HANDLE hOut = GetStdHandle(STD_OUTPUT_HANDLE);
    const char msg[] = "WriteConsoleA ok\n";
    DWORD written;
    WriteConsoleA(hOut, msg, sizeof(msg) - 1, &written, NULL);
    return 0;
}
