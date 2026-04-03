// Test common-dialog small-buffer error path.
// Runs deterministically with RINE_DIALOG_TEST_PATH set by integration test.

#include <windows.h>
#include <commdlg.h>
#include <string.h>

#include "rine_test.h"

int main(void) {
    char tiny[2] = {0};
    OPENFILENAMEA ofn;
    memset(&ofn, 0, sizeof(ofn));
    ofn.lStructSize = sizeof(ofn);
    ofn.lpstrFile = tiny;
    ofn.nMaxFile = 2;
    ofn.lpstrTitle = "rine tiny buffer";

    BOOL ok = GetOpenFileNameA(&ofn);
    if (ok != FALSE) {
        puts("FAIL: GetOpenFileNameA should fail for tiny buffer");
        return 1;
    }
    puts("PASS: GetOpenFileNameA failed for tiny buffer");

    DWORD err = CommDlgExtendedError();
    if (err != 0x3003u) {
        puts("FAIL: CommDlgExtendedError should be FNERR_BUFFERTOOSMALL");
        put_kv_int("expected", 0x3003);
        put_kv_int("got", (int)err);
        return 1;
    }
    puts("PASS: CommDlgExtendedError is FNERR_BUFFERTOOSMALL");

    puts("All dialog small-buffer tests passed");
    return 0;
}
