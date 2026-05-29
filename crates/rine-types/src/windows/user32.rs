//! Windows window management types (user32.dll).
//!
//! This module defines structures and constants for Win32 window management,
//! message handling, and the window class system.

use core::fmt;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::{Arc, Condvar, Mutex};

use crate::handles::HMENU;

// ---------------------------------------------------------------------------
// Window Handle (HWND)
// ---------------------------------------------------------------------------

/// A Windows window handle (HWND).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct HWND(usize);

impl HWND {
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

impl fmt::Debug for HWND {
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
    pub const WA_INACTIVE: u32 = 0;
    pub const WA_ACTIVE: u32 = 1;
    pub const WA_CLICKACTIVE: u32 = 2;
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

/// Accelerator flag bits used by `ACCEL.f_virt`.
pub mod accel_flags {
    pub const FVIRTKEY: u8 = 0x01;
    pub const FNOINVERT: u8 = 0x02;
    pub const FSHIFT: u8 = 0x04;
    pub const FCONTROL: u8 = 0x08;
    pub const FALT: u8 = 0x10;
    pub const FLAST: u8 = 0x80;
}

/// A subset of Win32 virtual-key constants used by keyboard and accelerator paths.
pub mod virtual_key {
    pub const VK_BACK: u16 = 0x08;
    pub const VK_TAB: u16 = 0x09;
    pub const VK_RETURN: u16 = 0x0D;
    pub const VK_SHIFT: u16 = 0x10;
    pub const VK_CONTROL: u16 = 0x11;
    pub const VK_MENU: u16 = 0x12;
    pub const VK_PAUSE: u16 = 0x13;
    pub const VK_CAPITAL: u16 = 0x14;
    pub const VK_ESCAPE: u16 = 0x1B;
    pub const VK_SPACE: u16 = 0x20;
    pub const VK_PRIOR: u16 = 0x21;
    pub const VK_NEXT: u16 = 0x22;
    pub const VK_END: u16 = 0x23;
    pub const VK_HOME: u16 = 0x24;
    pub const VK_LEFT: u16 = 0x25;
    pub const VK_UP: u16 = 0x26;
    pub const VK_RIGHT: u16 = 0x27;
    pub const VK_DOWN: u16 = 0x28;
    pub const VK_INSERT: u16 = 0x2D;
    pub const VK_DELETE: u16 = 0x2E;
    pub const VK_0: u16 = 0x30;
    pub const VK_1: u16 = 0x31;
    pub const VK_2: u16 = 0x32;
    pub const VK_3: u16 = 0x33;
    pub const VK_4: u16 = 0x34;
    pub const VK_5: u16 = 0x35;
    pub const VK_6: u16 = 0x36;
    pub const VK_7: u16 = 0x37;
    pub const VK_8: u16 = 0x38;
    pub const VK_9: u16 = 0x39;
    pub const VK_A: u16 = 0x41;
    pub const VK_B: u16 = 0x42;
    pub const VK_C: u16 = 0x43;
    pub const VK_D: u16 = 0x44;
    pub const VK_E: u16 = 0x45;
    pub const VK_F: u16 = 0x46;
    pub const VK_G: u16 = 0x47;
    pub const VK_H: u16 = 0x48;
    pub const VK_I: u16 = 0x49;
    pub const VK_J: u16 = 0x4A;
    pub const VK_K: u16 = 0x4B;
    pub const VK_L: u16 = 0x4C;
    pub const VK_M: u16 = 0x4D;
    pub const VK_N: u16 = 0x4E;
    pub const VK_O: u16 = 0x4F;
    pub const VK_P: u16 = 0x50;
    pub const VK_Q: u16 = 0x51;
    pub const VK_R: u16 = 0x52;
    pub const VK_S: u16 = 0x53;
    pub const VK_T: u16 = 0x54;
    pub const VK_U: u16 = 0x55;
    pub const VK_V: u16 = 0x56;
    pub const VK_W: u16 = 0x57;
    pub const VK_X: u16 = 0x58;
    pub const VK_Y: u16 = 0x59;
    pub const VK_Z: u16 = 0x5A;
    pub const VK_F1: u16 = 0x70;
    pub const VK_F2: u16 = 0x71;
    pub const VK_F3: u16 = 0x72;
    pub const VK_F4: u16 = 0x73;
    pub const VK_F5: u16 = 0x74;
    pub const VK_F6: u16 = 0x75;
    pub const VK_F7: u16 = 0x76;
    pub const VK_F8: u16 = 0x77;
    pub const VK_F9: u16 = 0x78;
    pub const VK_F10: u16 = 0x79;
    pub const VK_F11: u16 = 0x7A;
    pub const VK_F12: u16 = 0x7B;
}

/// System metrics indices for GetSystemMetrics.
pub mod system_metrics {
    pub const SM_CXSCREEN: i32 = 0;
    pub const SM_CYSCREEN: i32 = 1;
    pub const SM_CXVSCROLL: i32 = 2;
    pub const SM_CYHSCROLL: i32 = 3;
    pub const SM_CYCAPTION: i32 = 4;
    pub const SM_CXBORDER: i32 = 5;
    pub const SM_CYBORDER: i32 = 6;
    pub const SM_CXDLGFRAME: i32 = 7;
    pub const SM_CYDLGFRAME: i32 = 8;
    pub const SM_CYVTHUMB: i32 = 9;
    pub const SM_CXHTHUMB: i32 = 10;
    pub const SM_CXICON: i32 = 11;
    pub const SM_CYICON: i32 = 12;
    pub const SM_CXCURSOR: i32 = 13;
    pub const SM_CYCURSOR: i32 = 14;
    pub const SM_CYMENU: i32 = 15;
    pub const SM_CXFULLSCREEN: i32 = 16;
    pub const SM_CYFULLSCREEN: i32 = 17;
    pub const SM_CYKANJIWINDOW: i32 = 18;
    pub const SM_MOUSEPRESENT: i32 = 19;
    pub const SM_CYVSCROLL: i32 = 20;
    pub const SM_CXHSCROLL: i32 = 21;
    pub const SM_DEBUG: i32 = 22;
    pub const SM_SWAPBUTTON: i32 = 23;
    pub const SM_CXMIN: i32 = 28;
    pub const SM_CYMIN: i32 = 29;
    pub const SM_CXSIZE: i32 = 30;
    pub const SM_CYSIZE: i32 = 31;
    pub const SM_CXFRAME: i32 = 32;
    pub const SM_CYFRAME: i32 = 33;
    pub const SM_CXMINTRACK: i32 = 34;
    pub const SM_CYMINTRACK: i32 = 35;
    pub const SM_CXDOUBLECLK: i32 = 36;
    pub const SM_CYDOUBLECLK: i32 = 37;
    pub const SM_CXICONSPACING: i32 = 38;
    pub const SM_CYICONSPACING: i32 = 39;
    pub const SM_MENUDROPALIGNMENT: i32 = 40;
    pub const SM_PENWINDOWS: i32 = 41;
    pub const SM_DBCSENABLED: i32 = 42;
    pub const SM_CMOUSEBUTTONS: i32 = 43;
    pub const SM_SECURE: i32 = 44;
    pub const SM_CXEDGE: i32 = 45;
    pub const SM_CYEDGE: i32 = 46;
    pub const SM_CXMINSPACING: i32 = 47;
    pub const SM_CYMINSPACING: i32 = 48;
    pub const SM_CXSMICON: i32 = 49;
    pub const SM_CYSMICON: i32 = 50;
    pub const SM_CYSMCAPTION: i32 = 51;
    pub const SM_CXSMSIZE: i32 = 52;
    pub const SM_CYSMSIZE: i32 = 53;
    pub const SM_CXMENUSIZE: i32 = 54;
    pub const SM_CYMENUSIZE: i32 = 55;
    pub const SM_ARRANGE: i32 = 56;
    pub const SM_CXMINIMIZED: i32 = 57;
    pub const SM_CYMINIMIZED: i32 = 58;
    pub const SM_CXMAXTRACK: i32 = 59;
    pub const SM_CYMAXTRACK: i32 = 60;
    pub const SM_CXMAXIMIZED: i32 = 61;
    pub const SM_CYMAXIMIZED: i32 = 62;
    pub const SM_NETWORK: i32 = 63;
    pub const SM_CLEANBOOT: i32 = 67;
    pub const SM_CXDRAG: i32 = 68;
    pub const SM_CYDRAG: i32 = 69;
    pub const SM_SHOWSOUNDS: i32 = 70;
    pub const SM_CXMENUCHECK: i32 = 71;
    pub const SM_CYMENUCHECK: i32 = 72;
    pub const SM_SLOWMACHINE: i32 = 73;
    pub const SM_MIDEASTENABLED: i32 = 74;
    pub const SM_MOUSEWHEELPRESENT: i32 = 75;
    pub const SM_XVIRTUALSCREEN: i32 = 76;
    pub const SM_YVIRTUALSCREEN: i32 = 77;
    pub const SM_CXVIRTUALSCREEN: i32 = 78;
    pub const SM_CYVIRTUALSCREEN: i32 = 79;
    pub const SM_CMONITORS: i32 = 80;
    pub const SM_SAMEDISPLAYFORMAT: i32 = 81;
    pub const SM_IMMENABLED: i32 = 82;
    pub const SM_CXFOCUSBORDER: i32 = 83;
    pub const SM_CYFOCUSBORDER: i32 = 84;
    pub const SM_TABLETPC: i32 = 86;
    pub const SM_MEDIACENTER: i32 = 87;
    pub const SM_STARTER: i32 = 88;
    pub const SM_SERVERR2: i32 = 89;
    pub const SM_MOUSEHORIZONTALWHEELPRESENT: i32 = 91;
    pub const SM_CXPADDEDBORDER: i32 = 92;
    pub const SM_DIGITIZER: i32 = 94;
    pub const SM_MAXIMUMTOUCHES: i32 = 95;
    pub const SM_REMOTESESSION: i32 = 0x1000;
    pub const SM_SHUTTINGDOWN: i32 = 0x2000;
    pub const SM_REMOTECONTROL: i32 = 0x2001;
}

/// Window long index values for SetWindowLong/GetWindowLong.
///
/// These indices specify which attribute of a window to retrieve or modify.
/// Negative values reference per-window data, while non-negative values
/// reference per-class or dialog-specific data.
pub mod window_long_index {
    pub const GWL_STYLE: i32 = -16;
    pub const GWL_EXSTYLE: i32 = -20;
    pub const GWL_ID: i32 = -12;
    pub const GWLP_WNDPROC: i32 = -4;
    pub const GWLP_HINSTANCE: i32 = -6;
    pub const GWLP_HWNDPARENT: i32 = -8;
    pub const GWLP_USERDATA: i32 = -21;
    pub const DWLP_MSGRESULT: i32 = 0;
    pub const DWLP_DLGPROC: i32 = 4;
    pub const DWLP_USER: i32 = 8;
}

/// Handle into the AtomTable for registered window classes.
pub type ATOM = u16;

/// Pointer-sized Windows parameter/result aliases.
pub type WPARAM = usize;
pub type LPARAM = isize;
pub type LRESULT = isize;

// ---------------------------------------------------------------------------
// MSG structure
// ---------------------------------------------------------------------------

/// The Windows MSG structure used by GetMessage/DispatchMessage.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Msg {
    pub hwnd: HWND,
    pub message: u32,
    pub w_param: WPARAM,
    pub l_param: LPARAM,
    pub time: u32,
    pub pt: crate::windows::Point,
}

/// The Win32 `ACCEL` structure used by accelerator tables.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Accel {
    pub f_virt: u8,
    pub key: u16,
    pub cmd: u16,
}

