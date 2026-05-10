//! Windows HANDLE types and a global handle table.
//!
//! The handle table maps Windows `HANDLE` values to Linux resources
//! (file descriptors, find-data iterators, etc.).  Handles are allocated
//! from a monotonically increasing counter so they never collide with
//! `NULL` (0) or `INVALID_HANDLE_VALUE` (−1).
//!
//! The three standard I/O handles (stdin/stdout/stderr) are pre-registered
//! in the table by [`HandleTable::init`].

use core::fmt;
use std::collections::HashMap;

use std::sync::Mutex;

use crate::registry::RegistryKeyState;
use crate::threading::{
    EventWaitable, MutexWaitable, ProcessWaitable, SemaphoreWaitable, ThreadWaitable, Waitable,
};
use crate::windows::HWND;

// ---------------------------------------------------------------------------
// Heap state (for HeapCreate handles)
// ---------------------------------------------------------------------------

/// State tracked for a heap created via `HeapCreate`.
///
/// Each heap is backed by the Rust global allocator.  We track outstanding
/// allocations so we can clean up on `HeapDestroy` and implement `HeapSize`.
#[derive(Debug)]
pub struct HeapState {
    /// Outstanding allocations: address → (layout.size, layout.align).
    pub allocations: Mutex<HashMap<usize, (usize, usize)>>,
    /// Default flags passed to `HeapCreate` (e.g. `HEAP_GENERATE_EXCEPTIONS`).
    pub flags: u32,
}

// ---------------------------------------------------------------------------
// Handle / HModule newtypes
// ---------------------------------------------------------------------------

/// A Windows HANDLE value, stored as an `isize` to match the Windows ABI
/// (where `HANDLE` is a pointer-sized signed value, and pseudo-handles are
/// negative).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct HANDLE(isize);

/// Well-known pseudo-handle returned by `GetStdHandle`.
pub const STD_INPUT_HANDLE: u32 = 0xFFFF_FFF6; // (DWORD)-10
pub const STD_OUTPUT_HANDLE: u32 = 0xFFFF_FFF5; // (DWORD)-11
pub const STD_ERROR_HANDLE: u32 = 0xFFFF_FFF4; // (DWORD)-12

impl HANDLE {
    /// The null handle (`NULL`).
    ///
    /// Note that `NULL` is a valid handle value that represents "no object", while `INVALID_HANDLE_VALUE` indicates an error.
    pub const NULL: Self = Self(0);

    /// The invalid handle sentinel (`INVALID_HANDLE_VALUE`).
    ///
    /// Note that `INVALID_HANDLE_VALUE` (−1) indicates an error, while `NULL` (0) is a valid handle value that represents "no object".
    pub const INVALID: Self = Self(-1);

    /// Create a `Handle` from a raw `isize` value, for use in the Windows ABI.
    #[inline]
    pub const fn from_raw(value: isize) -> Self {
        Self(value)
    }

    /// Get the raw `isize` value of this handle, for use in the Windows ABI.
    #[inline]
    pub const fn as_raw(self) -> isize {
        self.0
    }

    /// Check if this handle is `NULL` (0), which is a valid but non-functional handle.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    /// Check if this handle is `INVALID_HANDLE_VALUE` (−1), which indicates an error.
    #[inline]
    pub const fn is_invalid(self) -> bool {
        self.0 == -1
    }

    /// Helper to check if a handle is valid (not NULL and not INVALID_HANDLE_VALUE)
    ///
    /// In Windows conventions, valid handles are positive integers (or zero for NULL),
    /// while negative values indicate errors.  
    ///
    /// This method returns true for valid handles and false for NULL or INVALID_HANDLE_VALUE.
    ///
    /// In Windows conventions:
    /// - Handle(0) = NULL (valid but represents no object)
    /// - Handle(-1) = INVALID_HANDLE_VALUE (indicates failure)
    /// - Valid handles are positive integers >= 1
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

/// A Windows `HLOCAL` value, stored as an `isize` to match the Windows ABI
/// (where `HLOCAL` is a pointer-sized signed value, and pseudo-handles are
/// negative).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct HLOCAL(HANDLE);

impl HLOCAL {
    /// The null handle (`NULL`).
    ///
    /// Note that `NULL` is a valid handle value that represents "no object", while `INVALID_HANDLE_VALUE` indicates an error.
    pub const NULL: Self = Self(HANDLE::NULL);

