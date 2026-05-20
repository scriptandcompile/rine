//! Windows file attribute, access, share, and creation disposition constants.

// ---------------------------------------------------------------------------
// File attributes
// ---------------------------------------------------------------------------

pub const FILE_ATTRIBUTE_READONLY: u32 = 0x0000_0001;
pub const FILE_ATTRIBUTE_HIDDEN: u32 = 0x0000_0002;
pub const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x0000_0010;
pub const FILE_ATTRIBUTE_ARCHIVE: u32 = 0x0000_0020;
pub const FILE_ATTRIBUTE_NORMAL: u32 = 0x0000_0080;

// ---------------------------------------------------------------------------
// Generic access rights
// ---------------------------------------------------------------------------

pub const GENERIC_READ: u32 = 0x8000_0000;
pub const GENERIC_WRITE: u32 = 0x4000_0000;
pub const GENERIC_EXECUTE: u32 = 0x2000_0000;
pub const GENERIC_ALL: u32 = 0x1000_0000;

// ---------------------------------------------------------------------------
// File share modes
// ---------------------------------------------------------------------------

pub const FILE_SHARE_READ: u32 = 0x0000_0001;
pub const FILE_SHARE_WRITE: u32 = 0x0000_0002;
pub const FILE_SHARE_DELETE: u32 = 0x0000_0004;

// ---------------------------------------------------------------------------
// File creation disposition
// ---------------------------------------------------------------------------

pub const CREATE_NEW: u32 = 1;
pub const CREATE_ALWAYS: u32 = 2;
pub const OPEN_EXISTING: u32 = 3;
pub const OPEN_ALWAYS: u32 = 4;
pub const TRUNCATE_EXISTING: u32 = 5;

// ---------------------------------------------------------------------------
// File pointer move methods
// ---------------------------------------------------------------------------

pub const FILE_BEGIN: u32 = 0;
pub const FILE_CURRENT: u32 = 1;
pub const FILE_END: u32 = 2;

pub const INVALID_SET_FILE_POINTER: u32 = 0xFFFF_FFFF;
pub const INVALID_FILE_SIZE: u32 = 0xFFFF_FFFF;
