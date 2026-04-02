//! Windows window management types (user32.dll).
//!
//! This module defines structures and constants for Win32 window management,
//! message handling, and the window class system.

use core::fmt;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------
// Window Handle (HWND)
// ---------------------------------------------------------------------------

/// A Windows window handle (HWND).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Hwnd(usize);

impl Hwnd {
    pub const NULL: Self = Self(0);

    #[inline]
    pub const fn from_raw(value: usize) -> Self {
        Self(value)
    }

    #[inline]
    pub const fn as_raw(self) -> usize {
        self.0
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }
}

impl fmt::Debug for Hwnd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HWND({:#x})", self.0)
    }
}

// ---------------------------------------------------------------------------
// Window Class Types
// ---------------------------------------------------------------------------

/// Window class styles (CS_*).
pub mod class_style {
    pub const CS_VREDRAW: u32 = 0x0001;
    pub const CS_HREDRAW: u32 = 0x0002;
    pub const CS_DBLCLKS: u32 = 0x0008;
    pub const CS_OWNDC: u32 = 0x0020;
    pub const CS_CLASSDC: u32 = 0x0040;
    pub const CS_PARENTDC: u32 = 0x0080;
    pub const CS_NOCLOSE: u32 = 0x0200;
    pub const CS_SAVEBITS: u32 = 0x0800;
    pub const CS_BYTEALIGNCLIENT: u32 = 0x1000;
    pub const CS_BYTEALIGNWINDOW: u32 = 0x2000;
    pub const CS_GLOBALCLASS: u32 = 0x4000;
}

/// Window styles (WS_*).
pub mod window_style {
    pub const WS_OVERLAPPED: u32 = 0x00000000;
    pub const WS_POPUP: u32 = 0x80000000;
    pub const WS_CHILD: u32 = 0x40000000;
    pub const WS_MINIMIZE: u32 = 0x20000000;
    pub const WS_VISIBLE: u32 = 0x10000000;
    pub const WS_DISABLED: u32 = 0x08000000;
    pub const WS_CLIPSIBLINGS: u32 = 0x04000000;
    pub const WS_CLIPCHILDREN: u32 = 0x02000000;
    pub const WS_MAXIMIZE: u32 = 0x01000000;
    pub const WS_CAPTION: u32 = 0x00C00000;
    pub const WS_BORDER: u32 = 0x00800000;
    pub const WS_DLGFRAME: u32 = 0x00400000;
    pub const WS_VSCROLL: u32 = 0x00200000;
    pub const WS_HSCROLL: u32 = 0x00100000;
    pub const WS_SYSMENU: u32 = 0x00080000;
    pub const WS_THICKFRAME: u32 = 0x00040000;
    pub const WS_GROUP: u32 = 0x00020000;
    pub const WS_TABSTOP: u32 = 0x00010000;
    pub const WS_MINIMIZEBOX: u32 = 0x00020000;
    pub const WS_MAXIMIZEBOX: u32 = 0x00010000;

    pub const WS_OVERLAPPEDWINDOW: u32 =
        WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX;
    pub const WS_POPUPWINDOW: u32 = WS_POPUP | WS_BORDER | WS_SYSMENU;
}

/// Extended window styles (WS_EX_*).
pub mod window_style_ex {
    pub const WS_EX_DLGMODALFRAME: u32 = 0x00000001;
    pub const WS_EX_NOPARENTNOTIFY: u32 = 0x00000004;
    pub const WS_EX_TOPMOST: u32 = 0x00000008;
    pub const WS_EX_ACCEPTFILES: u32 = 0x00000010;
    pub const WS_EX_TRANSPARENT: u32 = 0x00000020;
    pub const WS_EX_MDICHILD: u32 = 0x00000040;
    pub const WS_EX_TOOLWINDOW: u32 = 0x00000080;
    pub const WS_EX_WINDOWEDGE: u32 = 0x00000100;
    pub const WS_EX_CLIENTEDGE: u32 = 0x00000200;
    pub const WS_EX_CONTEXTHELP: u32 = 0x00000400;
    pub const WS_EX_RIGHT: u32 = 0x00001000;
    pub const WS_EX_LEFT: u32 = 0x00000000;
    pub const WS_EX_RTLREADING: u32 = 0x00002000;
    pub const WS_EX_LTRREADING: u32 = 0x00000000;
    pub const WS_EX_LEFTSCROLLBAR: u32 = 0x00004000;
    pub const WS_EX_RIGHTSCROLLBAR: u32 = 0x00000000;
    pub const WS_EX_CONTROLPARENT: u32 = 0x00010000;
    pub const WS_EX_STATICEDGE: u32 = 0x00020000;
    pub const WS_EX_APPWINDOW: u32 = 0x00040000;
    pub const WS_EX_OVERLAPPEDWINDOW: u32 = WS_EX_WINDOWEDGE | WS_EX_CLIENTEDGE;
    pub const WS_EX_PALETTEWINDOW: u32 = WS_EX_WINDOWEDGE | WS_EX_TOOLWINDOW | WS_EX_TOPMOST;
    pub const WS_EX_LAYERED: u32 = 0x00080000;
    pub const WS_EX_NOINHERITLAYOUT: u32 = 0x00100000;
    pub const WS_EX_NOREDIRECTIONBITMAP: u32 = 0x00200000;
    pub const WS_EX_LAYOUTRTL: u32 = 0x00400000;
    pub const WS_EX_COMPOSITED: u32 = 0x02000000;
    pub const WS_EX_NOACTIVATE: u32 = 0x08000000;
}

