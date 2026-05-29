#include <windows.h>
#include <commctrl.h>
#include <stdio.h>

#define IDC_BTN_PUSH 1001
#define IDC_BTN_DEFAULT 1002
#define IDC_CHK_OPTION 1003
#define IDC_RAD_ONE 1004
#define IDC_RAD_TWO 1005
#define IDC_EDIT_SINGLE 1006
#define IDC_EDIT_PASSWORD 1007
#define IDC_EDIT_MULTI 1008
#define IDC_LISTBOX 1009
#define IDC_COMBOBOX 1010
#define IDC_SCROLL_H 1011
#define IDC_SCROLL_V 1012
#define IDC_LISTVIEW 1013
#define IDC_TREEVIEW 1014
#define IDC_TAB 1015
#define IDC_PROGRESS 1016
#define IDC_TRACKBAR 1017
#define IDC_UPDOWN_EDIT 1018
#define IDC_UPDOWN 1019
#define IDC_DATETIME 1020
#define IDC_MONTHCAL 1021
#define IDC_HOTKEY 1022
#define IDC_HEADER 1023
#define IDC_STATUSBAR 1024

static HFONT g_ui_font = NULL;

static void apply_ui_font_to_children(HWND parent) {
    HWND child = GetWindow(parent, GW_CHILD);
    while (child != NULL) {
        SendMessageW(child, WM_SETFONT, (WPARAM)g_ui_font, TRUE);
        child = GetWindow(child, GW_HWNDNEXT);
    }
}

static void populate_list_box(HWND list_box) {
    SendMessageW(list_box, LB_ADDSTRING, 0, (LPARAM)L"List item 1");
    SendMessageW(list_box, LB_ADDSTRING, 0, (LPARAM)L"List item 2");
    SendMessageW(list_box, LB_ADDSTRING, 0, (LPARAM)L"List item 3");
    SendMessageW(list_box, LB_SETCURSEL, 0, 0);
}

static void populate_combo_box(HWND combo_box) {
    SendMessageW(combo_box, CB_ADDSTRING, 0, (LPARAM)L"Combo item A");
    SendMessageW(combo_box, CB_ADDSTRING, 0, (LPARAM)L"Combo item B");
    SendMessageW(combo_box, CB_ADDSTRING, 0, (LPARAM)L"Combo item C");
    SendMessageW(combo_box, CB_SETCURSEL, 0, 0);
}

static void populate_list_view(HWND list_view) {
    LVCOLUMNW column = {0};
    column.mask = LVCF_TEXT | LVCF_WIDTH;
    column.cx = 120;

    column.pszText = L"Name";
    ListView_InsertColumn(list_view, 0, &column);

    column.pszText = L"Type";
    ListView_InsertColumn(list_view, 1, &column);

    LVITEMW item = {0};
    item.mask = LVIF_TEXT;

    item.iItem = 0;
    item.iSubItem = 0;
    item.pszText = L"Alpha";
    ListView_InsertItem(list_view, &item);
    ListView_SetItemText(list_view, 0, 1, L"Example");

    item.iItem = 1;
    item.iSubItem = 0;
    item.pszText = L"Beta";
    ListView_InsertItem(list_view, &item);
    ListView_SetItemText(list_view, 1, 1, L"Fixture");
}

static void populate_tree_view(HWND tree_view) {
    TVINSERTSTRUCTW insert = {0};

    insert.hParent = TVI_ROOT;
    insert.hInsertAfter = TVI_LAST;
    insert.item.mask = TVIF_TEXT;
    insert.item.pszText = L"Root node";
    HTREEITEM root = TreeView_InsertItem(tree_view, &insert);

    insert.hParent = root;
    insert.item.pszText = L"Child node 1";
    TreeView_InsertItem(tree_view, &insert);

    insert.item.pszText = L"Child node 2";
    TreeView_InsertItem(tree_view, &insert);

    TreeView_Expand(tree_view, root, TVE_EXPAND);
}

static void populate_tab_control(HWND tab_control) {
    TCITEMW item = {0};
    item.mask = TCIF_TEXT;

    item.pszText = L"Tab One";
    TabCtrl_InsertItem(tab_control, 0, &item);

    item.pszText = L"Tab Two";
    TabCtrl_InsertItem(tab_control, 1, &item);
}

static void populate_header(HWND header) {
    HDITEMW item = {0};
    item.mask = HDI_TEXT | HDI_WIDTH | HDI_FORMAT;
    item.fmt = HDF_LEFT | HDF_STRING;

    item.pszText = L"Header A";
    item.cxy = 110;
    Header_InsertItem(header, 0, &item);

    item.pszText = L"Header B";
    item.cxy = 110;
    Header_InsertItem(header, 1, &item);
}

