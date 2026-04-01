//! Miscellaneous Windows structures used by DLL implementations.

/// IO_STATUS_BLOCK — returned by NT I/O functions.
#[repr(C)]
pub struct IoStatusBlock {
    /// NTSTATUS or pointer (union in Windows; we use the status variant).
    pub status: u32,
    /// Number of bytes transferred.
    pub information: usize,
}

// ---------------------------------------------------------------------------
// Process creation structures
// ---------------------------------------------------------------------------

/// `STARTUPINFOA` — startup parameters passed to `CreateProcessA`.
///
/// Most fields are ignored by rine; we define the full layout so the PE
/// code's pointer arithmetic lands correctly.
#[repr(C)]
pub struct StartupInfoA {
    pub cb: u32,
    pub reserved: *mut u8,
    pub desktop: *mut u8,
    pub title: *mut u8,
    pub x: u32,
    pub y: u32,
    pub x_size: u32,
    pub y_size: u32,
    pub x_count_chars: u32,
    pub y_count_chars: u32,
    pub fill_attribute: u32,
    pub flags: u32,
    pub show_window: u16,
    pub cb_reserved2: u16,
    pub reserved2: *mut u8,
    pub std_input: isize,
    pub std_output: isize,
    pub std_error: isize,
}

/// `STARTUPINFOW` — wide variant.
#[repr(C)]
pub struct StartupInfoW {
    pub cb: u32,
    pub reserved: *mut u16,
    pub desktop: *mut u16,
    pub title: *mut u16,
    pub x: u32,
    pub y: u32,
    pub x_size: u32,
    pub y_size: u32,
    pub x_count_chars: u32,
    pub y_count_chars: u32,
    pub fill_attribute: u32,
    pub flags: u32,
    pub show_window: u16,
    pub cb_reserved2: u16,
    pub reserved2: *mut u8,
    pub std_input: isize,
    pub std_output: isize,
    pub std_error: isize,
}

/// `PROCESS_INFORMATION` — filled in by `CreateProcessA/W`.
#[repr(C)]
pub struct ProcessInformation {
    pub process: isize,
    pub thread: isize,
    pub process_id: u32,
    pub thread_id: u32,
}
