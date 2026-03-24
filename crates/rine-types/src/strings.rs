//! Windows string types and conversion helpers.

use core::fmt;

/// NT UNICODE_STRING — a counted (not null-terminated) wide-character string
/// used throughout the NT kernel API.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct UnicodeString {
    /// Length in **bytes** (not characters), excluding any trailing null.
    pub length: u16,
    /// Maximum length in **bytes** of the buffer.
    pub maximum_length: u16,
    /// Pointer to a UTF-16LE buffer.
    pub buffer: *mut u16,
}

impl UnicodeString {
    pub const fn empty() -> Self {
        Self {
            length: 0,
            maximum_length: 0,
            buffer: core::ptr::null_mut(),
        }
    }
}

impl fmt::Debug for UnicodeString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "UNICODE_STRING(len={}, max={}, buf={:?})",
            self.length, self.maximum_length, self.buffer,
        )
    }
}
