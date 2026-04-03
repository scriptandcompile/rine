use crate::common::assert_run;

#[test]
fn test_gdi_objects() {
    assert_run(
        "gdi_objects",
        0,
        "PASS: CreateCompatibleDC\n\
         PASS: CreateCompatibleBitmap\n\
         PASS: SelectObject(bitmap)\n\
         PASS: DeleteObject(selected bitmap) fails\n\
         PASS: SelectObject(restore old bitmap)\n\
         PASS: DeleteObject(bitmap)\n\
         PASS: CreateSolidBrush\n\
         PASS: CreatePen\n\
         PASS: DeleteDC\n\
         PASS: DeleteObject(brush)\n\
         PASS: DeleteObject(pen)\n\
         All GDI object tests passed",
    );
}

#[test]
fn test_gdi_rendering() {
    assert_run(
        "gdi_rendering",
        0,
        "PASS: CreateCompatibleDC\n\
         PASS: CreateCompatibleBitmap\n\
         PASS: SelectObject(bitmap)\n\
         PASS: TextOutA\n\
         PASS: TextOutW\n\
         PASS: BitBlt(SRCCOPY)\n\
         PASS: BitBlt(BLACKNESS) fails\n\
         PASS: DeleteDC\n\
         All GDI rendering tests passed",
    );
}
