// Test basic common-dialog behavior through comdlg32.
// This fixture is intended to run with RINE_DIALOG_MODE=emulated so behavior
// is deterministic and non-interactive.

#include <windows.h>
#include <commdlg.h>
#include <string.h>

#include "rine_test.h"

int main(void) {
    char file_buf_a[MAX_PATH] = {0};
    OPENFILENAMEA ofn_a;
    memset(&ofn_a, 0, sizeof(ofn_a));
    ofn_a.lStructSize = sizeof(ofn_a);
    ofn_a.lpstrFile = file_buf_a;
    ofn_a.nMaxFile = MAX_PATH;
    ofn_a.lpstrTitle = "rine dialog A";

    BOOL ok_a = GetOpenFileNameA(&ofn_a);
    if (ok_a != FALSE) {
        puts("FAIL: GetOpenFileNameA should fail in emulated mode");
        return 1;
    }
    puts("PASS: GetOpenFileNameA failed as expected");

    DWORD err_a = CommDlgExtendedError();
    if (err_a != 0xFFFFu) {
        puts("FAIL: CommDlgExtendedError A mismatch");
        put_kv_int("expected", 65535);
        put_kv_int("got", (int)err_a);
        return 1;
    }
    puts("PASS: CommDlgExtendedError A is CDERR_DIALOGFAILURE");

    WCHAR file_buf_w[MAX_PATH] = {0};
    OPENFILENAMEW ofn_w;
    memset(&ofn_w, 0, sizeof(ofn_w));
    ofn_w.lStructSize = sizeof(ofn_w);
    ofn_w.lpstrFile = file_buf_w;
    ofn_w.nMaxFile = MAX_PATH;
    ofn_w.lpstrTitle = L"rine dialog W";

    BOOL ok_w = GetSaveFileNameW(&ofn_w);
    if (ok_w != FALSE) {
        puts("FAIL: GetSaveFileNameW should fail in emulated mode");
        return 1;
    }
    puts("PASS: GetSaveFileNameW failed as expected");

    DWORD err_w = CommDlgExtendedError();
    if (err_w != 0xFFFFu) {
        puts("FAIL: CommDlgExtendedError W mismatch");
        put_kv_int("expected", 65535);
        put_kv_int("got", (int)err_w);
        return 1;
    }
    puts("PASS: CommDlgExtendedError W is CDERR_DIALOGFAILURE");

    puts("All dialog basic tests passed");
    return 0;
}