    /// The invalid handle sentinel (`INVALID_HANDLE_VALUE`).
    ///
    /// Note that `INVALID_HANDLE_VALUE` (−1) indicates an error, while `NULL` (0) is a valid handle value that represents "no object".
    pub const INVALID: Self = Self(HANDLE::INVALID);

    /// Create a `HLOCAL` from a raw `isize` value, for use in the Windows ABI.
    #[inline]
    pub const fn from_raw(value: isize) -> Self {
        Self(HANDLE::from_raw(value))
    }

    /// Get the raw `isize` value of this handle, for use in the Windows ABI.
    #[inline]
    pub const fn as_raw(self) -> isize {
        self.0.as_raw()
    }

    /// Check if this handle is `NULL` (0), which is a valid but non-functional handle.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Check if this handle is `INVALID_HANDLE_VALUE` (−1), which indicates an error.
    #[inline]
    pub const fn is_invalid(self) -> bool {
        self.0.is_invalid()
    }

    /// Helper to check if a handle is valid (not NULL and not INVALID_HANDLE_VALUE)
    ///
    /// In Windows conventions, valid handles are positive integers (or zero for NULL),
    /// while negative values indicate errors.  
    ///
    /// This method returns true for valid handles and false for NULL or INVALID_HANDLE_VALUE.
    ///
    /// In Windows conventions:
    /// - Handle(0) = NULL (valid but represents no object)
    /// - Handle(-1) = INVALID_HANDLE_VALUE (indicates failure)
    /// - Valid handles are positive integers >= 1
    #[inline]
    pub const fn is_valid(self) -> bool {
        self.0.is_valid()
    }
}

impl fmt::Debug for HLOCAL {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HLOCAL({:#x})", self.0.as_raw())
    }
}

/// A handle to a file from `OpenFile`, not `CreateFile`.
/// `HFILE` is a legacy API file handle instead of a `HANDLE`.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct HFILE(i32);

impl HFILE {
    /// The invalid HFILE sentinel (`HFILE_ERROR`), returned by `OpenFile` on error.
    pub const INVALID: Self = Self(-1);

    /// The null file handle (`NULL`).
    pub const NULL: Self = Self(0);

    /// Create an `HFile` from a raw `i32` value, for use in the Windows ABI.
    ///
    /// # Return
    /// An `HFile` wrapping the given raw value.
    pub const fn from_raw(value: i32) -> Self {
        Self(value)
    }

    /// Get the raw `i32` value of this HFILE, for use in the Windows ABI.
    ///
    /// # Return
    /// The raw `i32` value of this HFILE.
    ///
    /// # Notes
    /// Valid values are non-negative integers representing file handles,
    /// while `INVALID` (-1) indicates an error and `NULL` (0) represents "no file".
    pub const fn as_raw(self) -> i32 {
        self.0
    }

    /// Check if this HFILE is `INVALID` (-1), which indicates an error.
    ///
    /// # Return
    /// `true` if this HFILE is `INVALID`, `false` otherwise.
    ///
    /// # Notes
    /// `INVALID` (-1) indicates an error, while `NULL` (0) is a valid HFILE value that represents "no file".
    pub const fn is_invalid(self) -> bool {
        self.0 == -1
    }

    /// Check if this handle is `NULL` (0), which is a valid but non-functional handle.
    ///
    /// # Return
    /// `true` if this handle is `NULL`, `false` otherwise.
    ///
    /// # Notes
    /// `NULL` (0) is a valid handle value that represents "no object", while `INVALID` (-1) indicates an error.
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }
}

/// A handle to an instance/module (from `LoadLibrary` and related functions).
/// This is the base address of the module in memory.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct HINSTANCE(HANDLE);

impl HINSTANCE {
    /// The null instance handle (`NULL`).
    pub const NULL: Self = Self(HANDLE::NULL);
    /// The invalid instance handle sentinel (`INVALID`).
    pub const INVALID: Self = Self(HANDLE::INVALID);