// ---------------------------------------------------------------------------
// WNDCLASSEX structure
// ---------------------------------------------------------------------------

/// The Windows WNDCLASSA structure (ANSI version).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WndClassA {
    pub style: u32,
    pub lpfn_wnd_proc: usize,
    pub cb_cls_extra: i32,
    pub cb_wnd_extra: i32,
    pub h_instance: usize,
    pub h_icon: usize,
    pub h_cursor: usize,
    pub hbr_background: usize,
    pub lpsz_menu_name: *const u8,
    pub lpsz_class_name: *const u8,
}

/// The Windows WNDCLASSW structure (Unicode version).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WndClassW {
    pub style: u32,
    pub lpfn_wnd_proc: usize,
    pub cb_cls_extra: i32,
    pub cb_wnd_extra: i32,
    pub h_instance: usize,
    pub h_icon: usize,
    pub h_cursor: usize,
    pub hbr_background: usize,
    pub lpsz_menu_name: *const u16,
    pub lpsz_class_name: *const u16,
}

/// The Windows WNDCLASSEXA structure (ANSI version).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WndClassExA {
    pub cb_size: u32,
    pub style: u32,
    pub lpfn_wnd_proc: usize,
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
    pub lpfn_wnd_proc: usize,
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
    pub hwnd: HWND,
    pub class_name: String,
    pub title: String,
    pub style: u32,
    pub ex_style: u32,
    pub rect: crate::windows::Rect,
    pub client_rect: crate::windows::Rect,
    pub parent: HWND,
    pub visible: bool,
    pub enabled: bool,
    pub wnd_proc: usize,
    pub user_data: usize,
    pub menu: Option<HMENU>,
    pub system_menu: Option<HMENU>,
}

