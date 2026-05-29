#include <windows.h>
#include <commctrl.h>
#include <stdio.h>

#define IDD_ALL_CONTROLS_DIALOG 3001

#define IDC_BTN_PUSH 3101
#define IDC_CHK_OPTION 3102
#define IDC_RAD_ONE 3103
#define IDC_RAD_TWO 3104
#define IDC_EDIT_SINGLE 3105
#define IDC_EDIT_PASSWORD 3106
#define IDC_EDIT_MULTI 3107
#define IDC_LISTBOX 3108
#define IDC_COMBOBOX 3109
#define IDC_SCROLL_H 3110
#define IDC_SCROLL_V 3111
#define IDC_LISTVIEW 3112
#define IDC_TREEVIEW 3113
#define IDC_TAB 3114
#define IDC_PROGRESS 3115
#define IDC_TRACKBAR 3116
#define IDC_UPDOWN_EDIT 3117
#define IDC_UPDOWN 3118
#define IDC_DATETIME 3119
#define IDC_MONTHCAL 3120
#define IDC_HOTKEY 3121
#define IDC_HEADER 3122

static void populate_list_box(HWND list_box) {
    SendMessageW(list_box, LB_ADDSTRING, 0, (LPARAM)L"List item 1");
    SendMessageW(list_box, LB_ADDSTRING, 0, (LPARAM)L"List item 2");
    SendMessageW(list_box, LB_ADDSTRING, 0, (LPARAM)L"List item 3");
    SendMessageW(list_box, LB_SETCURSEL, 0, 0);
}

static void populate_combo_box(HWND combo_box) {
    SendMessageW(combo_box, CB_ADDSTRING, 0, (LPARAM)L"Choice A");
    SendMessageW(combo_box, CB_ADDSTRING, 0, (LPARAM)L"Choice B");
    SendMessageW(combo_box, CB_ADDSTRING, 0, (LPARAM)L"Choice C");
    SendMessageW(combo_box, CB_SETCURSEL, 0, 0);
}

static void populate_list_view(HWND list_view) {
    LVCOLUMNW column = {0};
    column.mask = LVCF_TEXT | LVCF_WIDTH;
    column.cx = 76;

    column.pszText = L"Name";
    ListView_InsertColumn(list_view, 0, &column);

    column.pszText = L"Kind";
    ListView_InsertColumn(list_view, 1, &column);

    LVITEMW item = {0};
    item.mask = LVIF_TEXT;

    item.iItem = 0;
    item.iSubItem = 0;
    item.pszText = L"Alpha";
    ListView_InsertItem(list_view, &item);
    ListView_SetItemText(list_view, 0, 1, L"Fixture");

    item.iItem = 1;
    item.iSubItem = 0;
    item.pszText = L"Beta";
    ListView_InsertItem(list_view, &item);
    ListView_SetItemText(list_view, 1, 1, L"Sample");
}

static void populate_tree_view(HWND tree_view) {
    TVINSERTSTRUCTW insert = {0};

    insert.hParent = TVI_ROOT;
    insert.hInsertAfter = TVI_LAST;
    insert.item.mask = TVIF_TEXT;
    insert.item.pszText = L"Root";
    HTREEITEM root = TreeView_InsertItem(tree_view, &insert);

    insert.hParent = root;
    insert.item.pszText = L"Child A";
    TreeView_InsertItem(tree_view, &insert);

    insert.item.pszText = L"Child B";
    TreeView_InsertItem(tree_view, &insert);

    TreeView_Expand(tree_view, root, TVE_EXPAND);
}

static void populate_tab_control(HWND tab_control) {
    TCITEMW item = {0};
    item.mask = TCIF_TEXT;

    item.pszText = L"Tab 1";
    TabCtrl_InsertItem(tab_control, 0, &item);

    item.pszText = L"Tab 2";
    TabCtrl_InsertItem(tab_control, 1, &item);
}

static void populate_header(HWND header) {
    HDITEMW item = {0};
    item.mask = HDI_TEXT | HDI_WIDTH | HDI_FORMAT;
    item.fmt = HDF_LEFT | HDF_STRING;

    item.pszText = L"Header A";
    item.cxy = 84;
    Header_InsertItem(header, 0, &item);

    item.pszText = L"Header B";
    item.cxy = 84;
    Header_InsertItem(header, 1, &item);
}

