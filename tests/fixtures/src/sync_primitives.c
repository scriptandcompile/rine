// Tests synchronization primitives: critical sections, events, mutexes, semaphores.
// Tests: InitializeCriticalSection, EnterCriticalSection, LeaveCriticalSection,
//        DeleteCriticalSection, CreateEvent, SetEvent, ResetEvent,
//        WaitForSingleObject, CreateMutex, ReleaseMutex,
//        CreateSemaphore, ReleaseSemaphore, CreateThread.
#include <windows.h>
#include "rine_test.h"

// ── Shared state protected by critical section ──────────────────

static CRITICAL_SECTION g_cs;
static volatile int g_counter = 0;

#define CS_THREADS 4
#define CS_INCREMENTS 1000

static DWORD WINAPI cs_worker(LPVOID param) {
    (void)param;
    for (int i = 0; i < CS_INCREMENTS; i++) {
        EnterCriticalSection(&g_cs);
        g_counter++;
        LeaveCriticalSection(&g_cs);
    }
    return 0;
}

static int test_critical_section(void) {
    InitializeCriticalSection(&g_cs);
    g_counter = 0;

    HANDLE threads[CS_THREADS];
    for (int i = 0; i < CS_THREADS; i++) {
        threads[i] = CreateThread(NULL, 0, cs_worker, NULL, 0, NULL);
        if (threads[i] == NULL) return 1;
    }

    WaitForMultipleObjects(CS_THREADS, threads, TRUE, INFINITE);
    for (int i = 0; i < CS_THREADS; i++) CloseHandle(threads[i]);

    DeleteCriticalSection(&g_cs);

    int expected = CS_THREADS * CS_INCREMENTS;
    if (g_counter != expected) return 1;
    return 0;
}

// ── Event test: one producer, one consumer ──────────────────────

static HANDLE g_event;
static volatile int g_event_value = 0;

static DWORD WINAPI event_producer(LPVOID param) {
    (void)param;
    g_event_value = 99;
    SetEvent(g_event);
    return 0;
}

static int test_events(void) {
    // Manual-reset event, initially unsignaled.
    g_event = CreateEventA(NULL, TRUE, FALSE, NULL);
    if (g_event == NULL) return 1;

    g_event_value = 0;

    HANDLE th = CreateThread(NULL, 0, event_producer, NULL, 0, NULL);
    if (th == NULL) return 1;

    // Wait for producer to signal.
    DWORD r = WaitForSingleObject(g_event, 5000);
    if (r != WAIT_OBJECT_0) return 1;

    // Value should be set by producer.
    if (g_event_value != 99) return 1;

    // Manual-reset: event stays signaled.
    r = WaitForSingleObject(g_event, 0);
    if (r != WAIT_OBJECT_0) return 1;

    // Reset and check it's now unsignaled.
    ResetEvent(g_event);
    r = WaitForSingleObject(g_event, 0);
    if (r != WAIT_TIMEOUT) return 1;

    WaitForSingleObject(th, INFINITE);
    CloseHandle(th);
    CloseHandle(g_event);
    return 0;
}

// ── Auto-reset event: only one waiter wakes ─────────────────────

static int test_auto_reset_event(void) {
    HANDLE ev = CreateEventA(NULL, FALSE, TRUE, NULL);
    if (ev == NULL) return 1;

    // Initially signaled: first wait succeeds and auto-resets.
    DWORD r = WaitForSingleObject(ev, 0);
    if (r != WAIT_OBJECT_0) return 1;

    // Should be unsignaled now.
    r = WaitForSingleObject(ev, 0);
    if (r != WAIT_TIMEOUT) return 1;

    CloseHandle(ev);
    return 0;
}

// ── Mutex test: mutual exclusion across threads ─────────────────

static HANDLE g_mutex;
static volatile int g_mutex_counter = 0;

#define MUTEX_THREADS 4
#define MUTEX_INCREMENTS 500

static DWORD WINAPI mutex_worker(LPVOID param) {
    (void)param;
    for (int i = 0; i < MUTEX_INCREMENTS; i++) {
        WaitForSingleObject(g_mutex, INFINITE);
        g_mutex_counter++;
        ReleaseMutex(g_mutex);
    }
    return 0;
}

static int test_mutex(void) {
    g_mutex = CreateMutexA(NULL, FALSE, NULL);
    if (g_mutex == NULL) return 1;

    g_mutex_counter = 0;

    HANDLE threads[MUTEX_THREADS];
    for (int i = 0; i < MUTEX_THREADS; i++) {
        threads[i] = CreateThread(NULL, 0, mutex_worker, NULL, 0, NULL);
        if (threads[i] == NULL) return 1;
    }

    WaitForMultipleObjects(MUTEX_THREADS, threads, TRUE, INFINITE);
    for (int i = 0; i < MUTEX_THREADS; i++) CloseHandle(threads[i]);

    CloseHandle(g_mutex);

    int expected = MUTEX_THREADS * MUTEX_INCREMENTS;
    if (g_mutex_counter != expected) return 1;
    return 0;
}

