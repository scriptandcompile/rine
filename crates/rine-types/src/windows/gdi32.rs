//! Windows GDI types (gdi32.dll).
//!
//! This module defines structures and constants for GDI graphics operations.

use core::fmt;

/// A Windows device context handle (HDC).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct HDC(usize);

impl HDC {
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

impl fmt::Debug for HDC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HDC({:#x})", self.0)
    }
}