    /// Check if this handle is `NULL` (0), which is a valid but non-functional handle.
    ///
    /// # Return
    /// `true` if this handle is `NULL`, `false` otherwise.
    ///
    /// # Notes
    /// `NULL` (0) is a valid handle value that represents "no object", while `INVALID` (-1) indicates an error.
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Check if this handle is `INVALID` (-1), which indicates an error.
    ///
    /// # Return
    /// `true` if this handle is `INVALID`, `false` otherwise.
    ///
    /// # Notes
    /// `INVALID` (-1) indicates an error, while `NULL` (0) is a valid handle value that represents "no object".
    pub const fn is_invalid(self) -> bool {
        self.0.is_invalid()
    }

    /// Create an `HINSTANCE` from a raw `usize` value, for use in the Windows ABI.
    ///
    /// # Arguments
    /// * `value` - The raw `usize` value representing the base address of the module.
    ///
    /// # Return
    /// An `HINSTANCE` wrapping the given raw value.
    ///
    /// # Notes
    /// Valid values are non-zero addresses representing loaded modules, while `NULL` (0) is a valid handle value that
    /// represents "no object", and `INVALID` (-1) indicates an error.
    pub const fn from_raw(value: usize) -> Self {
        Self(HANDLE::from_raw(value as isize))
    }

    /// Get the raw `usize` value of this `HINSTANCE`, for use in the Windows ABI.
    ///
    /// # Return
    /// The raw `usize` value of this `HINSTANCE`, representing the base address of the module.
    ///
    /// # Notes
    /// Valid values are non-zero addresses representing loaded modules, while `NULL` (0) is a valid handle value that
    /// represents "no object", and `INVALID` (-1) indicates an error.
    /// The returned value is the base address of the module in memory.
    pub const fn as_raw(self) -> usize {
        self.0.as_raw() as usize
    }
}

/// A handle to a menu (from `CreateMenu` and related functions).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct HMENU(HANDLE);

impl HMENU {
    /// The null menu handle (`NULL`).
    pub const NULL: Self = Self(HANDLE::NULL);

    /// Check if this handle is `NULL` (0), which is a valid but non-functional handle.
    ///
    /// # Return
    /// `true` if this handle is `NULL`, `false` otherwise.
    ///
    /// # Notes
    /// `NULL` (0) is a valid handle value that represents "no object".  
    /// There is no standard `INVALID` value for menus, so we only check for `NULL`.
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
// Standard-handle helpers (convenience — still used by GetStdHandle)
// ---------------------------------------------------------------------------

/// Map a Windows standard-handle constant to a Linux file descriptor.
pub fn std_handle_to_fd(nstd_handle: u32) -> Option<i32> {
    match nstd_handle {
        STD_INPUT_HANDLE => Some(0),
        STD_OUTPUT_HANDLE => Some(1),
        STD_ERROR_HANDLE => Some(2),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Handle table — resource tracking
// ---------------------------------------------------------------------------

/// What a handle points to.
#[derive(Debug)]
pub enum HandleEntry {
    /// A Linux file descriptor (from `open`, `socket`, …).
    File(i32),
    /// A `FindFirstFile` directory search iterator.
    FindData(FindDataState),
    /// A thread created by `CreateThread`.
    Thread(ThreadWaitable),
    /// An event object created by `CreateEvent`.
    Event(EventWaitable),
    /// A child process created by `CreateProcess`.
    Process(ProcessWaitable),
    /// A mutex object created by `CreateMutex`.
    Mutex(MutexWaitable),
    /// A semaphore object created by `CreateSemaphore`.
    Semaphore(SemaphoreWaitable),
    /// A heap object created by `HeapCreate`.
    Heap(HeapState),
    /// A registry key opened by `RegOpenKeyEx` / `RegCreateKeyEx`.
    RegistryKey(RegistryKeyState),
    /// A window created by `CreateWindow`.
    Window(HWND),
}

/// State kept for an active `FindFirstFile`/`FindNextFile` session.
#[derive(Debug)]
pub struct FindDataState {
    /// The directory iterator.
    pub entries: Vec<FindEntry>,
    /// Index of the next entry to return.
    pub cursor: usize,
}

/// A single directory entry returned by `FindFirstFile`/`FindNextFile`.
#[derive(Debug, Clone)]
pub struct FindEntry {
    /// File name (just the leaf, not the full path).
    pub file_name: String,
    /// File size in bytes.
    pub file_size: u64,
    /// File attributes (FILE_ATTRIBUTE_* flags).
    pub attributes: u32,
}

/// Global handle table — maps `Handle` → `HandleEntry`.
///
/// Access via [`HANDLE_TABLE`].
pub struct HandleTable {
    inner: Mutex<HandleTableInner>,
}

struct HandleTableInner {
    /// Map from HANDLE raw value → entry.
    map: HashMap<isize, HandleEntry>,
    /// Next HANDLE value to allocate.  Starts above 0x1000 to avoid
    /// collisions with NULL and other sentinels.
    next_id: isize,
}

use std::sync::LazyLock;

/// The process-wide handle table.
static HANDLE_TABLE: LazyLock<HandleTable> = LazyLock::new(|| HandleTable {
    inner: Mutex::new(HandleTableInner {
        map: HashMap::new(),
        next_id: 0x1000,
    }),
});

impl Default for HandleTable {
    fn default() -> Self {
        Self::new()
    }
}

impl HandleTable {
    /// Create a new empty handle table (for tests).
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HandleTableInner {
                map: HashMap::new(),
                next_id: 0x1000,
            }),
        }
    }

