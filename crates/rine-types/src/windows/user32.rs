//! Windows window management types (user32.dll).
//!
//! This module defines structures and constants for Win32 window management,
//! message handling, and the window class system.

use core::fmt;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::{Arc, Condvar, Mutex};

use crate::handles::HMENU;

mod constants;
mod manager;
mod message_queue;
mod registry;

pub use constants::*;
pub use manager::*;
pub use message_queue::*;
pub use registry::*;

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

pub static WINDOW_CLASS_REGISTRY: LazyLock<Arc<WindowClassRegistry>> =
    LazyLock::new(|| Arc::new(WindowClassRegistry::new()));

pub static WINDOW_MANAGER: LazyLock<Arc<WindowManager>> =
    LazyLock::new(|| Arc::new(WindowManager::new()));

#[cfg(test)]
mod tests {
    use super::*;

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
}