/// ShowWindow commands.
pub mod show_window {
    pub const SW_HIDE: i32 = 0;
    pub const SW_SHOWNORMAL: i32 = 1;
    pub const SW_NORMAL: i32 = 1;
    pub const SW_SHOWMINIMIZED: i32 = 2;
    pub const SW_SHOWMAXIMIZED: i32 = 3;
    pub const SW_MAXIMIZE: i32 = 3;
    pub const SW_SHOWNOACTIVATE: i32 = 4;
    pub const SW_SHOW: i32 = 5;
    pub const SW_MINIMIZE: i32 = 6;
    pub const SW_SHOWMINNOACTIVE: i32 = 7;
    pub const SW_SHOWNA: i32 = 8;
    pub const SW_RESTORE: i32 = 9;
    pub const SW_SHOWDEFAULT: i32 = 10;
    pub const SW_FORCEMINIMIZE: i32 = 11;
}

// ---------------------------------------------------------------------------
// Message Types
// ---------------------------------------------------------------------------

/// Window messages (WM_*).
pub mod window_message {
    pub const WM_NULL: u32 = 0x0000;
    pub const WM_CREATE: u32 = 0x0001;
    pub const WM_DESTROY: u32 = 0x0002;
    pub const WM_MOVE: u32 = 0x0003;
    pub const WM_SIZE: u32 = 0x0005;
    pub const WM_ACTIVATE: u32 = 0x0006;
    pub const WM_SETFOCUS: u32 = 0x0007;
    pub const WM_KILLFOCUS: u32 = 0x0008;
    pub const WM_ENABLE: u32 = 0x000A;
    pub const WM_SETREDRAW: u32 = 0x000B;
    pub const WM_SETTEXT: u32 = 0x000C;
    pub const WM_GETTEXT: u32 = 0x000D;
    pub const WM_GETTEXTLENGTH: u32 = 0x000E;
    pub const WM_PAINT: u32 = 0x000F;
    pub const WM_CLOSE: u32 = 0x0010;
    pub const WM_QUERYENDSESSION: u32 = 0x0011;
    pub const WM_QUIT: u32 = 0x0012;
    pub const WM_QUERYOPEN: u32 = 0x0013;
    pub const WM_ERASEBKGND: u32 = 0x0014;
    pub const WM_SYSCOLORCHANGE: u32 = 0x0015;
    pub const WM_ENDSESSION: u32 = 0x0016;
    pub const WM_SHOWWINDOW: u32 = 0x0018;
    pub const WM_WININICHANGE: u32 = 0x001A;
    pub const WM_SETTINGCHANGE: u32 = 0x001A;
    pub const WM_DEVMODECHANGE: u32 = 0x001B;
    pub const WM_ACTIVATEAPP: u32 = 0x001C;
    pub const WM_FONTCHANGE: u32 = 0x001D;
    pub const WM_TIMECHANGE: u32 = 0x001E;
    pub const WM_CANCELMODE: u32 = 0x001F;
    pub const WM_SETCURSOR: u32 = 0x0020;
    pub const WM_MOUSEACTIVATE: u32 = 0x0021;
    pub const WM_CHILDACTIVATE: u32 = 0x0022;
    pub const WM_QUEUESYNC: u32 = 0x0023;
    pub const WM_GETMINMAXINFO: u32 = 0x0024;
    pub const WM_PAINTICON: u32 = 0x0026;
    pub const WM_ICONERASEBKGND: u32 = 0x0027;
    pub const WM_NEXTDLGCTL: u32 = 0x0028;
    pub const WM_SPOOLERSTATUS: u32 = 0x002A;
    pub const WM_DRAWITEM: u32 = 0x002B;
    pub const WM_MEASUREITEM: u32 = 0x002C;
    pub const WM_DELETEITEM: u32 = 0x002D;
    pub const WM_VKEYTOITEM: u32 = 0x002E;
    pub const WM_CHARTOITEM: u32 = 0x002F;
    pub const WM_SETFONT: u32 = 0x0030;
    pub const WM_GETFONT: u32 = 0x0031;
    pub const WM_SETHOTKEY: u32 = 0x0032;
    pub const WM_GETHOTKEY: u32 = 0x0033;
    pub const WM_QUERYDRAGICON: u32 = 0x0037;
    pub const WM_COMPAREITEM: u32 = 0x0039;
    pub const WM_GETOBJECT: u32 = 0x003D;
    pub const WM_COMPACTING: u32 = 0x0041;
    pub const WM_COMMNOTIFY: u32 = 0x0044;
    pub const WM_WINDOWPOSCHANGING: u32 = 0x0046;
    pub const WM_WINDOWPOSCHANGED: u32 = 0x0047;
    pub const WM_POWER: u32 = 0x0048;
    pub const WM_COPYDATA: u32 = 0x004A;
    pub const WM_CANCELJOURNAL: u32 = 0x004B;
    pub const WM_NOTIFY: u32 = 0x004E;
    pub const WM_INPUTLANGCHANGEREQUEST: u32 = 0x0050;
    pub const WM_INPUTLANGCHANGE: u32 = 0x0051;
    pub const WM_TCARD: u32 = 0x0052;
    pub const WM_HELP: u32 = 0x0053;
    pub const WM_USERCHANGED: u32 = 0x0054;
    pub const WM_NOTIFYFORMAT: u32 = 0x0055;
    pub const WM_CONTEXTMENU: u32 = 0x007B;
    pub const WM_STYLECHANGING: u32 = 0x007C;
    pub const WM_STYLECHANGED: u32 = 0x007D;
    pub const WM_DISPLAYCHANGE: u32 = 0x007E;
    pub const WM_GETICON: u32 = 0x007F;
    pub const WM_SETICON: u32 = 0x0080;
    pub const WM_NCCREATE: u32 = 0x0081;
    pub const WM_NCDESTROY: u32 = 0x0082;
    pub const WM_NCCALCSIZE: u32 = 0x0083;
    pub const WM_NCHITTEST: u32 = 0x0084;
    pub const WM_NCPAINT: u32 = 0x0085;
    pub const WM_NCACTIVATE: u32 = 0x0086;
    pub const WM_GETDLGCODE: u32 = 0x0087;
    pub const WM_SYNCPAINT: u32 = 0x0088;
    pub const WM_NCMOUSEMOVE: u32 = 0x00A0;
    pub const WM_NCLBUTTONDOWN: u32 = 0x00A1;
    pub const WM_NCLBUTTONUP: u32 = 0x00A2;
    pub const WM_NCLBUTTONDBLCLK: u32 = 0x00A3;
    pub const WM_NCRBUTTONDOWN: u32 = 0x00A4;
    pub const WM_NCRBUTTONUP: u32 = 0x00A5;
    pub const WM_NCRBUTTONDBLCLK: u32 = 0x00A6;
    pub const WM_NCMBUTTONDOWN: u32 = 0x00A7;
    pub const WM_NCMBUTTONUP: u32 = 0x00A8;
    pub const WM_NCMBUTTONDBLCLK: u32 = 0x00A9;
    pub const WM_NCXBUTTONDOWN: u32 = 0x00AB;
    pub const WM_NCXBUTTONUP: u32 = 0x00AC;
    pub const WM_NCXBUTTONDBLCLK: u32 = 0x00AD;
    pub const WM_KEYFIRST: u32 = 0x0100;
    pub const WM_KEYDOWN: u32 = 0x0100;
    pub const WM_KEYUP: u32 = 0x0101;
    pub const WM_CHAR: u32 = 0x0102;
    pub const WM_DEADCHAR: u32 = 0x0103;
    pub const WM_SYSKEYDOWN: u32 = 0x0104;
    pub const WM_SYSKEYUP: u32 = 0x0105;
    pub const WM_SYSCHAR: u32 = 0x0106;
    pub const WM_SYSDEADCHAR: u32 = 0x0107;
    pub const WM_KEYLAST: u32 = 0x0109;
    pub const WM_UNICHAR: u32 = 0x0109;
    pub const WM_IME_STARTCOMPOSITION: u32 = 0x010D;
    pub const WM_IME_ENDCOMPOSITION: u32 = 0x010E;
    pub const WM_IME_COMPOSITION: u32 = 0x010F;
    pub const WM_IME_KEYLAST: u32 = 0x010F;
    pub const WM_INITDIALOG: u32 = 0x0110;
    pub const WM_COMMAND: u32 = 0x0111;
    pub const WM_SYSCOMMAND: u32 = 0x0112;
    pub const WM_TIMER: u32 = 0x0113;
    pub const WM_HSCROLL: u32 = 0x0114;
    pub const WM_VSCROLL: u32 = 0x0115;
    pub const WM_INITMENU: u32 = 0x0116;
    pub const WM_INITMENUPOPUP: u32 = 0x0117;
    pub const WM_MENUSELECT: u32 = 0x011F;
    pub const WM_MENUCHAR: u32 = 0x0120;
    pub const WM_ENTERIDLE: u32 = 0x0121;
    pub const WM_MENURBUTTONUP: u32 = 0x0122;
    pub const WM_MENUDRAG: u32 = 0x0123;
    pub const WM_MENUGETOBJECT: u32 = 0x0124;
    pub const WM_UNINITMENUPOPUP: u32 = 0x0125;
    pub const WM_MENUCOMMAND: u32 = 0x0126;
    pub const WM_CHANGEUISTATE: u32 = 0x0127;
    pub const WM_UPDATEUISTATE: u32 = 0x0128;
    pub const WM_QUERYUISTATE: u32 = 0x0129;
    pub const WM_CTLCOLORMSGBOX: u32 = 0x0132;
    pub const WM_CTLCOLOREDIT: u32 = 0x0133;
    pub const WM_CTLCOLORLISTBOX: u32 = 0x0134;
    pub const WM_CTLCOLORBTN: u32 = 0x0135;
    pub const WM_CTLCOLORDLG: u32 = 0x0136;
    pub const WM_CTLCOLORSCROLLBAR: u32 = 0x0137;
    pub const WM_CTLCOLORSTATIC: u32 = 0x0138;
    pub const WM_MOUSEFIRST: u32 = 0x0200;
    pub const WM_MOUSEMOVE: u32 = 0x0200;
    pub const WM_LBUTTONDOWN: u32 = 0x0201;
    pub const WM_LBUTTONUP: u32 = 0x0202;
    pub const WM_LBUTTONDBLCLK: u32 = 0x0203;
    pub const WM_RBUTTONDOWN: u32 = 0x0204;
    pub const WM_RBUTTONUP: u32 = 0x0205;
    pub const WM_RBUTTONDBLCLK: u32 = 0x0206;
    pub const WM_MBUTTONDOWN: u32 = 0x0207;
    pub const WM_MBUTTONUP: u32 = 0x0208;
    pub const WM_MBUTTONDBLCLK: u32 = 0x0209;
    pub const WM_MOUSEWHEEL: u32 = 0x020A;
    pub const WM_XBUTTONDOWN: u32 = 0x020B;
    pub const WM_XBUTTONUP: u32 = 0x020C;
    pub const WM_XBUTTONDBLCLK: u32 = 0x020D;
    pub const WM_MOUSEHWHEEL: u32 = 0x020E;
    pub const WM_MOUSELAST: u32 = 0x020E;
    pub const WM_PARENTNOTIFY: u32 = 0x0210;
    pub const WM_ENTERMENULOOP: u32 = 0x0211;
    pub const WM_EXITMENULOOP: u32 = 0x0212;
    pub const WM_NEXTMENU: u32 = 0x0213;
    pub const WM_SIZING: u32 = 0x0214;
    pub const WM_CAPTURECHANGED: u32 = 0x0215;
    pub const WM_MOVING: u32 = 0x0216;
    pub const WM_POWERBROADCAST: u32 = 0x0218;
    pub const WM_DEVICECHANGE: u32 = 0x0219;
    pub const WM_MDICREATE: u32 = 0x0220;
    pub const WM_MDIDESTROY: u32 = 0x0221;
    pub const WM_MDIACTIVATE: u32 = 0x0222;
    pub const WM_MDIRESTORE: u32 = 0x0223;
    pub const WM_MDINEXT: u32 = 0x0224;
    pub const WM_MDIMAXIMIZE: u32 = 0x0225;
    pub const WM_MDITILE: u32 = 0x0226;
    pub const WM_MDICASCADE: u32 = 0x0227;
    pub const WM_MDIICONARRANGE: u32 = 0x0228;
    pub const WM_MDIGETACTIVE: u32 = 0x0229;
    pub const WM_MDISETMENU: u32 = 0x0230;
    pub const WM_ENTERSIZEMOVE: u32 = 0x0231;
    pub const WM_EXITSIZEMOVE: u32 = 0x0232;
    pub const WM_DROPFILES: u32 = 0x0233;
    pub const WM_MDIREFRESHMENU: u32 = 0x0234;
    pub const WM_IME_SETCONTEXT: u32 = 0x0281;
    pub const WM_IME_NOTIFY: u32 = 0x0282;
    pub const WM_IME_CONTROL: u32 = 0x0283;
    pub const WM_IME_COMPOSITIONFULL: u32 = 0x0284;
    pub const WM_IME_SELECT: u32 = 0x0285;
    pub const WM_IME_CHAR: u32 = 0x0286;
    pub const WM_IME_REQUEST: u32 = 0x0288;
    pub const WM_IME_KEYDOWN: u32 = 0x0290;
    pub const WM_IME_KEYUP: u32 = 0x0291;
    pub const WM_MOUSEHOVER: u32 = 0x02A1;
    pub const WM_MOUSELEAVE: u32 = 0x02A3;
    pub const WM_NCMOUSEHOVER: u32 = 0x02A0;
    pub const WM_NCMOUSELEAVE: u32 = 0x02A2;
    pub const WM_WTSSESSION_CHANGE: u32 = 0x02B1;
    pub const WM_TABLET_FIRST: u32 = 0x02C0;
    pub const WM_TABLET_LAST: u32 = 0x02DF;
    pub const WM_CUT: u32 = 0x0300;
    pub const WM_COPY: u32 = 0x0301;
    pub const WM_PASTE: u32 = 0x0302;
    pub const WM_CLEAR: u32 = 0x0303;
    pub const WM_UNDO: u32 = 0x0304;
    pub const WM_RENDERFORMAT: u32 = 0x0305;
    pub const WM_RENDERALLFORMATS: u32 = 0x0306;
    pub const WM_DESTROYCLIPBOARD: u32 = 0x0307;
    pub const WM_DRAWCLIPBOARD: u32 = 0x0308;
    pub const WM_PAINTCLIPBOARD: u32 = 0x0309;
    pub const WM_VSCROLLCLIPBOARD: u32 = 0x030A;
    pub const WM_SIZECLIPBOARD: u32 = 0x030B;
    pub const WM_ASKCBFORMATNAME: u32 = 0x030C;
    pub const WM_CHANGECBCHAIN: u32 = 0x030D;
    pub const WM_HSCROLLCLIPBOARD: u32 = 0x030E;
    pub const WM_QUERYNEWPALETTE: u32 = 0x030F;
    pub const WM_PALETTEISCHANGING: u32 = 0x0310;
    pub const WM_PALETTECHANGED: u32 = 0x0311;
    pub const WM_HOTKEY: u32 = 0x0312;
    pub const WM_PRINT: u32 = 0x0317;
    pub const WM_PRINTCLIENT: u32 = 0x0318;
    pub const WM_APPCOMMAND: u32 = 0x0319;
    pub const WM_THEMECHANGED: u32 = 0x031A;
    pub const WM_CLIPBOARDUPDATE: u32 = 0x031D;
    pub const WM_DWMCOMPOSITIONCHANGED: u32 = 0x031E;
    pub const WM_DWMNCRENDERINGCHANGED: u32 = 0x031F;
    pub const WM_DWMCOLORIZATIONCOLORCHANGED: u32 = 0x0320;
    pub const WM_DWMWINDOWMAXIMIZEDCHANGE: u32 = 0x0321;
    pub const WM_GETTITLEBARINFOEX: u32 = 0x033F;
    pub const WM_USER: u32 = 0x0400;
    pub const WM_APP: u32 = 0x8000;
}