static void set_status_text(HWND status_bar, const wchar_t *text) {
    SendMessageW(status_bar, SB_SETTEXTW, 0, (LPARAM)text);
}

static void create_controls(HWND hwnd, HINSTANCE instance) {
    CreateWindowExW(0, L"STATIC", L"Classic controls", WS_CHILD | WS_VISIBLE,
                    16, 14, 140, 20, hwnd, NULL, instance, NULL);

    CreateWindowExW(0, L"BUTTON", L"Button group", WS_CHILD | WS_VISIBLE | BS_GROUPBOX,
                    12, 36, 232, 124, hwnd, NULL, instance, NULL);

    CreateWindowExW(0, L"BUTTON", L"Push Button", WS_TABSTOP | WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
                    24, 58, 96, 26, hwnd, (HMENU)(INT_PTR)IDC_BTN_PUSH, instance, NULL);

    CreateWindowExW(0, L"BUTTON", L"Default Button", WS_TABSTOP | WS_CHILD | WS_VISIBLE | BS_DEFPUSHBUTTON,
                    132, 58, 96, 26, hwnd, (HMENU)(INT_PTR)IDC_BTN_DEFAULT, instance, NULL);

    CreateWindowExW(0, L"BUTTON", L"Checkbox", WS_TABSTOP | WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX,
                    24, 92, 88, 20, hwnd, (HMENU)(INT_PTR)IDC_CHK_OPTION, instance, NULL);

    CreateWindowExW(0, L"BUTTON", L"Radio One", WS_TABSTOP | WS_CHILD | WS_VISIBLE | BS_AUTORADIOBUTTON,
                    24, 116, 88, 20, hwnd, (HMENU)(INT_PTR)IDC_RAD_ONE, instance, NULL);

    CreateWindowExW(0, L"BUTTON", L"Radio Two", WS_TABSTOP | WS_CHILD | WS_VISIBLE | BS_AUTORADIOBUTTON,
                    132, 116, 88, 20, hwnd, (HMENU)(INT_PTR)IDC_RAD_TWO, instance, NULL);

    CreateWindowExW(0, L"STATIC", L"Edit controls", WS_CHILD | WS_VISIBLE,
                    264, 14, 120, 20, hwnd, NULL, instance, NULL);

    CreateWindowExW(WS_EX_CLIENTEDGE, L"EDIT", L"Single-line text",
                    WS_TABSTOP | WS_CHILD | WS_VISIBLE | ES_AUTOHSCROLL,
                    264, 38, 250, 24, hwnd, (HMENU)(INT_PTR)IDC_EDIT_SINGLE, instance, NULL);

    CreateWindowExW(WS_EX_CLIENTEDGE, L"EDIT", L"secret",
                    WS_TABSTOP | WS_CHILD | WS_VISIBLE | ES_AUTOHSCROLL | ES_PASSWORD,
                    264, 70, 250, 24, hwnd, (HMENU)(INT_PTR)IDC_EDIT_PASSWORD, instance, NULL);

    CreateWindowExW(WS_EX_CLIENTEDGE, L"EDIT", L"Multiline\r\nline 2",
                    WS_TABSTOP | WS_CHILD | WS_VISIBLE | ES_LEFT | ES_MULTILINE | ES_AUTOVSCROLL | WS_VSCROLL,
                    264, 102, 250, 58, hwnd, (HMENU)(INT_PTR)IDC_EDIT_MULTI, instance, NULL);

    CreateWindowExW(0, L"STATIC", L"List and selection controls", WS_CHILD | WS_VISIBLE,
                    16, 168, 200, 20, hwnd, NULL, instance, NULL);

    HWND list_box = CreateWindowExW(WS_EX_CLIENTEDGE, L"LISTBOX", NULL,
                                    WS_TABSTOP | WS_CHILD | WS_VISIBLE | LBS_NOTIFY | WS_VSCROLL,
                                    16, 192, 150, 96, hwnd, (HMENU)(INT_PTR)IDC_LISTBOX, instance, NULL);

    HWND combo_box = CreateWindowExW(0, L"COMBOBOX", NULL,
                                     WS_TABSTOP | WS_CHILD | WS_VISIBLE | CBS_DROPDOWNLIST | WS_VSCROLL,
                                     176, 192, 170, 160, hwnd, (HMENU)(INT_PTR)IDC_COMBOBOX, instance, NULL);

    HWND scroll_h = CreateWindowExW(0, L"SCROLLBAR", NULL,
                                    WS_CHILD | WS_VISIBLE | SBS_HORZ,
                                    16, 296, 330, 18, hwnd, (HMENU)(INT_PTR)IDC_SCROLL_H, instance, NULL);

    HWND scroll_v = CreateWindowExW(0, L"SCROLLBAR", NULL,
                                    WS_CHILD | WS_VISIBLE | SBS_VERT,
                                    352, 192, 18, 122, hwnd, (HMENU)(INT_PTR)IDC_SCROLL_V, instance, NULL);

    SetScrollRange(scroll_h, SB_CTL, 0, 100, TRUE);
    SetScrollPos(scroll_h, SB_CTL, 35, TRUE);
    SetScrollRange(scroll_v, SB_CTL, 0, 100, TRUE);
    SetScrollPos(scroll_v, SB_CTL, 55, TRUE);

    CreateWindowExW(0, L"STATIC", L"Common controls", WS_CHILD | WS_VISIBLE,
                    386, 168, 160, 20, hwnd, NULL, instance, NULL);

    HWND list_view = CreateWindowExW(WS_EX_CLIENTEDGE, WC_LISTVIEWW, NULL,
                                     WS_TABSTOP | WS_CHILD | WS_VISIBLE | LVS_REPORT,
                                     386, 192, 260, 112, hwnd, (HMENU)(INT_PTR)IDC_LISTVIEW, instance, NULL);

    HWND tree_view = CreateWindowExW(WS_EX_CLIENTEDGE, WC_TREEVIEWW, NULL,
                                     WS_TABSTOP | WS_CHILD | WS_VISIBLE | TVS_HASLINES | TVS_LINESATROOT,
                                     656, 192, 220, 112, hwnd, (HMENU)(INT_PTR)IDC_TREEVIEW, instance, NULL);

    HWND tab_control = CreateWindowExW(0, WC_TABCONTROLW, NULL,
                                       WS_TABSTOP | WS_CHILD | WS_VISIBLE,
                                       386, 312, 250, 76, hwnd, (HMENU)(INT_PTR)IDC_TAB, instance, NULL);

    HWND progress = CreateWindowExW(0, PROGRESS_CLASSW, NULL,
                                    WS_CHILD | WS_VISIBLE,
                                    386, 396, 250, 20, hwnd, (HMENU)(INT_PTR)IDC_PROGRESS, instance, NULL);

    HWND trackbar = CreateWindowExW(0, TRACKBAR_CLASSW, NULL,
                                    WS_TABSTOP | WS_CHILD | WS_VISIBLE | TBS_AUTOTICKS,
                                    386, 424, 250, 36, hwnd, (HMENU)(INT_PTR)IDC_TRACKBAR, instance, NULL);

    HWND updown_edit = CreateWindowExW(WS_EX_CLIENTEDGE, L"EDIT", L"5",
                                       WS_TABSTOP | WS_CHILD | WS_VISIBLE | ES_NUMBER,
                                       656, 314, 58, 24, hwnd, (HMENU)(INT_PTR)IDC_UPDOWN_EDIT, instance, NULL);

    HWND updown = CreateWindowExW(0, UPDOWN_CLASSW, NULL,
                                  WS_CHILD | WS_VISIBLE | UDS_ALIGNRIGHT | UDS_SETBUDDYINT | UDS_ARROWKEYS,
                                  0, 0, 0, 0, hwnd, (HMENU)(INT_PTR)IDC_UPDOWN, instance, NULL);

    HWND datetime = CreateWindowExW(0, DATETIMEPICK_CLASSW, NULL,
                                    WS_TABSTOP | WS_CHILD | WS_VISIBLE,
                                    726, 314, 150, 24, hwnd, (HMENU)(INT_PTR)IDC_DATETIME, instance, NULL);

    HWND monthcal = CreateWindowExW(0, MONTHCAL_CLASSW, NULL,
                                    WS_TABSTOP | WS_CHILD | WS_VISIBLE,
                                    656, 346, 220, 160, hwnd, (HMENU)(INT_PTR)IDC_MONTHCAL, instance, NULL);

    HWND hotkey = CreateWindowExW(WS_EX_CLIENTEDGE, HOTKEY_CLASSW, NULL,
                                  WS_TABSTOP | WS_CHILD | WS_VISIBLE,
                                  386, 468, 140, 24, hwnd, (HMENU)(INT_PTR)IDC_HOTKEY, instance, NULL);

    HWND header = CreateWindowExW(0, WC_HEADERW, NULL,
                                  WS_CHILD | WS_VISIBLE | HDS_BUTTONS,
                                  386, 500, 230, 24, hwnd, (HMENU)(INT_PTR)IDC_HEADER, instance, NULL);

    HWND status_bar = CreateWindowExW(0, STATUSCLASSNAMEW, NULL,
                                      WS_CHILD | WS_VISIBLE | SBARS_SIZEGRIP,
                                      0, 0, 0, 0, hwnd, (HMENU)(INT_PTR)IDC_STATUSBAR, instance, NULL);

    populate_list_box(list_box);
    populate_combo_box(combo_box);
    populate_list_view(list_view);
    populate_tree_view(tree_view);
    populate_tab_control(tab_control);
    populate_header(header);

    SendMessageW(progress, PBM_SETRANGE, 0, MAKELPARAM(0, 100));
    SendMessageW(progress, PBM_SETPOS, 66, 0);

    SendMessageW(trackbar, TBM_SETRANGE, TRUE, MAKELPARAM(0, 100));
    SendMessageW(trackbar, TBM_SETTICFREQ, 10, 0);
    SendMessageW(trackbar, TBM_SETPOS, TRUE, 44);

    SendMessageW(updown, UDM_SETBUDDY, (WPARAM)updown_edit, 0);
    SendMessageW(updown, UDM_SETRANGE32, 0, 100);
    SendMessageW(updown, UDM_SETPOS32, 0, 5);

    SendMessageW(hotkey, HKM_SETHOTKEY, MAKEWORD('H', HOTKEYF_CONTROL | HOTKEYF_ALT), 0);

    int status_parts[3] = {300, 620, -1};
    SendMessageW(status_bar, SB_SETPARTS, 3, (LPARAM)status_parts);
    SendMessageW(status_bar, SB_SETTEXTW, 1, (LPARAM)L"all_controls_c_app fixture");
    set_status_text(status_bar, L"Ready");
}

