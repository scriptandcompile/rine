//! Windows error codes and NTSTATUS values used by rine DLL implementations.

/// NTSTATUS — 32-bit status code returned by NT kernel functions.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct NtStatus(pub u32);

impl NtStatus {
    pub const SUCCESS: Self = Self(0x0000_0000);
    pub const INVALID_HANDLE: Self = Self(0xC000_0008);
    pub const INVALID_PARAMETER: Self = Self(0xC000_000D);
    pub const NOT_IMPLEMENTED: Self = Self(0xC000_0002);

    #[inline]
    pub const fn is_success(self) -> bool {
        (self.0 as i32) >= 0
    }
}

impl core::fmt::Debug for NtStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "NTSTATUS({:#010x})", self.0)
    }
}

/// Win32 BOOL — 0 means FALSE, non-zero means TRUE.
pub type WinBool = i32;
pub const TRUE: WinBool = 1;
pub const FALSE: WinBool = 0;