// ---------------------------------------------------------------------------
// MSG structure
// ---------------------------------------------------------------------------

/// The Windows MSG structure used by GetMessage/DispatchMessage.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Msg {
    pub hwnd: Hwnd,
    pub message: u32,
    pub w_param: usize,
    pub l_param: isize,
    pub time: u32,
    pub pt: Point,
}

/// The Windows POINT structure.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

// ---------------------------------------------------------------------------
// WNDCLASSEX structure
// ---------------------------------------------------------------------------

/// The Windows WNDCLASSEXA structure (ANSI version).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WndClassExA {
    pub cb_size: u32,
    pub style: u32,
    pub lpfn_wnd_proc: usize, // function pointer
    pub cb_cls_extra: i32,
    pub cb_wnd_extra: i32,
    pub h_instance: usize,
    pub h_icon: usize,
    pub h_cursor: usize,
    pub hbr_background: usize,
    pub lpsz_menu_name: *const u8,
    pub lpsz_class_name: *const u8,
    pub h_icon_sm: usize,
}

/// The Windows WNDCLASSEXW structure (Unicode version).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WndClassExW {
    pub cb_size: u32,
    pub style: u32,
    pub lpfn_wnd_proc: usize, // function pointer
    pub cb_cls_extra: i32,
    pub cb_wnd_extra: i32,
    pub h_instance: usize,
    pub h_icon: usize,
    pub h_cursor: usize,
    pub hbr_background: usize,
    pub lpsz_menu_name: *const u16,
    pub lpsz_class_name: *const u16,
    pub h_icon_sm: usize,
}