// ── Mutex recursive ownership ───────────────────────────────────

static int test_mutex_recursive(void) {
    HANDLE m = CreateMutexA(NULL, TRUE, NULL);  // initially owned
    if (m == NULL) return 1;

    // Same thread can acquire recursively.
    DWORD r = WaitForSingleObject(m, 0);
    if (r != WAIT_OBJECT_0) return 1;

    // Release twice (once for each acquire).
    if (!ReleaseMutex(m)) return 1;
    if (!ReleaseMutex(m)) return 1;

    // Third release should fail (not owned).
    if (ReleaseMutex(m)) return 1;

    CloseHandle(m);
    return 0;
}

// ── Semaphore test: bounded concurrency ─────────────────────────

static HANDLE g_semaphore;
static volatile int g_sem_active = 0;
static volatile int g_sem_max_active = 0;
static CRITICAL_SECTION g_sem_cs;

#define SEM_THREADS 6
#define SEM_MAX_CONCURRENT 2

static DWORD WINAPI semaphore_worker(LPVOID param) {
    (void)param;
    WaitForSingleObject(g_semaphore, INFINITE);

    EnterCriticalSection(&g_sem_cs);
    g_sem_active++;
    if (g_sem_active > g_sem_max_active)
        g_sem_max_active = g_sem_active;
    LeaveCriticalSection(&g_sem_cs);

    // Simulate work.
    Sleep(10);

    EnterCriticalSection(&g_sem_cs);
    g_sem_active--;
    LeaveCriticalSection(&g_sem_cs);

    ReleaseSemaphore(g_semaphore, 1, NULL);
    return 0;
}

static int test_semaphore(void) {
    InitializeCriticalSection(&g_sem_cs);
    g_semaphore = CreateSemaphoreA(NULL, SEM_MAX_CONCURRENT, SEM_MAX_CONCURRENT, NULL);
    if (g_semaphore == NULL) return 1;

    g_sem_active = 0;
    g_sem_max_active = 0;

    HANDLE threads[SEM_THREADS];
    for (int i = 0; i < SEM_THREADS; i++) {
        threads[i] = CreateThread(NULL, 0, semaphore_worker, NULL, 0, NULL);
        if (threads[i] == NULL) return 1;
    }

    WaitForMultipleObjects(SEM_THREADS, threads, TRUE, INFINITE);
    for (int i = 0; i < SEM_THREADS; i++) CloseHandle(threads[i]);

    CloseHandle(g_semaphore);
    DeleteCriticalSection(&g_sem_cs);

    // Max concurrent should not exceed semaphore limit.
    if (g_sem_max_active > SEM_MAX_CONCURRENT) return 1;
    return 0;
}

// ── Semaphore release with previous-count output ────────────────

static int test_semaphore_release(void) {
    HANDLE s = CreateSemaphoreA(NULL, 0, 5, NULL);
    if (s == NULL) return 1;

    LONG prev = -1;
    if (!ReleaseSemaphore(s, 2, &prev)) return 1;
    if (prev != 0) return 1;  // was 0 before release

    prev = -1;
    if (!ReleaseSemaphore(s, 1, &prev)) return 1;
    if (prev != 2) return 1;  // was 2 before release

    // Should fail: 3 + 3 = 6 > max 5.
    if (ReleaseSemaphore(s, 3, NULL)) return 1;

    CloseHandle(s);
    return 0;
}

// ── main: run all tests ─────────────────────────────────────────

int main(void) {
    int failures = 0;

    if (test_critical_section() == 0) {
        puts("cs: ok");
    } else {
        puts("cs: FAIL");
        failures++;
    }

    if (test_events() == 0) {
        puts("events: ok");
    } else {
        puts("events: FAIL");
        failures++;
    }

    if (test_auto_reset_event() == 0) {
        puts("auto_reset: ok");
    } else {
        puts("auto_reset: FAIL");
        failures++;
    }

    if (test_mutex() == 0) {
        puts("mutex: ok");
    } else {
        puts("mutex: FAIL");
        failures++;
    }

    if (test_mutex_recursive() == 0) {
        puts("mutex_recursive: ok");
    } else {
        puts("mutex_recursive: FAIL");
        failures++;
    }

    if (test_semaphore() == 0) {
        puts("semaphore: ok");
    } else {
        puts("semaphore: FAIL");
        failures++;
    }

    if (test_semaphore_release() == 0) {
        puts("sem_release: ok");
    } else {
        puts("sem_release: FAIL");
        failures++;
    }

    return failures;
}
