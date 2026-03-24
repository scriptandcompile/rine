//! Windows HANDLE types and standard handle constants.

use core::fmt;

/// A Windows HANDLE value, stored as an `isize` to match the Windows ABI
/// (where `HANDLE` is a pointer-sized signed value, and pseudo-handles are
/// negative).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Handle(isize);

/// Well-known pseudo-handle returned by `GetStdHandle`.
pub const STD_INPUT_HANDLE: u32 = 0xFFFF_FFF6; // (DWORD)-10
pub const STD_OUTPUT_HANDLE: u32 = 0xFFFF_FFF5; // (DWORD)-11
pub const STD_ERROR_HANDLE: u32 = 0xFFFF_FFF4; // (DWORD)-12

/// The invalid handle sentinel (`INVALID_HANDLE_VALUE`).
pub const INVALID_HANDLE_VALUE: Handle = Handle(-1);

impl Handle {
    pub const NULL: Self = Self(0);

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
}

impl fmt::Debug for Handle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HANDLE({:#x})", self.0)
    }
}

/// An HMODULE value (base address of a loaded module).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct HModule(usize);

impl HModule {
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

impl fmt::Debug for HModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HMODULE({:#x})", self.0)
    }
}

/// Map a Windows standard-handle constant to a Linux file descriptor.
pub fn std_handle_to_fd(nstd_handle: u32) -> Option<i32> {
    match nstd_handle {
        STD_INPUT_HANDLE => Some(0),
        STD_OUTPUT_HANDLE => Some(1),
        STD_ERROR_HANDLE => Some(2),
        _ => None,
    }
}

/// Encode a Linux file descriptor as a Windows HANDLE.
///
/// We use the simple scheme: `HANDLE = fd + HANDLE_FD_BASE` so that the
/// three standard handles don't collide with NULL or INVALID_HANDLE_VALUE.
const HANDLE_FD_BASE: isize = 0x1000;

pub fn fd_to_handle(fd: i32) -> Handle {
    Handle::from_raw(fd as isize + HANDLE_FD_BASE)
}

pub fn handle_to_fd(h: Handle) -> Option<i32> {
    let raw = h.as_raw();
    if raw >= HANDLE_FD_BASE {
        Some((raw - HANDLE_FD_BASE) as i32)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn std_handle_mapping() {
        assert_eq!(std_handle_to_fd(STD_INPUT_HANDLE), Some(0));
        assert_eq!(std_handle_to_fd(STD_OUTPUT_HANDLE), Some(1));
        assert_eq!(std_handle_to_fd(STD_ERROR_HANDLE), Some(2));
        assert_eq!(std_handle_to_fd(0), None);
    }

    #[test]
    fn fd_handle_roundtrip() {
        for fd in 0..10 {
            let h = fd_to_handle(fd);
            assert_eq!(handle_to_fd(h), Some(fd));
        }
    }

    #[test]
    fn invalid_handle() {
        assert!(INVALID_HANDLE_VALUE.is_invalid());
        assert!(!Handle::NULL.is_invalid());
    }
}
