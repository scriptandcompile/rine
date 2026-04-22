// Tests kernel32 last-error functions.
// Tests: GetLastError, SetLastError, and thread-local isolation.
#include <windows.h>
#include "rine_test.h"

// ── Thread entry: return the thread's current last-error code ─────────────

static DWORD WINAPI read_last_error_thread(LPVOID param) {
    (void)param;
    return GetLastError();
}

// ── Test: SetLastError + GetLastError round-trip ──────────────────────────

static int test_roundtrip(void) {
    SetLastError(12345u);
    return GetLastError() == 12345u ? 0 : 1;
}

// ── Test: latest SetLastError value wins ───────────────────────────────────

static int test_overwrite(void) {
    SetLastError(2u);
    SetLastError(87u);
    return GetLastError() == 87u ? 0 : 1;
}

// ── Test: error code is thread-local ───────────────────────────────────────

static int test_thread_local_isolation(void) {
    SetLastError(0xfaceu);

    HANDLE th = CreateThread(NULL, 0, read_last_error_thread, NULL, 0, NULL);
    if (th == NULL) return 1;

    DWORD wait = WaitForSingleObject(th, 5000);
    if (wait != WAIT_OBJECT_0) {
        CloseHandle(th);
        return 1;
    }

    DWORD child_last_error = 0xffffffffu;
    if (!GetExitCodeThread(th, &child_last_error)) {
        CloseHandle(th);
        return 1;
    }

    CloseHandle(th);

    // New thread should start with ERROR_SUCCESS (0), while parent keeps its value.
    if (child_last_error != 0u) return 1;
    if (GetLastError() != 0xfaceu) return 1;

    return 0;
}

// ── main ───────────────────────────────────────────────────────────────────

int main(void) {
    puts(test_roundtrip() == 0 ? "last_error_roundtrip: ok" : "FAIL: last_error_roundtrip");
    puts(test_overwrite() == 0 ? "last_error_overwrite: ok" : "FAIL: last_error_overwrite");
    puts(test_thread_local_isolation() == 0 ? "last_error_thread_local: ok" : "FAIL: last_error_thread_local");
    return 0;
}