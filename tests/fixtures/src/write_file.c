// Uses WriteFile on stdout handle.
// Tests: GetStdHandle, WriteFile, handle→fd mapping.
#include <windows.h>

int main(void) {
    HANDLE hOut = GetStdHandle(STD_OUTPUT_HANDLE);
    const char msg[] = "WriteFile works\n";
    DWORD written;
    WriteFile(hOut, msg, sizeof(msg) - 1, &written, NULL);
    return 0;
}