// ---------------------------------------------------------------------------
// RECT structure
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

// ---------------------------------------------------------------------------
// POINT structure
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Window Class Storage
// ---------------------------------------------------------------------------

/// A registered window class.
#[derive(Debug, Clone)]
pub struct WindowClass {
    pub name: String,
    pub style: u32,
    pub wnd_proc: usize,
    pub cls_extra: i32,
    pub wnd_extra: i32,
    pub instance: usize,
    pub icon: usize,
    pub cursor: usize,
    pub background: usize,
    pub menu_name: Option<String>,
    pub icon_sm: usize,
}

/// Global registry of window classes.
pub struct WindowClassRegistry {
    classes: Mutex<HashMap<String, WindowClass>>,
}

impl WindowClassRegistry {
    pub fn new() -> Self {
        Self {
            classes: Mutex::new(HashMap::new()),
        }
    }

    pub fn register(&self, name: String, class: WindowClass) {
        let mut classes = self.classes.lock().unwrap();
        classes.insert(name, class);
    }

    pub fn get(&self, name: &str) -> Option<WindowClass> {
        let classes = self.classes.lock().unwrap();
        classes.get(name).cloned()
    }

    pub fn unregister(&self, name: &str) -> bool {
        let mut classes = self.classes.lock().unwrap();
        classes.remove(name).is_some()
    }
}

