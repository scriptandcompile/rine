// Tests GetExitCodeProcess failure paths set GetLastError.
#include <windows.h>
#include "rine_test.h"

// ── Test: NULL exit_code pointer should set ERROR_INVALID_PARAMETER ───────

static int test_null_out_ptr_sets_invalid_parameter(void) {
    SetLastError(0);

    if (GetExitCodeProcess((HANDLE)0x9999, NULL) != FALSE) return 1;
    if (GetLastError() != ERROR_INVALID_PARAMETER) return 1;

    return 0;
}

// ── Test: invalid handle should set ERROR_INVALID_HANDLE ───────────────────

static int test_invalid_handle_sets_invalid_handle(void) {
    DWORD code = 0;
    SetLastError(0);

    if (GetExitCodeProcess((HANDLE)0x9999, &code) != FALSE) return 1;
    if (GetLastError() != ERROR_INVALID_HANDLE) return 1;

    return 0;
}

int main(void) {
    puts(test_null_out_ptr_sets_invalid_parameter() == 0
             ? "get_exit_code_process_null_out_error: ok"
             : "FAIL: get_exit_code_process_null_out_error");
    puts(test_invalid_handle_sets_invalid_handle() == 0
             ? "get_exit_code_process_invalid_handle_error: ok"
             : "FAIL: get_exit_code_process_invalid_handle_error");
    return 0;
}