/// Global window state manager.
pub struct WindowManager {
    windows: Mutex<HashMap<HWND, WindowState>>,
    control_ids: Mutex<HashMap<HWND, i32>>,
    next_hwnd: Mutex<usize>,
    focused_hwnd: Mutex<HWND>,
    foreground_hwnd: Mutex<HWND>,
    last_active_popup: Mutex<HashMap<HWND, HWND>>,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            windows: Mutex::new(HashMap::new()),
            control_ids: Mutex::new(HashMap::new()),
            next_hwnd: Mutex::new(0x10000),
            focused_hwnd: Mutex::new(HWND::NULL),
            foreground_hwnd: Mutex::new(HWND::NULL),
            last_active_popup: Mutex::new(HashMap::new()),
        }
    }

    pub fn get_focus(&self) -> HWND {
        *self.focused_hwnd.lock().unwrap()
    }

    pub fn set_focus(&self, hwnd: HWND) -> HWND {
        let mut focused = self.focused_hwnd.lock().unwrap();
        let prev = *focused;
        *focused = hwnd;
        prev
    }

    pub fn get_foreground(&self) -> HWND {
        *self.foreground_hwnd.lock().unwrap()
    }

    pub fn set_foreground(&self, hwnd: HWND) -> HWND {
        let mut foreground = self.foreground_hwnd.lock().unwrap();
        let prev = *foreground;
        *foreground = hwnd;
        prev
    }

    pub fn create_window(&self, state: WindowState) -> HWND {
        let mut windows = self.windows.lock().unwrap();
        let mut next_hwnd = self.next_hwnd.lock().unwrap();

        let hwnd = HWND::from_raw(*next_hwnd);
        *next_hwnd += 1;

        let mut state = state;
        state.hwnd = hwnd;
        windows.insert(hwnd, state);

        hwnd
    }

    pub fn get_window(&self, hwnd: HWND) -> Option<WindowState> {
        let windows = self.windows.lock().unwrap();
        windows.get(&hwnd).cloned()
    }

    pub fn update_window<F>(&self, hwnd: HWND, f: F) -> bool
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

    pub fn destroy_window(&self, hwnd: HWND) -> bool {
        let mut windows = self.windows.lock().unwrap();
        let removed = windows.remove(&hwnd).is_some();
        drop(windows);

        if removed {
            let mut control_ids = self.control_ids.lock().unwrap();
            control_ids.remove(&hwnd);
        }

        removed
    }

    pub fn set_control_id(&self, hwnd: HWND, control_id: i32) -> bool {
        let windows = self.windows.lock().unwrap();
        if !windows.contains_key(&hwnd) {
            return false;
        }
        drop(windows);

        let mut control_ids = self.control_ids.lock().unwrap();
        control_ids.insert(hwnd, control_id);
        true
    }

    pub fn get_control_id(&self, hwnd: HWND) -> Option<i32> {
        let control_ids = self.control_ids.lock().unwrap();
        control_ids.get(&hwnd).copied()
    }

    pub fn find_child_by_control_id(&self, parent: HWND, control_id: i32) -> HWND {
        let windows = self.windows.lock().unwrap();
        let control_ids = self.control_ids.lock().unwrap();

        let mut found = HWND::NULL;
        for (&hwnd, state) in windows.iter() {
            if state.parent != parent {
                continue;
            }

            if control_ids.get(&hwnd).copied() != Some(control_id) {
                continue;
            }

            if found.is_null() || hwnd.as_raw() > found.as_raw() {
                found = hwnd;
            }
        }

        found
    }

    pub fn record_popup_activation(&self, popup: HWND) {
        if popup.is_null() {
            return;
        }
        let windows = self.windows.lock().unwrap();
        let owner = windows.get(&popup).map(|s| s.parent).unwrap_or(HWND::NULL);
        drop(windows);

        if !owner.is_null() {
            let mut map = self.last_active_popup.lock().unwrap();
            map.insert(owner, popup);
        }
    }

    pub fn get_last_active_popup(&self, owner: HWND) -> HWND {
        let map = self.last_active_popup.lock().unwrap();
        map.get(&owner).copied().unwrap_or(HWND::NULL)
    }

    pub fn child_window_from_point(&self, parent: HWND, pt: crate::windows::Point) -> HWND {
        let windows = self.windows.lock().unwrap();

        if !windows.contains_key(&parent) {
            return HWND::NULL;
        }

        let mut found: HWND = HWND::NULL;
        for (&hwnd, state) in windows.iter() {
            if state.parent != parent {
                continue;
            }
            if !state.visible || !state.enabled {
                continue;
            }
            if !state.rect.contains_point(pt.x, pt.y) {
                continue;
            }
            if found.is_null() || hwnd.as_raw() > found.as_raw() {
                found = hwnd;
            }
        }

        if found.is_null() { parent } else { found }
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

/// A message queue helper for a single thread.
///
/// Runtime queue ownership lives in the shared user32 layer
/// (`rine-common-user32`), which provides the canonical thread queue used by
/// GetMessage/PeekMessage flows.
#[derive(Debug)]
pub struct MessageQueue {
    state: Mutex<MessageQueueState>,
    wake: Condvar,
}

#[derive(Debug, Default)]
struct MessageQueueState {
    messages: Vec<Msg>,
    quit: bool,
    quit_code: i32,
}

impl MessageQueue {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(MessageQueueState::default()),
            wake: Condvar::new(),
        }
    }

    pub fn post_message(&self, msg: Msg) {
        {
            let mut state = self.state.lock().unwrap();
            state.messages.push(msg);
        }
        self.wake.notify_one();
    }

    pub fn post_quit(&self, exit_code: i32) {
        {
            let mut state = self.state.lock().unwrap();
            state.quit = true;
            state.quit_code = exit_code;
        }
        self.wake.notify_all();
    }

    pub fn get_message(&self, msg: &mut Msg) -> bool {
        let mut state = self.state.lock().unwrap();
        loop {
            if state.quit {
                msg.message = window_message::WM_QUIT;
                msg.w_param = state.quit_code as usize;
                return false;
            }

            if !state.messages.is_empty() {
                *msg = state.messages.remove(0);
                return true;
            }

            state = self.wake.wait(state).unwrap();
        }
    }

    pub fn peek_message(&self, msg: &mut Msg, remove: bool) -> bool {
        let mut state = self.state.lock().unwrap();

        if state.quit {
            msg.message = window_message::WM_QUIT;
            msg.w_param = state.quit_code as usize;
            return true;
        }

        if state.messages.is_empty() {
            return false;
        }

        *msg = state.messages[0];
        if remove {
            state.messages.remove(0);
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

pub static WINDOW_CLASS_REGISTRY: LazyLock<Arc<WindowClassRegistry>> =
    LazyLock::new(|| Arc::new(WindowClassRegistry::new()));

pub static WINDOW_MANAGER: LazyLock<Arc<WindowManager>> =
    LazyLock::new(|| Arc::new(WindowManager::new()));

// ---------------------------------------------------------------------------
// Unit Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::windows::Point;
    use crate::windows::Rect;

    use super::*;

    // ── Hwnd ──────────────────────────────────────────────────────

    #[test]
    fn hwnd_null() {
        assert!(HWND::NULL.is_null());
        assert_eq!(HWND::NULL.as_raw(), 0);
    }

    #[test]
    fn hwnd_from_raw() {
        let hwnd = HWND::from_raw(0x1234);
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
        assert!(!registry.unregister("TempClass"));
    }

    // ── WindowManager ──────────────────────────────────────────────

    #[test]
    fn window_manager_create_window() {
        let manager = WindowManager::new();
        let state = WindowState {
            hwnd: HWND::NULL,
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
            parent: HWND::NULL,
            visible: true,
            enabled: true,
            wnd_proc: 0x3000,
            user_data: 0,
            menu: None,
            system_menu: None,
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
            hwnd: HWND::NULL,
            class_name: "TestClass".into(),
            title: "Original".into(),
            style: window_style::WS_OVERLAPPEDWINDOW,
            ex_style: 0,
            rect: Rect::default(),
            client_rect: Rect::default(),
            parent: HWND::NULL,
            visible: false,
            enabled: true,
            wnd_proc: 0,
            user_data: 0,
            menu: None,
            system_menu: None,
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
            hwnd: HWND::NULL,
            class_name: "TestClass".into(),
            title: "Temp".into(),
            style: 0,
            ex_style: 0,
            rect: Rect::default(),
            client_rect: Rect::default(),
            parent: HWND::NULL,
            visible: false,
            enabled: true,
            wnd_proc: 0,
            user_data: 0,
            menu: None,
            system_menu: None,
        };

        let hwnd = manager.create_window(state);
        assert!(manager.get_window(hwnd).is_some());

        assert!(manager.destroy_window(hwnd));
        assert!(manager.get_window(hwnd).is_none());
        assert!(!manager.destroy_window(hwnd));
    }

    // ── MessageQueue ───────────────────────────────────────────────

    #[test]
    fn message_queue_post_and_peek() {
        let queue = MessageQueue::new();
        let msg = Msg {
            hwnd: HWND::from_raw(0x1000),
            message: window_message::WM_PAINT,
            w_param: 0,
            l_param: 0,
            time: 1000,
            pt: Point { x: 10, y: 20 },
        };

        queue.post_message(msg);

        let mut retrieved = Msg {
            hwnd: HWND::NULL,
            message: 0,
            w_param: 0,
            l_param: 0,
            time: 0,
            pt: Point { x: 0, y: 0 },
        };

        assert!(queue.peek_message(&mut retrieved, false));
        assert_eq!(retrieved.message, window_message::WM_PAINT);
        assert_eq!(retrieved.hwnd.as_raw(), 0x1000);

        assert!(queue.peek_message(&mut retrieved, true));
        assert!(!queue.peek_message(&mut retrieved, false));
    }

    #[test]
    fn message_queue_post_quit() {
        let queue = MessageQueue::new();
        queue.post_quit(42);

        let mut msg = Msg {
            hwnd: HWND::NULL,
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
}