impl Default for WindowClassRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Window State
// ---------------------------------------------------------------------------

/// Per-window state tracked by the user32 implementation.
#[derive(Debug, Clone)]
pub struct WindowState {
    /// Window handle.
    pub hwnd: Hwnd,
    /// Window class name.
    pub class_name: String,
    /// Window title.
    pub title: String,
    /// Window styles (WS_*).
    pub style: u32,
    /// Extended window styles (WS_EX_*).
    pub ex_style: u32,
    /// Window position and size.
    pub rect: Rect,
    /// Client area rectangle.
    pub client_rect: Rect,
    /// Parent window handle.
    pub parent: Hwnd,
    /// Whether the window is visible.
    pub visible: bool,
    /// Whether the window is enabled.
    pub enabled: bool,
    /// Window procedure from the class.
    pub wnd_proc: usize,
    /// User data pointer (SetWindowLongPtr).
    pub user_data: usize,
}

/// Global window state manager.
pub struct WindowManager {
    windows: Mutex<HashMap<Hwnd, WindowState>>,
    next_hwnd: Mutex<usize>,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            windows: Mutex::new(HashMap::new()),
            next_hwnd: Mutex::new(0x10000),
        }
    }

    pub fn create_window(&self, state: WindowState) -> Hwnd {
        let mut windows = self.windows.lock().unwrap();
        let mut next_hwnd = self.next_hwnd.lock().unwrap();

        let hwnd = Hwnd::from_raw(*next_hwnd);
        *next_hwnd += 1;

        let mut state = state;
        state.hwnd = hwnd;
        windows.insert(hwnd, state);

        hwnd
    }

    pub fn get_window(&self, hwnd: Hwnd) -> Option<WindowState> {
        let windows = self.windows.lock().unwrap();
        windows.get(&hwnd).cloned()
    }

    pub fn update_window<F>(&self, hwnd: Hwnd, f: F) -> bool
    where
        F: FnOnce(&mut WindowState),
    {
        let mut windows = self.windows.lock().unwrap();
        if let Some(state) = windows.get_mut(&hwnd) {
            f(state);
            true
        } else {
            false
        }
    }

    pub fn destroy_window(&self, hwnd: Hwnd) -> bool {
        let mut windows = self.windows.lock().unwrap();
        windows.remove(&hwnd).is_some()
    }
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Message Queue
// ---------------------------------------------------------------------------

