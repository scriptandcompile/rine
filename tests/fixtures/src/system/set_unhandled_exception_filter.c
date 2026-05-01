// C integration fixture for SetUnhandledExceptionFilter.
// Installs a top-level filter, triggers a real access violation, and writes
// a marker from inside the filter callback.

#include <windows.h>

static LONG WINAPI test_filter(struct _EXCEPTION_POINTERS* exception_info) {
    (void)exception_info;
    const char marker[] = "seh_filter_called: ok\n";
    DWORD written = 0;
    HANDLE out = GetStdHandle(STD_OUTPUT_HANDLE);
    WriteFile(out, marker, (DWORD)(sizeof(marker) - 1), &written, NULL);
    return EXCEPTION_EXECUTE_HANDLER;
}

int main(void) {
    SetUnhandledExceptionFilter(test_filter);

    *(volatile int*)0 = 1;

    // We should never reach this line.
    return 3;
}