    /// Initialise the table with the three standard I/O handles.
    /// Call once during loader startup.
    pub fn init(&self) {
        let mut inner = self.inner.lock().unwrap();
        // Pre-register stdin (fd 0), stdout (fd 1), stderr (fd 2)
        // with well-known handle values.
        for fd in 0..3 {
            let id = inner.next_id;
            inner.next_id += 1;
            inner.map.insert(id, HandleEntry::File(fd));
        }
    }

    /// Insert a new resource and return its HANDLE.
    pub fn insert(&self, entry: HandleEntry) -> HANDLE {
        let mut inner = self.inner.lock().unwrap();
        let id = inner.next_id;
        inner.next_id += 1;
        inner.map.insert(id, entry);
        HANDLE::from_raw(id)
    }

    /// Remove a handle from the table, returning the entry if it existed.
    pub fn remove(&self, h: HANDLE) -> Option<HandleEntry> {
        let mut inner = self.inner.lock().unwrap();
        inner.map.remove(&h.as_raw())
    }

    /// Get the Linux fd for a handle, if it points to a file.
    pub fn get_fd(&self, h: HANDLE) -> Option<i32> {
        let inner = self.inner.lock().unwrap();
        match inner.map.get(&h.as_raw()) {
            Some(HandleEntry::File(fd)) => Some(*fd),
            _ => None,
        }
    }

    /// Run a closure with mutable access to the find-data state behind a handle.
    /// Returns `None` if the handle doesn't exist or isn't a FindData handle.
    pub fn with_find_data<F, R>(&self, h: HANDLE, f: F) -> Option<R>
    where
        F: FnOnce(&mut FindDataState) -> R,
    {
        let mut inner = self.inner.lock().unwrap();
        match inner.map.get_mut(&h.as_raw()) {
            Some(HandleEntry::FindData(state)) => Some(f(state)),
            _ => None,
        }
    }

    /// Run a closure with access to the heap state behind a handle.
    /// Returns `None` if the handle doesn't exist or isn't a Heap handle.
    pub fn with_heap<F, R>(&self, h: HANDLE, f: F) -> Option<R>
    where
        F: FnOnce(&HeapState) -> R,
    {
        let inner = self.inner.lock().unwrap();
        match inner.map.get(&h.as_raw()) {
            Some(HandleEntry::Heap(state)) => Some(f(state)),
            _ => None,
        }
    }