static HMENU create_main_menu(void) {
    HMENU menu_bar = CreateMenu();
    HMENU menu_file = CreateMenu();
    HMENU menu_help = CreateMenu();

    AppendMenuW(menu_file, MF_STRING, 2001, L"&Reset status");
    AppendMenuW(menu_file, MF_SEPARATOR, 0, NULL);
    AppendMenuW(menu_file, MF_STRING, 2002, L"E&xit");

    AppendMenuW(menu_help, MF_STRING, 2003, L"&About");

    AppendMenuW(menu_bar, MF_POPUP, (UINT_PTR)menu_file, L"&File");
    AppendMenuW(menu_bar, MF_POPUP, (UINT_PTR)menu_help, L"&Help");
    return menu_bar;
}

static LRESULT CALLBACK window_proc(HWND hwnd, UINT msg, WPARAM wparam, LPARAM lparam) {
    switch (msg) {
    case WM_CREATE: {
        HINSTANCE instance = ((LPCREATESTRUCTW)lparam)->hInstance;
        SetMenu(hwnd, create_main_menu());
        create_controls(hwnd, instance);

        g_ui_font = CreateFontW(
            17,
            0,
            0,
            0,
            FW_NORMAL,
            FALSE,
            FALSE,
            FALSE,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            VARIABLE_PITCH,
            L"Segoe UI");

        if (g_ui_font != NULL) {
            apply_ui_font_to_children(hwnd);
        }

        return 0;
    }

    case WM_COMMAND: {
        const WORD id = LOWORD(wparam);

        if (id == 2001) {
            HWND status_bar = GetDlgItem(hwnd, IDC_STATUSBAR);
            if (status_bar != NULL) {
                set_status_text(status_bar, L"Status reset");
            }
            return 0;
        }

        if (id == 2002) {
            DestroyWindow(hwnd);
            return 0;
        }

        if (id == 2003) {
            MessageBoxW(hwnd,
                        L"all_controls_c_app\nFixture with broad Win32 control coverage.",
                        L"About",
                        MB_OK | MB_ICONINFORMATION);
            return 0;
        }

        if ((id == IDC_BTN_PUSH || id == IDC_BTN_DEFAULT) && HIWORD(wparam) == BN_CLICKED) {
            HWND status_bar = GetDlgItem(hwnd, IDC_STATUSBAR);
            if (status_bar != NULL) {
                set_status_text(status_bar, L"Button clicked");
            }
            return 0;
        }

        break;
    }

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

    INITCOMMONCONTROLSEX controls = {0};
    controls.dwSize = sizeof(controls);
    controls.dwICC = ICC_STANDARD_CLASSES |
                     ICC_LISTVIEW_CLASSES |
                     ICC_TREEVIEW_CLASSES |
                     ICC_TAB_CLASSES |
                     ICC_PROGRESS_CLASS |
                     ICC_BAR_CLASSES |
                     ICC_DATE_CLASSES |
                     ICC_UPDOWN_CLASS;
    InitCommonControlsEx(&controls);

    const wchar_t *class_name = L"AllControlsFixtureClass";

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
        L"all_controls_c_app",
        WS_OVERLAPPEDWINDOW,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        920,
        640,
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