/// A message queue for a single thread.
#[derive(Debug)]
pub struct MessageQueue {
    messages: Mutex<Vec<Msg>>,
    quit: Mutex<bool>,
    quit_code: Mutex<i32>,
}

impl MessageQueue {
    pub fn new() -> Self {
        Self {
            messages: Mutex::new(Vec::new()),
            quit: Mutex::new(false),
            quit_code: Mutex::new(0),
        }
    }

    pub fn post_message(&self, msg: Msg) {
        let mut messages = self.messages.lock().unwrap();
        messages.push(msg);
    }

    pub fn post_quit(&self, exit_code: i32) {
        *self.quit.lock().unwrap() = true;
        *self.quit_code.lock().unwrap() = exit_code;
    }

    pub fn get_message(&self, msg: &mut Msg) -> bool {
        loop {
            // Check for quit
            if *self.quit.lock().unwrap() {
                msg.message = window_message::WM_QUIT;
                msg.w_param = *self.quit_code.lock().unwrap() as usize;
                return false;
            }

            // Check for pending messages - retrieve from front of queue (FIFO)
            let mut messages = self.messages.lock().unwrap();
            if !messages.is_empty() {
                *msg = messages.remove(0);
                return true;
            }

            // No messages, yield
            drop(messages);
            std::thread::yield_now();
        }
    }