    /// Run a closure with access to the registry key state behind a handle.
    /// Returns `None` if the handle doesn't exist or isn't a RegistryKey handle.
    pub fn with_registry_key<F, R>(&self, h: HANDLE, f: F) -> Option<R>
    where
        F: FnOnce(&RegistryKeyState) -> R,
    {
        let inner = self.inner.lock().unwrap();
        match inner.map.get(&h.as_raw()) {
            Some(HandleEntry::RegistryKey(state)) => Some(f(state)),
            _ => None,
        }
    }

    /// Get the HWND for a window handle.
    /// Returns `None` if the handle doesn't exist or isn't a Window handle.
    pub fn get_hwnd(&self, h: HANDLE) -> Option<HWND> {
        let inner = self.inner.lock().unwrap();
        match inner.map.get(&h.as_raw()) {
            Some(HandleEntry::Window(hwnd)) => Some(*hwnd),
            _ => None,
        }
    }

    /// Get a cloneable waitable object for `WaitForSingleObject` etc.
    /// The returned `Waitable` is Arc-backed so it can be waited on
    /// without holding the table lock.
    pub fn get_waitable(&self, h: HANDLE) -> Option<Waitable> {
        let inner = self.inner.lock().unwrap();
        match inner.map.get(&h.as_raw()) {
            Some(HandleEntry::Thread(t)) => Some(Waitable::Thread(t.clone())),
            Some(HandleEntry::Event(e)) => Some(Waitable::Event(e.clone())),
            Some(HandleEntry::Process(p)) => Some(Waitable::Process(p.clone())),
            Some(HandleEntry::Mutex(m)) => Some(Waitable::Mutex(m.clone())),
            Some(HandleEntry::Semaphore(s)) => Some(Waitable::Semaphore(s.clone())),
            _ => None,
        }
    }

    /// Read a thread's exit code (returns [`STILL_ACTIVE`](crate::threading::STILL_ACTIVE) while running).
    pub fn get_thread_exit_code(&self, h: HANDLE) -> Option<u32> {
        let inner = self.inner.lock().unwrap();
        match inner.map.get(&h.as_raw()) {
            Some(HandleEntry::Thread(t)) => {
                Some(t.exit_code.load(std::sync::atomic::Ordering::Acquire))
            }
            _ => None,
        }
    }
}

/// Access the process-wide handle table.
pub fn handle_table() -> &'static HandleTable {
    &HANDLE_TABLE
}

// ---------------------------------------------------------------------------
// Compatibility shims for (GetStdHandle, WriteFile, etc.)
//
// Uses a simple `fd + HANDLE_FD_BASE` encoding.  The table-based
// approach supersedes it, but we keep the old helpers so existing call
// sites compile without changes.  They now route through the table.
// ---------------------------------------------------------------------------

pub const HANDLE_FD_BASE: isize = 0x1000;

/// Encode a Linux file descriptor as a Windows HANDLE.
pub fn fd_to_handle(fd: i32) -> HANDLE {
    HANDLE::from_raw(fd as isize + HANDLE_FD_BASE)
}

