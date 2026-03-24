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
    pub const OBJECT_NAME_NOT_FOUND: Self = Self(0xC000_0034);
    pub const OBJECT_NAME_INVALID: Self = Self(0xC000_0033);
    pub const OBJECT_PATH_NOT_FOUND: Self = Self(0xC000_003A);
    pub const ACCESS_DENIED: Self = Self(0xC000_0022);
    pub const NO_MORE_FILES: Self = Self(0x8000_0006);
    pub const END_OF_FILE: Self = Self(0xC000_0011);
    pub const NO_SUCH_FILE: Self = Self(0xC000_000F);

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

// Win32 error codes (GetLastError / SetLastError values).
pub const ERROR_SUCCESS: u32 = 0;
pub const ERROR_FILE_NOT_FOUND: u32 = 2;
pub const ERROR_PATH_NOT_FOUND: u32 = 3;
pub const ERROR_ACCESS_DENIED: u32 = 5;
pub const ERROR_INVALID_HANDLE: u32 = 6;
pub const ERROR_NO_MORE_FILES: u32 = 18;
pub const ERROR_ALREADY_EXISTS: u32 = 183;