    pub fn peek_message(&self, msg: &mut Msg, remove: bool) -> bool {
        // Check for quit first
        if *self.quit.lock().unwrap() {
            msg.message = window_message::WM_QUIT;
            msg.w_param = *self.quit_code.lock().unwrap() as usize;
            return true;
        }

        let mut messages = self.messages.lock().unwrap();
        if messages.is_empty() {
            return false;
        }

        // Treat messages as a FIFO queue: read/remove from the front
        *msg = messages[0];
        if remove {
            messages.remove(0);
        }
        true
    }
}

impl Default for MessageQueue {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Global State
// ---------------------------------------------------------------------------

use std::sync::LazyLock;

pub static WINDOW_CLASS_REGISTRY: LazyLock<Arc<WindowClassRegistry>> =
    LazyLock::new(|| Arc::new(WindowClassRegistry::new()));

pub static WINDOW_MANAGER: LazyLock<Arc<WindowManager>> =
    LazyLock::new(|| Arc::new(WindowManager::new()));

thread_local! {
    pub static THREAD_MESSAGE_QUEUE: MessageQueue = MessageQueue::new();
}

// ---------------------------------------------------------------------------
// Unit Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── Hwnd ──────────────────────────────────────────────────────

    #[test]
    fn hwnd_null() {
        assert!(Hwnd::NULL.is_null());
        assert_eq!(Hwnd::NULL.as_raw(), 0);
    }

    #[test]
    fn hwnd_from_raw() {
        let hwnd = Hwnd::from_raw(0x1234);
        assert_eq!(hwnd.as_raw(), 0x1234);
        assert!(!hwnd.is_null());
    }

    // ── WindowClassRegistry ────────────────────────────────────────

    #[test]
    fn window_class_register_and_get() {
        let registry = WindowClassRegistry::new();
        let class = WindowClass {
            name: "TestClass".into(),
            style: class_style::CS_HREDRAW | class_style::CS_VREDRAW,
            wnd_proc: 0x1000,
            cls_extra: 0,
            wnd_extra: 0,
            instance: 0,
            icon: 0,
            cursor: 0,
            background: 0,
            menu_name: None,
            icon_sm: 0,
        };

        registry.register("TestClass".into(), class.clone());
        let retrieved = registry.get("TestClass").unwrap();
        assert_eq!(retrieved.name, "TestClass");
        assert_eq!(
            retrieved.style,
            class_style::CS_HREDRAW | class_style::CS_VREDRAW
        );
        assert_eq!(retrieved.wnd_proc, 0x1000);
    }

    #[test]
    fn window_class_get_nonexistent() {
        let registry = WindowClassRegistry::new();
        assert!(registry.get("NonExistent").is_none());
    }

    #[test]
    fn window_class_unregister() {
        let registry = WindowClassRegistry::new();
        let class = WindowClass {
            name: "TempClass".into(),
            style: 0,
            wnd_proc: 0x2000,
            cls_extra: 0,
            wnd_extra: 0,
            instance: 0,
            icon: 0,
            cursor: 0,
            background: 0,
            menu_name: None,
            icon_sm: 0,
        };

        registry.register("TempClass".into(), class);
        assert!(registry.get("TempClass").is_some());

        assert!(registry.unregister("TempClass"));
        assert!(registry.get("TempClass").is_none());
        assert!(!registry.unregister("TempClass")); // Second unregister fails
    }

    // ── WindowManager ──────────────────────────────────────────────

    #[test]
    fn window_manager_create_window() {
        let manager = WindowManager::new();
        let state = WindowState {
            hwnd: Hwnd::NULL,
            class_name: "TestClass".into(),
            title: "Test Window".into(),
            style: window_style::WS_OVERLAPPEDWINDOW,
            ex_style: 0,
            rect: Rect {
                left: 100,
                top: 100,
                right: 500,
                bottom: 400,
            },
            client_rect: Rect {
                left: 0,
                top: 0,
                right: 400,
                bottom: 300,
            },
            parent: Hwnd::NULL,
            visible: true,
            enabled: true,
            wnd_proc: 0x3000,
            user_data: 0,
        };

        let hwnd = manager.create_window(state);
        assert!(!hwnd.is_null());

        let retrieved = manager.get_window(hwnd).unwrap();
        assert_eq!(retrieved.title, "Test Window");
        assert_eq!(retrieved.class_name, "TestClass");
        assert_eq!(retrieved.wnd_proc, 0x3000);
    }