static INT_PTR CALLBACK dialog_proc(HWND dialog, UINT msg, WPARAM wparam, LPARAM lparam) {
    (void)lparam;

    switch (msg) {
    case WM_INITDIALOG: {
        HWND list_box = GetDlgItem(dialog, IDC_LISTBOX);
        HWND combo_box = GetDlgItem(dialog, IDC_COMBOBOX);
        HWND list_view = GetDlgItem(dialog, IDC_LISTVIEW);
        HWND tree_view = GetDlgItem(dialog, IDC_TREEVIEW);
        HWND tab_control = GetDlgItem(dialog, IDC_TAB);
        HWND progress = GetDlgItem(dialog, IDC_PROGRESS);
        HWND trackbar = GetDlgItem(dialog, IDC_TRACKBAR);
        HWND updown_edit = GetDlgItem(dialog, IDC_UPDOWN_EDIT);
        HWND updown = GetDlgItem(dialog, IDC_UPDOWN);
        HWND hotkey = GetDlgItem(dialog, IDC_HOTKEY);
        HWND header = GetDlgItem(dialog, IDC_HEADER);

        SetDlgItemTextW(dialog, IDC_EDIT_SINGLE, L"Single-line text");
        SetDlgItemTextW(dialog, IDC_EDIT_PASSWORD, L"secret");
        SetDlgItemTextW(dialog, IDC_EDIT_MULTI, L"Multiline\r\nline 2");

        CheckRadioButton(dialog, IDC_RAD_ONE, IDC_RAD_TWO, IDC_RAD_ONE);
        CheckDlgButton(dialog, IDC_CHK_OPTION, BST_UNCHECKED);

        SetScrollRange(GetDlgItem(dialog, IDC_SCROLL_H), SB_CTL, 0, 100, TRUE);
        SetScrollPos(GetDlgItem(dialog, IDC_SCROLL_H), SB_CTL, 45, TRUE);
        SetScrollRange(GetDlgItem(dialog, IDC_SCROLL_V), SB_CTL, 0, 100, TRUE);
        SetScrollPos(GetDlgItem(dialog, IDC_SCROLL_V), SB_CTL, 55, TRUE);

        populate_list_box(list_box);
        populate_combo_box(combo_box);
        populate_list_view(list_view);
        populate_tree_view(tree_view);
        populate_tab_control(tab_control);
        populate_header(header);

        SendMessageW(progress, PBM_SETRANGE, 0, MAKELPARAM(0, 100));
        SendMessageW(progress, PBM_SETPOS, 62, 0);

        SendMessageW(trackbar, TBM_SETRANGE, TRUE, MAKELPARAM(0, 100));
        SendMessageW(trackbar, TBM_SETTICFREQ, 10, 0);
        SendMessageW(trackbar, TBM_SETPOS, TRUE, 38);

        SendMessageW(updown, UDM_SETBUDDY, (WPARAM)updown_edit, 0);
        SendMessageW(updown, UDM_SETRANGE32, 0, 100);
        SendMessageW(updown, UDM_SETPOS32, 0, 5);

        SendMessageW(hotkey, HKM_SETHOTKEY, MAKEWORD('D', HOTKEYF_CONTROL), 0);

        return (INT_PTR)TRUE;
    }

    case WM_COMMAND: {
        WORD id = LOWORD(wparam);

        if (id == IDC_BTN_PUSH && HIWORD(wparam) == BN_CLICKED) {
            MessageBoxW(dialog, L"Push button clicked.", L"Fixture", MB_OK | MB_ICONINFORMATION);
            return (INT_PTR)TRUE;
        }

        if (id == IDOK || id == IDCANCEL) {
            EndDialog(dialog, id);
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

    INT_PTR result = DialogBoxParamW(
        instance,
        MAKEINTRESOURCEW(IDD_ALL_CONTROLS_DIALOG),
        NULL,
        dialog_proc,
        0);

    if (result == -1) {
        MessageBoxW(NULL, L"DialogBoxParamW failed.", L"Error", MB_ICONERROR | MB_OK);
        return 1;
    }

    return 0;
}
