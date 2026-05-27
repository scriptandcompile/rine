#include <windows.h>
#include <stdio.h>
#include <wctype.h>

#define IDD_MAIN_DIALOG 2001
#define IDC_NAME_EDIT 2002
#define IDC_UPPER_CHECK 2003

static void uppercase_in_place(wchar_t *text) {
    while (*text != L'\0') {
        *text = towupper(*text);
        text++;
    }
}

static INT_PTR CALLBACK dialog_proc(HWND dialog, UINT msg, WPARAM wparam, LPARAM lparam) {
    (void)lparam;

    switch (msg) {
    case WM_INITDIALOG:
        SetDlgItemTextW(dialog, IDC_NAME_EDIT, L"world");
        CheckDlgButton(dialog, IDC_UPPER_CHECK, BST_UNCHECKED);
        return (INT_PTR)TRUE;

    case WM_COMMAND: {
        const WORD id = LOWORD(wparam);

        if (id == IDOK) {
            wchar_t name[128];
            wchar_t output[256];

            GetDlgItemTextW(dialog, IDC_NAME_EDIT, name, 128);
            if (name[0] == L'\0') {
                wcscpy(name, L"world");
            }

            if (IsDlgButtonChecked(dialog, IDC_UPPER_CHECK) == BST_CHECKED) {
                uppercase_in_place(name);
            }

            swprintf(output, 256, L"Hello, %ls", name);
            MessageBoxW(dialog, output, L"Greeting", MB_OK | MB_ICONINFORMATION);
            EndDialog(dialog, IDOK);
            return (INT_PTR)TRUE;
        }

        if (id == IDCANCEL) {
            EndDialog(dialog, IDCANCEL);
            return (INT_PTR)TRUE;
        }

        break;
    }
    }

    return (INT_PTR)FALSE;
}

int WINAPI wWinMain(HINSTANCE instance, HINSTANCE prev, PWSTR cmd_line, int cmd_show) {
    (void)prev;
    (void)cmd_line;
    (void)cmd_show;

    const INT_PTR result = DialogBoxParamW(
        instance,
        MAKEINTRESOURCEW(IDD_MAIN_DIALOG),
        NULL,
        dialog_proc,
        0);

    if (result == -1) {
        MessageBoxW(NULL, L"DialogBoxParamW failed.", L"Error", MB_ICONERROR | MB_OK);
        return 1;
    }

    return 0;
}
