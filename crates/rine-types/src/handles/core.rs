//! Windows HANDLE newtypes: `HANDLE`, `HLOCAL`, `HGLOBAL`, `HFILE`,
//! `HINSTANCE`, `HMENU`, `HACCEL`, `HMODULE`.

use core::fmt;

// ---------------------------------------------------------------------------
// Handle / HModule newtypes
// ---------------------------------------------------------------------------

/// A Windows HANDLE value, stored as an `isize` to match the Windows ABI
/// (where `HANDLE` is a pointer-sized signed value, and pseudo-handles are
/// negative).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct HANDLE(isize);

impl HANDLE {
    pub const NULL: Self = Self(0);
    pub const INVALID: Self = Self(-1);

    #[inline]
    pub const fn from_raw(value: isize) -> Self {
        Self(value)
    }

    #[inline]
    pub const fn as_raw(self) -> isize {
        self.0
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub const fn is_invalid(self) -> bool {
        self.0 == -1
    }

    #[inline]
    pub const fn is_valid(self) -> bool {
        self.0 > 0
    }
}

impl fmt::Debug for HANDLE {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HANDLE({:#x})", self.0)
    }
}

/// A Windows `HLOCAL` value, stored as an `isize` to match the Windows ABI.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct HLOCAL(HANDLE);

impl HLOCAL {
    pub const NULL: Self = Self(HANDLE::NULL);
    pub const INVALID: Self = Self(HANDLE::INVALID);

    #[inline]
    pub const fn from_raw(value: isize) -> Self {
        Self(HANDLE::from_raw(value))
    }

    #[inline]
    pub const fn as_raw(self) -> isize {
        self.0.as_raw()
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    #[inline]
    pub const fn is_invalid(self) -> bool {
        self.0.is_invalid()
    }

    #[inline]
    pub const fn is_valid(self) -> bool {
        self.0.is_valid()
    }
}

impl From<HGLOBAL> for HLOCAL {
    #[inline]
    fn from(handle: HGLOBAL) -> Self {
        Self(HANDLE::from_raw(handle.as_raw()))
    }
}

impl fmt::Debug for HLOCAL {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HLOCAL({:#x})", self.0.as_raw())
    }
}

/// A Windows `HGLOBAL` value, stored as an `isize` to match the Windows ABI.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct HGLOBAL(HANDLE);

impl HGLOBAL {
    pub const NULL: Self = Self(HANDLE::NULL);
    pub const INVALID: Self = Self(HANDLE::INVALID);

    #[inline]
    pub const fn from_raw(value: isize) -> Self {
        Self(HANDLE::from_raw(value))
    }

    #[inline]
    pub const fn as_raw(self) -> isize {
        self.0.as_raw()
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    #[inline]
    pub const fn is_invalid(self) -> bool {
        self.0.is_invalid()
    }

    #[inline]
    pub const fn is_valid(self) -> bool {
        self.0.is_valid()
    }
}

impl From<HLOCAL> for HGLOBAL {
    #[inline]
    fn from(handle: HLOCAL) -> Self {
        Self(HANDLE::from_raw(handle.as_raw()))
    }
}

impl fmt::Debug for HGLOBAL {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HGLOBAL({:#x})", self.0.as_raw())
    }
}

/// A handle to a file from `OpenFile`, not `CreateFile`.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct HFILE(i32);

impl HFILE {
    pub const INVALID: Self = Self(-1);
    pub const NULL: Self = Self(0);

    pub const fn from_raw(value: i32) -> Self {
        Self(value)
    }

    pub const fn as_raw(self) -> i32 {
        self.0
    }

    pub const fn is_invalid(self) -> bool {
        self.0 == -1
    }

    pub const fn is_null(self) -> bool {
        self.0 == 0
    }
}

/// A handle to an instance/module (from `LoadLibrary` and related functions).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct HINSTANCE(HANDLE);

impl HINSTANCE {
    pub const NULL: Self = Self(HANDLE::NULL);
    pub const INVALID: Self = Self(HANDLE::INVALID);

    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    pub const fn is_invalid(self) -> bool {
        self.0.is_invalid()
    }

    pub const fn from_raw(value: usize) -> Self {
        Self(HANDLE::from_raw(value as isize))
    }

    pub const fn as_raw(self) -> usize {
        self.0.as_raw() as usize
    }
}

/// A handle to a menu (from `CreateMenu` and related functions).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct HMENU(HANDLE);

impl HMENU {
    pub const NULL: Self = Self(HANDLE::NULL);

    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// A handle to an accelerator table loaded by user32.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct HACCEL(HANDLE);

impl HACCEL {
    pub const NULL: Self = Self(HANDLE::NULL);

    #[inline]
    pub const fn from_raw(value: usize) -> Self {
        Self(HANDLE::from_raw(value as isize))
    }

    #[inline]
    pub const fn as_raw(self) -> usize {
        self.0.as_raw() as usize
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// An HMODULE value (base address of a loaded module).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct HMODULE(usize);

impl HMODULE {
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

impl fmt::Debug for HMODULE {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HMODULE({:#x})", self.0)
    }
}

// ---------------------------------------------------------------------------
// Unit Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_handle() {
        assert!(HANDLE::INVALID.is_invalid());
        assert!(!HANDLE::NULL.is_invalid());
    }
}