/// Decode a HANDLE back to a Linux fd.
///
/// Tries the handle table first; falls back to the arithmetic
/// encoding for backwards compatibility.
pub fn handle_to_fd(h: HANDLE) -> Option<i32> {
    // Try table first.
    if let Some(fd) = handle_table().get_fd(h) {
        return Some(fd);
    }
    // Fallback: arithmetic encoding.
    let raw = h.as_raw();
    if raw >= HANDLE_FD_BASE {
        Some((raw - HANDLE_FD_BASE) as i32)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Windows file attribute constants
// ---------------------------------------------------------------------------

pub const FILE_ATTRIBUTE_READONLY: u32 = 0x0000_0001;
pub const FILE_ATTRIBUTE_HIDDEN: u32 = 0x0000_0002;
pub const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x0000_0010;
pub const FILE_ATTRIBUTE_ARCHIVE: u32 = 0x0000_0020;
pub const FILE_ATTRIBUTE_NORMAL: u32 = 0x0000_0080;

// ---------------------------------------------------------------------------
// Windows file access / share / creation constants
// ---------------------------------------------------------------------------

pub const GENERIC_READ: u32 = 0x8000_0000;
pub const GENERIC_WRITE: u32 = 0x4000_0000;
pub const GENERIC_EXECUTE: u32 = 0x2000_0000;
pub const GENERIC_ALL: u32 = 0x1000_0000;

pub const FILE_SHARE_READ: u32 = 0x0000_0001;
pub const FILE_SHARE_WRITE: u32 = 0x0000_0002;
pub const FILE_SHARE_DELETE: u32 = 0x0000_0004;

pub const CREATE_NEW: u32 = 1;
pub const CREATE_ALWAYS: u32 = 2;
pub const OPEN_EXISTING: u32 = 3;
pub const OPEN_ALWAYS: u32 = 4;
pub const TRUNCATE_EXISTING: u32 = 5;

/// `SetFilePointer` / `SetFilePointerEx` move method.
pub const FILE_BEGIN: u32 = 0;
pub const FILE_CURRENT: u32 = 1;
pub const FILE_END: u32 = 2;

pub const INVALID_SET_FILE_POINTER: u32 = 0xFFFF_FFFF;
pub const INVALID_FILE_SIZE: u32 = 0xFFFF_FFFF;

// ---------------------------------------------------------------------------
// Win32 find-data structure (ANSI, as returned by FindFirstFileA)
// ---------------------------------------------------------------------------

/// `WIN32_FIND_DATAA` — the struct Windows fills in during FindFirstFileA /
/// FindNextFileA.  We only populate the fields that programs commonly read.
#[repr(C)]
pub struct Win32FindDataA {
    pub file_attributes: u32,
    pub creation_time_lo: u32,
    pub creation_time_hi: u32,
    pub last_access_time_lo: u32,
    pub last_access_time_hi: u32,
    pub last_write_time_lo: u32,
    pub last_write_time_hi: u32,
    pub file_size_high: u32,
    pub file_size_low: u32,
    pub reserved0: u32,
    pub reserved1: u32,
    /// Null-terminated filename, up to MAX_PATH (260) ANSI chars.
    pub file_name: [u8; 260],
    /// 8.3 alternate name.
    pub alternate_file_name: [u8; 14],
}

impl Win32FindDataA {
    /// Fill from a [`FindEntry`].
    pub fn from_entry(entry: &FindEntry) -> Self {
        let mut data = Self {
            file_attributes: entry.attributes,
            creation_time_lo: 0,
            creation_time_hi: 0,
            last_access_time_lo: 0,
            last_access_time_hi: 0,
            last_write_time_lo: 0,
            last_write_time_hi: 0,
            file_size_high: (entry.file_size >> 32) as u32,
            file_size_low: entry.file_size as u32,
            reserved0: 0,
            reserved1: 0,
            file_name: [0u8; 260],
            alternate_file_name: [0u8; 14],
        };
        let name_bytes = entry.file_name.as_bytes();
        let copy_len = name_bytes.len().min(259); // leave room for null
        data.file_name[..copy_len].copy_from_slice(&name_bytes[..copy_len]);
        data
    }
}

/// Wide variant — `WIN32_FIND_DATAW`.
#[repr(C)]
pub struct Win32FindDataW {
    pub file_attributes: u32,
    pub creation_time_lo: u32,
    pub creation_time_hi: u32,
    pub last_access_time_lo: u32,
    pub last_access_time_hi: u32,
    pub last_write_time_lo: u32,
    pub last_write_time_hi: u32,
    pub file_size_high: u32,
    pub file_size_low: u32,
    pub reserved0: u32,
    pub reserved1: u32,
    /// Null-terminated filename, up to MAX_PATH (260) wide chars.
    pub file_name: [u16; 260],
    /// 8.3 alternate name.
    pub alternate_file_name: [u16; 14],
}

impl Win32FindDataW {
    /// Fill from a [`FindEntry`].
    pub fn from_entry(entry: &FindEntry) -> Self {
        let mut data = Self {
            file_attributes: entry.attributes,
            creation_time_lo: 0,
            creation_time_hi: 0,
            last_access_time_lo: 0,
            last_access_time_hi: 0,
            last_write_time_lo: 0,
            last_write_time_hi: 0,
            file_size_high: (entry.file_size >> 32) as u32,
            file_size_low: entry.file_size as u32,
            reserved0: 0,
            reserved1: 0,
            file_name: [0u16; 260],
            alternate_file_name: [0u16; 14],
        };
        let wide: Vec<u16> = entry.file_name.encode_utf16().collect();
        let copy_len = wide.len().min(259);
        data.file_name[..copy_len].copy_from_slice(&wide[..copy_len]);
        data
    }
}

// ---------------------------------------------------------------------------
// Glob matching for FindFirstFile patterns
// ---------------------------------------------------------------------------

/// Simple glob match supporting `*` and `?` (Windows FindFirstFile semantics).
/// Case-insensitive.
pub fn glob_matches(pattern: &str, name: &str) -> bool {
    glob_match_bytes(
        pattern.to_ascii_lowercase().as_bytes(),
        name.to_ascii_lowercase().as_bytes(),
    )
}

fn glob_match_bytes(pattern: &[u8], name: &[u8]) -> bool {
    let (mut pi, mut ni) = (0, 0);
    let (mut star_p, mut star_n) = (usize::MAX, 0);

    while ni < name.len() {
        if pi < pattern.len() && (pattern[pi] == b'?' || pattern[pi] == name[ni]) {
            pi += 1;
            ni += 1;
        } else if pi < pattern.len() && pattern[pi] == b'*' {
            star_p = pi;
            star_n = ni;
            pi += 1;
        } else if star_p != usize::MAX {
            pi = star_p + 1;
            star_n += 1;
            ni = star_n;
        } else {
            return false;
        }
    }

    while pi < pattern.len() && pattern[pi] == b'*' {
        pi += 1;
    }
    pi == pattern.len()
}

// ---------------------------------------------------------------------------
// Helper: collect directory entries matching a glob
// ---------------------------------------------------------------------------

/// Read a directory and collect entries matching `pattern` (the filename
/// component of a Windows path like `C:\Dir\*.txt`).
///
/// `dir` is the already-translated Linux directory path.
pub fn collect_find_entries(dir: &std::path::Path, pattern: &str) -> Vec<FindEntry> {
    let mut entries = Vec::new();
    let Ok(read_dir) = std::fs::read_dir(dir) else {
        return entries;
    };

    for result in read_dir {
        let Ok(de) = result else { continue };
        let fname = de.file_name().to_string_lossy().into_owned();
        if !glob_matches(pattern, &fname) {
            continue;
        }
        let meta = de.metadata().ok();
        let file_size = meta.as_ref().map_or(0, |m| m.len());
        let is_dir = meta.as_ref().is_some_and(|m| m.is_dir());
        let attributes = if is_dir {
            FILE_ATTRIBUTE_DIRECTORY
        } else {
            FILE_ATTRIBUTE_ARCHIVE
        };
        entries.push(FindEntry {
            file_name: fname,
            file_size,
            attributes,
        });
    }
    entries
}

// ---------------------------------------------------------------------------
// Path utilities for splitting "dir\pattern"
// ---------------------------------------------------------------------------

/// Split a Windows find-file path (e.g. `C:\Dir\*.txt`) into the directory
/// portion and the glob pattern.  Returns `(dir_part, pattern)`.
///
/// If there's no backslash or forward slash, the entire string is the pattern
/// and directory is empty.
pub fn split_find_path(win_path: &str) -> (&str, &str) {
    // Find last separator.
    let sep = win_path.rfind(['\\', '/']).map(|i| i + 1);
    match sep {
        Some(pos) => (&win_path[..pos], &win_path[pos..]),
        None => ("", win_path),
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
        assert!(HANDLE::INVALID.is_invalid());
        assert!(!HANDLE::NULL.is_invalid());
    }

    #[test]
    fn handle_table_insert_and_get_fd() {
        let table = HandleTable::new();
        let h = table.insert(HandleEntry::File(42));
        assert_eq!(table.get_fd(h), Some(42));
    }

    #[test]
    fn handle_table_remove() {
        let table = HandleTable::new();
        let h = table.insert(HandleEntry::File(7));
        assert!(table.get_fd(h).is_some());
        let removed = table.remove(h);
        assert!(matches!(removed, Some(HandleEntry::File(7))));
        assert!(table.get_fd(h).is_none());
    }

    #[test]
    fn handle_table_find_data() {
        let table = HandleTable::new();
        let h = table.insert(HandleEntry::FindData(FindDataState {
            entries: vec![FindEntry {
                file_name: "test.txt".into(),
                file_size: 100,
                attributes: FILE_ATTRIBUTE_ARCHIVE,
            }],
            cursor: 0,
        }));
        // get_fd should return None for find-data handles
        assert!(table.get_fd(h).is_none());
        let name = table.with_find_data(h, |state| state.entries[state.cursor].file_name.clone());
        assert_eq!(name, Some("test.txt".into()));
    }

    #[test]
    fn glob_basic() {
        assert!(glob_matches("*", "anything"));
        assert!(glob_matches("*.txt", "readme.txt"));
        assert!(!glob_matches("*.txt", "readme.md"));
        assert!(glob_matches("test?", "test1"));
        assert!(!glob_matches("test?", "test12"));
        assert!(glob_matches("*.*", "file.txt"));
        assert!(glob_matches("FILE.*", "file.TXT")); // case insensitive
    }

    #[test]
    fn split_find_path_cases() {
        assert_eq!(split_find_path(r"C:\Dir\*.txt"), (r"C:\Dir\", "*.txt"));
        assert_eq!(split_find_path("*.exe"), ("", "*.exe"));
        assert_eq!(split_find_path(r"foo\bar"), (r"foo\", "bar"));
    }

    #[test]
    fn handle_table_thread_entry() {
        use crate::threading::{STILL_ACTIVE, ThreadWaitable};
        use std::sync::atomic::AtomicU32;
        use std::sync::{Arc, Condvar, Mutex};

        let table = HandleTable::new();
        let tw = ThreadWaitable {
            exit_code: Arc::new(AtomicU32::new(STILL_ACTIVE)),
            completed: Arc::new((Mutex::new(false), Condvar::new())),
        };
        let h = table.insert(HandleEntry::Thread(tw));

        // get_fd returns None for thread handles.
        assert!(table.get_fd(h).is_none());

        // get_thread_exit_code returns STILL_ACTIVE.
        assert_eq!(table.get_thread_exit_code(h), Some(STILL_ACTIVE));

        // get_waitable returns a Thread variant.
        assert!(matches!(
            table.get_waitable(h),
            Some(crate::threading::Waitable::Thread(_))
        ));
    }

    #[test]
    fn handle_table_event_entry() {
        use crate::threading::{EventInner, EventWaitable};
        use std::sync::{Arc, Condvar, Mutex};

        let table = HandleTable::new();
        let e = EventWaitable {
            inner: Arc::new(EventInner {
                signaled: Mutex::new(false),
                condvar: Condvar::new(),
                manual_reset: true,
            }),
        };
        let h = table.insert(HandleEntry::Event(e));

        assert!(table.get_fd(h).is_none());
        assert!(table.get_thread_exit_code(h).is_none());
        assert!(matches!(
            table.get_waitable(h),
            Some(crate::threading::Waitable::Event(_))
        ));
    }

    #[test]
    fn handle_table_get_waitable_returns_none_for_file() {
        let table = HandleTable::new();
        let h = table.insert(HandleEntry::File(99));
        assert!(table.get_waitable(h).is_none());
    }

    #[test]
    fn handle_table_remove_thread() {
        use crate::threading::{STILL_ACTIVE, ThreadWaitable};
        use std::sync::atomic::AtomicU32;
        use std::sync::{Arc, Condvar, Mutex};

        let table = HandleTable::new();
        let tw = ThreadWaitable {
            exit_code: Arc::new(AtomicU32::new(STILL_ACTIVE)),
            completed: Arc::new((Mutex::new(false), Condvar::new())),
        };
        let h = table.insert(HandleEntry::Thread(tw));
        assert!(matches!(table.remove(h), Some(HandleEntry::Thread(_))));
        assert!(table.get_waitable(h).is_none());
    }
}
