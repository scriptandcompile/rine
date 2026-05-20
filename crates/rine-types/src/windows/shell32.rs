//! Windows Shell types (shell32.dll).
//!
//! This module defines structures and constants for Windows Shell operations.

use core::fmt;

use crate::errors::BOOL;
use crate::windows::Point;

/// A shell drag-and-drop handle (HDROP).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct HDROP(usize);

impl HDROP {
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

impl fmt::Debug for HDROP {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HDROP({:#x})", self.0)
    }
}

/// The Windows DROPFILES structure used by WM_DROPFILES/HDROP.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DropFiles {
    pub p_files: u32,
    pub pt: Point,
    pub f_nc: BOOL,
    pub f_wide: BOOL,
}