    #[test]
    fn window_manager_update_window() {
        let manager = WindowManager::new();
        let state = WindowState {
            hwnd: Hwnd::NULL,
            class_name: "TestClass".into(),
            title: "Original".into(),
            style: window_style::WS_OVERLAPPEDWINDOW,
            ex_style: 0,
            rect: Rect::default(),
            client_rect: Rect::default(),
            parent: Hwnd::NULL,
            visible: false,
            enabled: true,
            wnd_proc: 0,
            user_data: 0,
        };

        let hwnd = manager.create_window(state);

        let updated = manager.update_window(hwnd, |state| {
            state.title = "Updated".into();
            state.visible = true;
        });
        assert!(updated);

        let retrieved = manager.get_window(hwnd).unwrap();
        assert_eq!(retrieved.title, "Updated");
        assert!(retrieved.visible);
    }

    #[test]
    fn window_manager_destroy_window() {
        let manager = WindowManager::new();
        let state = WindowState {
            hwnd: Hwnd::NULL,
            class_name: "TestClass".into(),
            title: "Temp".into(),
            style: 0,
            ex_style: 0,
            rect: Rect::default(),
            client_rect: Rect::default(),
            parent: Hwnd::NULL,
            visible: false,
            enabled: true,
            wnd_proc: 0,
            user_data: 0,
        };

        let hwnd = manager.create_window(state);
        assert!(manager.get_window(hwnd).is_some());

        assert!(manager.destroy_window(hwnd));
        assert!(manager.get_window(hwnd).is_none());
        assert!(!manager.destroy_window(hwnd)); // Second destroy fails
    }

    // ── MessageQueue ───────────────────────────────────────────────

    #[test]
    fn message_queue_post_and_peek() {
        let queue = MessageQueue::new();
        let msg = Msg {
            hwnd: Hwnd::from_raw(0x1000),
            message: window_message::WM_PAINT,
            w_param: 0,
            l_param: 0,
            time: 1000,
            pt: Point { x: 10, y: 20 },
        };

        queue.post_message(msg);

        let mut retrieved = Msg {
            hwnd: Hwnd::NULL,
            message: 0,
            w_param: 0,
            l_param: 0,
            time: 0,
            pt: Point { x: 0, y: 0 },
        };

        assert!(queue.peek_message(&mut retrieved, false)); // Don't remove
        assert_eq!(retrieved.message, window_message::WM_PAINT);
        assert_eq!(retrieved.hwnd.as_raw(), 0x1000);

        assert!(queue.peek_message(&mut retrieved, true)); // Remove
        assert!(!queue.peek_message(&mut retrieved, false)); // Now empty
    }

    #[test]
    fn message_queue_post_quit() {
        let queue = MessageQueue::new();
        queue.post_quit(42);

        let mut msg = Msg {
            hwnd: Hwnd::NULL,
            message: 0,
            w_param: 0,
            l_param: 0,
            time: 0,
            pt: Point { x: 0, y: 0 },
        };

        assert!(queue.peek_message(&mut msg, false));
        assert_eq!(msg.message, window_message::WM_QUIT);
        assert_eq!(msg.w_param, 42);
    }

    // ── Window Styles & Constants ──────────────────────────────────

    #[test]
    fn window_style_overlapped_window() {
        let style = window_style::WS_OVERLAPPEDWINDOW;
        assert!(style & window_style::WS_CAPTION != 0);
        assert!(style & window_style::WS_SYSMENU != 0);
        assert!(style & window_style::WS_THICKFRAME != 0);
    }

    #[test]
    fn show_window_constants() {
        assert_eq!(show_window::SW_HIDE, 0);
        assert_eq!(show_window::SW_SHOWNORMAL, 1);
        assert_eq!(show_window::SW_NORMAL, 1);
        assert_eq!(show_window::SW_SHOW, 5);
    }

    #[test]
    fn window_messages() {
        assert_eq!(window_message::WM_CREATE, 0x0001);
        assert_eq!(window_message::WM_DESTROY, 0x0002);
        assert_eq!(window_message::WM_PAINT, 0x000F);
        assert_eq!(window_message::WM_QUIT, 0x0012);
        assert_eq!(window_message::WM_CLOSE, 0x0010);
    }

    // ── Rect & Point ───────────────────────────────────────────────

    #[test]
    fn rect_default() {
        let rect = Rect::default();
        assert_eq!(rect.left, 0);
        assert_eq!(rect.top, 0);
        assert_eq!(rect.right, 0);
        assert_eq!(rect.bottom, 0);
    }

    #[test]
    fn point_default() {
        let point = Point::default();
        assert_eq!(point.x, 0);
        assert_eq!(point.y, 0);
    }
}
