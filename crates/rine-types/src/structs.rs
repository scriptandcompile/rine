//! Miscellaneous Windows structures used by DLL implementations.

/// IO_STATUS_BLOCK — returned by NT I/O functions.
#[repr(C)]
pub struct IoStatusBlock {
    /// NTSTATUS or pointer (union in Windows; we use the status variant).
    pub status: u32,
    /// Number of bytes transferred.
    pub information: usize,
}
