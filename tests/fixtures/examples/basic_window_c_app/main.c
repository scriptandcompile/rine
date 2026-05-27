#include <windows.h>
#include <windowsx.h>
#include <stdio.h>

#define IDC_CLICK_BUTTON 1001
#define IDC_STATUS_TEXT 1002

static HWND g_status = NULL;
static HFONT g_ui_font = NULL;
static int g_click_count = 0;

static void update_status_text(void) {
    wchar_t buffer[128];
    swprintf(buffer, 128, L"Button clicks: %d", g_click_count);
    SetWindowTextW(g_status, buffer);
}

static LRESULT CALLBACK window_proc(HWND hwnd, UINT msg, WPARAM wparam, LPARAM lparam) {
    switch (msg) {
    case WM_CREATE: {
        HINSTANCE instance = ((LPCREATESTRUCTW)lparam)->hInstance;

        g_status = CreateWindowExW(
            0,
            L"STATIC",
            L"Button clicks: 0",
            WS_CHILD | WS_VISIBLE,
            20,
            20,
            260,
            24,
            hwnd,
            (HMENU)(INT_PTR)IDC_STATUS_TEXT,
            instance,
            NULL);

        HWND button = CreateWindowExW(
            0,
            L"BUTTON",
            L"Click me",
            WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_PUSHBUTTON,
            20,
            60,
            120,
            30,
            hwnd,
            (HMENU)(INT_PTR)IDC_CLICK_BUTTON,
            instance,
            NULL);

        g_ui_font = CreateFontW(
            18,
            0,
            0,
            0,
            FW_NORMAL,
            FALSE,
            FALSE,
            FALSE,
            DEFAULT_CHARSET,
            OUT_OUTLINE_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            VARIABLE_PITCH,
            L"Segoe UI");

        if (g_ui_font != NULL) {
            SendMessageW(g_status, WM_SETFONT, (WPARAM)g_ui_font, TRUE);
            SendMessageW(button, WM_SETFONT, (WPARAM)g_ui_font, TRUE);
        }

        return 0;
    }

    case WM_COMMAND: {
        if (LOWORD(wparam) == IDC_CLICK_BUTTON && HIWORD(wparam) == BN_CLICKED) {
            g_click_count++;
            update_status_text();

            if ((g_click_count % 5) == 0) {
                SetWindowTextW(hwnd, L"basic_window_c_app (milestone reached)");
            }

            return 0;
        }
        break;
    }

    case WM_KEYDOWN:
        if (wparam == VK_ESCAPE) {
            DestroyWindow(hwnd);
            return 0;
        }
        break;

    case WM_DESTROY:
        if (g_ui_font != NULL) {
            DeleteObject(g_ui_font);
            g_ui_font = NULL;
        }
        PostQuitMessage(0);
        return 0;
    }

    return DefWindowProcW(hwnd, msg, wparam, lparam);
}

int WINAPI wWinMain(HINSTANCE instance, HINSTANCE prev, PWSTR cmd_line, int cmd_show) {
    (void)prev;
    (void)cmd_line;

    const wchar_t *class_name = L"BasicWindowExampleClass";

    WNDCLASSW wc = {0};
    wc.lpfnWndProc = window_proc;
    wc.hInstance = instance;
    wc.lpszClassName = class_name;
    wc.hCursor = LoadCursorW(NULL, IDC_ARROW);
    wc.hbrBackground = (HBRUSH)(COLOR_WINDOW + 1);

    if (!RegisterClassW(&wc)) {
        MessageBoxW(NULL, L"RegisterClassW failed.", L"Error", MB_ICONERROR | MB_OK);
        return 1;
    }

    HWND hwnd = CreateWindowExW(
        0,
        class_name,
        L"basic_window_c_app",
        WS_OVERLAPPEDWINDOW,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        360,
        180,
        NULL,
        NULL,
        instance,
        NULL);

    if (hwnd == NULL) {
        MessageBoxW(NULL, L"CreateWindowExW failed.", L"Error", MB_ICONERROR | MB_OK);
        return 1;
    }

    ShowWindow(hwnd, cmd_show);
    UpdateWindow(hwnd);

    MSG msg;
    while (GetMessageW(&msg, NULL, 0, 0) > 0) {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }

    return (int)msg.wParam;
}
