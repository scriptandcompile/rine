// Test basic GDI object lifecycle operations.
// Exercises DC/bitmap/brush/pen creation and delete semantics.

#include <windows.h>
#include "rine_test.h"

int main(void) {
    HDC dc = CreateCompatibleDC(NULL);
    if (!dc) {
        puts("FAIL: CreateCompatibleDC");
        return 1;
    }
    puts("PASS: CreateCompatibleDC");

    HBITMAP bmp = CreateCompatibleBitmap(dc, 64, 32);
    if (!bmp) {
        puts("FAIL: CreateCompatibleBitmap");
        return 1;
    }
    puts("PASS: CreateCompatibleBitmap");

    HGDIOBJ old_bmp = SelectObject(dc, bmp);
    if (!old_bmp) {
        puts("FAIL: SelectObject(bitmap)");
        return 1;
    }
    puts("PASS: SelectObject(bitmap)");

    // Selected objects should not be deletable.
    if (DeleteObject(bmp) != 0) {
        puts("FAIL: DeleteObject(selected bitmap) should fail");
        return 1;
    }
    puts("PASS: DeleteObject(selected bitmap) fails");

    HGDIOBJ replaced = SelectObject(dc, old_bmp);
    if (replaced != bmp) {
        puts("FAIL: SelectObject(restore old bitmap)");
        return 1;
    }
    puts("PASS: SelectObject(restore old bitmap)");

    if (!DeleteObject(bmp)) {
        puts("FAIL: DeleteObject(bitmap)");
        return 1;
    }
    puts("PASS: DeleteObject(bitmap)");

    HBRUSH brush = CreateSolidBrush(RGB(0x11, 0x22, 0x33));
    if (!brush) {
        puts("FAIL: CreateSolidBrush");
        return 1;
    }
    puts("PASS: CreateSolidBrush");

    HPEN pen = CreatePen(PS_SOLID, 1, RGB(0x44, 0x55, 0x66));
    if (!pen) {
        puts("FAIL: CreatePen");
        return 1;
    }
    puts("PASS: CreatePen");

    if (!DeleteDC(dc)) {
        puts("FAIL: DeleteDC");
        return 1;
    }
    puts("PASS: DeleteDC");

    // After DC teardown, non-selected objects should be deletable.
    if (!DeleteObject(brush)) {
        puts("FAIL: DeleteObject(brush)");
        return 1;
    }
    puts("PASS: DeleteObject(brush)");

    if (!DeleteObject(pen)) {
        puts("FAIL: DeleteObject(pen)");
        return 1;
    }
    puts("PASS: DeleteObject(pen)");

    puts("All GDI object tests passed");
    return 0;
}
