use crate::common::assert_run;

#[test]
fn test_window_basic() {
    assert_run(
        "window_basic",
        0,
        "PASS: RegisterClassExA\n\
         PASS: CreateWindowExA\n\
         PASS: DestroyWindow\n\
         PASS: UnregisterClassA\n\
         All window basic tests passed",
    );
}

#[test]
fn test_window_messages() {
    assert_run(
        "window_messages",
        0,
        "PASS: PostMessageA\n\
         PASS: PeekMessageA (PM_NOREMOVE)\n\
         PASS: PeekMessageA (PM_REMOVE)\n\
         PASS: Queue empty after removal\n\
         PASS: PostQuitMessage\n\
         PASS: WM_QUIT received with correct exit code\n\
         All message tests passed",
    );
}

#[test]
fn test_window_text() {
    assert_run(
        "window_text",
        0,
        "PASS: CreateWindowExA\n\
         PASS: GetWindowTextLengthA (initial)\n\
         PASS: GetWindowTextA (initial)\n\
         PASS: SetWindowTextA\n\
         PASS: GetWindowTextLengthA (after set)\n\
         PASS: GetWindowTextA (after set)\n\
         PASS: GetWindowTextA (buffer truncation)\n\
         All window text tests passed",
    );
}

#[test]
fn test_window_show() {
    assert_run(
        "window_show",
        0,
        "PASS: CreateWindowExA\n\
         PASS: ShowWindow with SW_SHOW (was not visible)\n\
         PASS: ShowWindow with SW_HIDE (was visible)\n\
         PASS: ShowWindow with SW_HIDE (already hidden)\n\
         PASS: UpdateWindow\n\
         All window show tests passed",
    );
}
