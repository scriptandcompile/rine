// Test GDI rendering pathways.
// Exercises TextOutA/W and BitBlt with SRCCOPY.

#include <windows.h>
#include "rine_test.h"

int main(void) {
    HDC src = CreateCompatibleDC(NULL);
    HDC dst = CreateCompatibleDC(NULL);
    if (!src || !dst) {
        puts("FAIL: CreateCompatibleDC");
        return 1;
    }
    puts("PASS: CreateCompatibleDC");

    HBITMAP src_bmp = CreateCompatibleBitmap(src, 64, 64);
    HBITMAP dst_bmp = CreateCompatibleBitmap(dst, 64, 64);
    if (!src_bmp || !dst_bmp) {
        puts("FAIL: CreateCompatibleBitmap");
        return 1;
    }
    puts("PASS: CreateCompatibleBitmap");

    if (!SelectObject(src, src_bmp) || !SelectObject(dst, dst_bmp)) {
        puts("FAIL: SelectObject(bitmap)");
        return 1;
    }
    puts("PASS: SelectObject(bitmap)");

    const char *text_a = "Hello";
    if (!TextOutA(src, 2, 2, text_a, 5)) {
        puts("FAIL: TextOutA");
        return 1;
    }
    puts("PASS: TextOutA");

    static const WCHAR text_w[] = L"Hi";
    if (!TextOutW(src, 4, 18, text_w, 2)) {
        puts("FAIL: TextOutW");
        return 1;
    }
    puts("PASS: TextOutW");

    if (!BitBlt(dst, 0, 0, 64, 64, src, 0, 0, SRCCOPY)) {
        puts("FAIL: BitBlt(SRCCOPY)");
        return 1;
    }
    puts("PASS: BitBlt(SRCCOPY)");

    // Non-SRCCOPY ROP is currently unsupported in rine GDI and should fail.
    if (BitBlt(dst, 0, 0, 64, 64, src, 0, 0, BLACKNESS) != 0) {
        puts("FAIL: BitBlt(BLACKNESS) should fail");
        return 1;
    }
    puts("PASS: BitBlt(BLACKNESS) fails");

    if (!DeleteDC(src) || !DeleteDC(dst)) {
        puts("FAIL: DeleteDC");
        return 1;
    }
    puts("PASS: DeleteDC");

    puts("All GDI rendering tests passed");
    return 0;
}
