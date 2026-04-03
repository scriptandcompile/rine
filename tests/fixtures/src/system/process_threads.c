// Tests process-related APIs.
// Tests: GetCurrentProcessId, GetCurrentProcess, CreateThread,
//        WaitForSingleObject, WaitForMultipleObjects, GetExitCodeThread,
//        Sleep, thread parameter passing.
#include <windows.h>
#include "rine_test.h"

// ── Thread entry: return its parameter as exit code ─────────────

static DWORD WINAPI echo_thread(LPVOID param) {
    return (DWORD)(ULONG_PTR)param;
}

// ── Thread entry: compute fibonacci ─────────────────────────────

static DWORD WINAPI fib_thread(LPVOID param) {
    int n = (int)(ULONG_PTR)param;
    int a = 0, b = 1;
    for (int i = 0; i < n; i++) {
        int t = a + b;
        a = b;
        b = t;
    }
    return (DWORD)a;
}

// ── Test: GetCurrentProcessId returns non-zero ──────────────────

static int test_current_pid(void) {
    DWORD pid = GetCurrentProcessId();
    return (pid != 0) ? 0 : 1;
}

// ── Test: GetCurrentProcess returns pseudo-handle (-1) ──────────

static int test_current_process(void) {
    HANDLE h = GetCurrentProcess();
    // Windows pseudo-handle is (HANDLE)-1.
    return (h == (HANDLE)(LONG_PTR)-1) ? 0 : 1;
}

// ── Test: simple thread create/join with exit code ──────────────

static int test_thread_exit_code(void) {
    HANDLE th = CreateThread(NULL, 0, echo_thread, (LPVOID)42, 0, NULL);
    if (th == NULL) return 1;

    DWORD r = WaitForSingleObject(th, 5000);
    if (r != WAIT_OBJECT_0) return 1;

    DWORD code = 0;
    if (!GetExitCodeThread(th, &code)) return 1;
    if (code != 42) return 1;

    CloseHandle(th);
    return 0;
}

// ── Test: thread parameter passing ──────────────────────────────

static int test_thread_param(void) {
    // Pass several different values and verify exit codes.
    DWORD values[] = {0, 1, 100, 255};
    for (int i = 0; i < 4; i++) {
        HANDLE th = CreateThread(NULL, 0, echo_thread, (LPVOID)(ULONG_PTR)values[i], 0, NULL);
        if (th == NULL) return 1;
        WaitForSingleObject(th, 5000);
        DWORD code = 0;
        GetExitCodeThread(th, &code);
        if (code != values[i]) return 1;
        CloseHandle(th);
    }
    return 0;
}

// ── Test: WaitForMultipleObjects ────────────────────────────────

#define MULTI_THREADS 4

static int test_wait_multiple(void) {
    HANDLE threads[MULTI_THREADS];
    for (int i = 0; i < MULTI_THREADS; i++) {
        threads[i] = CreateThread(NULL, 0, fib_thread, (LPVOID)(ULONG_PTR)(i + 5), 0, NULL);
        if (threads[i] == NULL) return 1;
    }

    // Wait for ALL threads.
    DWORD r = WaitForMultipleObjects(MULTI_THREADS, threads, TRUE, 5000);
    if (r != WAIT_OBJECT_0) return 1;

    // Verify exit codes: fib(5)=5, fib(6)=8, fib(7)=13, fib(8)=21.
    DWORD expected[] = {5, 8, 13, 21};
    for (int i = 0; i < MULTI_THREADS; i++) {
        DWORD code = 0;
        GetExitCodeThread(threads[i], &code);
        if (code != expected[i]) return 1;
        CloseHandle(threads[i]);
    }
    return 0;
}

// ── Test: WaitForSingleObject timeout ───────────────────────────

static DWORD WINAPI slow_thread(LPVOID param) {
    (void)param;
    Sleep(200);
    return 0;
}

static int test_wait_timeout(void) {
    HANDLE th = CreateThread(NULL, 0, slow_thread, NULL, 0, NULL);
    if (th == NULL) return 1;

    // Short timeout should return WAIT_TIMEOUT while thread is sleeping.
    DWORD r = WaitForSingleObject(th, 10);
    if (r != WAIT_TIMEOUT) return 1;

    // Eventually the thread finishes.
    r = WaitForSingleObject(th, 5000);
    if (r != WAIT_OBJECT_0) return 1;

    CloseHandle(th);
    return 0;
}

// ── Test: Sleep yields and returns ──────────────────────────────

static int test_sleep(void) {
    // Just verify Sleep(0) and a small Sleep don't hang.
    Sleep(0);
    Sleep(1);
    return 0;
}

// ── main: run all tests ─────────────────────────────────────────

int main(void) {
    int failures = 0;

    if (test_current_pid() == 0) {
        puts("pid: ok");
    } else {
        puts("pid: FAIL");
        failures++;
    }

    if (test_current_process() == 0) {
        puts("pseudo_handle: ok");
    } else {
        puts("pseudo_handle: FAIL");
        failures++;
    }

    if (test_thread_exit_code() == 0) {
        puts("thread_exit: ok");
    } else {
        puts("thread_exit: FAIL");
        failures++;
    }

    if (test_thread_param() == 0) {
        puts("thread_param: ok");
    } else {
        puts("thread_param: FAIL");
        failures++;
    }

    if (test_wait_multiple() == 0) {
        puts("wait_multiple: ok");
    } else {
        puts("wait_multiple: FAIL");
        failures++;
    }

    if (test_wait_timeout() == 0) {
        puts("wait_timeout: ok");
    } else {
        puts("wait_timeout: FAIL");
        failures++;
    }

    if (test_sleep() == 0) {
        puts("sleep: ok");
    } else {
        puts("sleep: FAIL");
        failures++;
    }

    return failures;
}
