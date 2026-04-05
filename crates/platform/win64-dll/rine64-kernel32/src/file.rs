//! kernel32 file I/O: CreateFileA/W, ReadFile, WriteFile, CloseHandle,
//! GetFileSize, SetFilePointer, FindFirstFileA/W, FindNextFileA/W, FindClose.

use rine_common_kernel32 as common;
use rine_types::errors::WinBool;
use rine_types::handles::{
    self, FILE_BEGIN, FILE_CURRENT, FILE_END, FindDataState, Handle, HandleEntry,
    INVALID_FILE_SIZE, INVALID_HANDLE_VALUE, INVALID_SET_FILE_POINTER, Win32FindDataA,
    Win32FindDataW, handle_table, handle_to_fd,
};
use rine_types::strings::{read_cstr, read_wstr};

// ---------------------------------------------------------------------------
// CreateFileA / CreateFileW
// ---------------------------------------------------------------------------

/// CreateFileA — open or create a file (ANSI path).
///
/// # Arguments
/// * `file_name`: pointer to a null-terminated ANSI string with the file path.
/// * `desired_access`: bitmask of GENERIC_READ, GENERIC_WRITE, etc.
/// * `creation_disposition`: action to take on files that exist or do not exist.
/// * _share_mode - ignored
/// * _security_attributes - ignored
/// * _flags_and_attributes - ignored
/// * _template_file - ignored
///
/// # Safety
/// `file_name` must be a valid null-terminated ANSI string.
///
/// The caller must ensure that the file path is valid and that the desired
/// access and creation disposition are appropriate.
///
/// # Note
/// This implementation does not support all features of the Windows API, such as
/// sharing modes, security attributes, or file attributes. It focuses on basic
/// file creation and opening functionality.
///
/// dev_emit! are handled in the common::create_file implementation, so that they
/// are emitted for both CreateFileA and CreateFileW in rine & rine32.
#[allow(non_snake_case)]
pub unsafe extern "win64" fn CreateFileA(
    file_name: *const u8,
    desired_access: u32,
    _share_mode: u32,
    _security_attributes: usize, // LPSECURITY_ATTRIBUTES (ignored)
    creation_disposition: u32,
    _flags_and_attributes: u32,
    _template_file: isize, // HANDLE (ignored)
) -> isize {
    if file_name.is_null() {
        return INVALID_HANDLE_VALUE.as_raw();
    }

    let c_str = unsafe { read_cstr(file_name).unwrap_or_default() };
    let path_str = c_str.to_string();

    common::file::create_file(&path_str, desired_access, creation_disposition)
}

/// CreateFileW — open or create a file (wide/UTF-16 path).
///
/// # Safety
/// `file_name` must be a valid null-terminated UTF-16LE string.
#[allow(non_snake_case)]
pub unsafe extern "win64" fn CreateFileW(
    file_name: *const u16,
    desired_access: u32,
    _share_mode: u32,
    _security_attributes: usize,
    creation_disposition: u32,
    _flags_and_attributes: u32,
    _template_file: isize,
) -> isize {
    if file_name.is_null() {
        return INVALID_HANDLE_VALUE.as_raw();
    }

    let wide_file_name = unsafe { read_wstr(file_name).unwrap_or_default() };
    let path_str = wide_file_name.to_string();

    common::file::create_file(&path_str, desired_access, creation_disposition)
}

// ---------------------------------------------------------------------------
// ReadFile
// ---------------------------------------------------------------------------

/// ReadFile — read data from a file.
///
/// # Safety
/// `buffer` must be writable for at least `bytes_to_read` bytes.
#[allow(non_snake_case)]
pub unsafe extern "win64" fn ReadFile(
    file: isize,
    buffer: *mut u8,
    bytes_to_read: u32,
    bytes_read: *mut u32,
    _overlapped: *mut core::ffi::c_void,
) -> WinBool {
    let handle = Handle::from_raw(file);
    let Some(fd) = handle_to_fd(handle) else {
        return WinBool::FALSE;
    };

    let n = unsafe { libc::read(fd, buffer.cast(), bytes_to_read as usize) };
    if n < 0 {
        return WinBool::FALSE;
    }

    if !bytes_read.is_null() {
        unsafe { *bytes_read = n as u32 };
    }
    WinBool::TRUE
}

// ---------------------------------------------------------------------------
// WriteFile (existing, updated)
// ---------------------------------------------------------------------------

/// WriteFile — write data to a file or I/O device.
///
/// # Safety
/// `buffer` must point to at least `bytes_to_write` readable bytes.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn WriteFile(
    file: isize,
    buffer: *const u8,
    bytes_to_write: u32,
    bytes_written: *mut u32,
    _overlapped: *mut core::ffi::c_void,
) -> WinBool {
    let handle = Handle::from_raw(file);
    let Some(fd) = handle_to_fd(handle) else {
        return WinBool::FALSE;
    };

    let written = unsafe { libc::write(fd, buffer.cast(), bytes_to_write as usize) };
    if written < 0 {
        return WinBool::FALSE;
    }

    if !bytes_written.is_null() {
        unsafe { *bytes_written = written as u32 };
    }
    WinBool::TRUE
}

// ---------------------------------------------------------------------------
// CloseHandle
// ---------------------------------------------------------------------------

/// CloseHandle — close an open handle (file, find-data, etc.).
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn CloseHandle(object: isize) -> WinBool {
    let handle = Handle::from_raw(object);
    rine_types::dev_notify!(on_handle_closed(object as i64));

    match handle_table().remove(handle) {
        Some(HandleEntry::File(fd)) => {
            unsafe { libc::close(fd) };
            WinBool::TRUE
        }
        Some(HandleEntry::FindData(_)) => {
            // FindData has no OS resource to free.
            WinBool::TRUE
        }
        Some(HandleEntry::Thread(_)) => {
            // Thread keeps running; we just release our handle.
            WinBool::TRUE
        }
        Some(HandleEntry::Event(_)) => WinBool::TRUE,
        Some(HandleEntry::Process(_)) => WinBool::TRUE,
        Some(HandleEntry::Mutex(_)) => WinBool::TRUE,
        Some(HandleEntry::Semaphore(_)) => WinBool::TRUE,
        Some(HandleEntry::Heap(_)) => WinBool::TRUE,
        Some(HandleEntry::RegistryKey(_)) => WinBool::TRUE,
        Some(HandleEntry::Window(_)) => {
            // Window handles are managed by user32, not kernel32.
            // They should not be closed via CloseHandle.
            WinBool::FALSE
        }
        None => {
            tracing::warn!(?handle, "CloseHandle: unknown handle");
            WinBool::FALSE
        }
    }
}

// ---------------------------------------------------------------------------
// GetFileSize
// ---------------------------------------------------------------------------

/// GetFileSize — return the size of a file in bytes.
///
/// Returns the low 32 bits. If `file_size_high` is non-null, the high
/// 32 bits are written there.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn GetFileSize(file: isize, file_size_high: *mut u32) -> u32 {
    let handle = Handle::from_raw(file);
    let Some(fd) = handle_to_fd(handle) else {
        return INVALID_FILE_SIZE;
    };

    let mut stat: libc::stat = unsafe { core::mem::zeroed() };
    if unsafe { libc::fstat(fd, &mut stat) } != 0 {
        return INVALID_FILE_SIZE;
    }

    let size = stat.st_size as u64;
    if !file_size_high.is_null() {
        unsafe { *file_size_high = (size >> 32) as u32 };
    }
    size as u32
}

// ---------------------------------------------------------------------------
// SetFilePointer
// ---------------------------------------------------------------------------

/// SetFilePointer — move the file pointer.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn SetFilePointer(
    file: isize,
    distance_to_move: i32,           // low 32 bits
    distance_to_move_high: *mut i32, // high 32 bits (in/out, optional)
    move_method: u32,
) -> u32 {
    let handle = Handle::from_raw(file);
    let Some(fd) = handle_to_fd(handle) else {
        return INVALID_SET_FILE_POINTER;
    };

    let offset: i64 = if !distance_to_move_high.is_null() {
        let high = unsafe { *distance_to_move_high } as i64;
        (high << 32) | (distance_to_move as u32 as i64)
    } else {
        distance_to_move as i64
    };

    let whence = match move_method {
        FILE_BEGIN => libc::SEEK_SET,
        FILE_CURRENT => libc::SEEK_CUR,
        FILE_END => libc::SEEK_END,
        _ => return INVALID_SET_FILE_POINTER,
    };

    let result = unsafe { libc::lseek(fd, offset, whence) };
    if result == -1 {
        return INVALID_SET_FILE_POINTER;
    }

    if !distance_to_move_high.is_null() {
        unsafe { *distance_to_move_high = (result >> 32) as i32 };
    }
    result as u32
}

// ---------------------------------------------------------------------------
// FindFirstFileA / FindFirstFileW
// ---------------------------------------------------------------------------

/// FindFirstFileA — begin searching for files matching a pattern (ANSI).
///
/// # Safety
/// `find_data` must point to a writable `WIN32_FIND_DATAA`.
#[allow(non_snake_case)]
pub unsafe extern "win64" fn FindFirstFileA(
    file_name: *const u8,
    find_data: *mut Win32FindDataA,
) -> isize {
    if file_name.is_null() || find_data.is_null() {
        return INVALID_HANDLE_VALUE.as_raw();
    }

    let c_str = unsafe { std::ffi::CStr::from_ptr(file_name.cast()) };
    let path_str = c_str.to_string_lossy();

    let (dir_part, pattern) = handles::split_find_path(&path_str);

    let linux_dir = common::file::translate_find_dir(dir_part);
    let entries = handles::collect_find_entries(&linux_dir, pattern);
    if entries.is_empty() {
        return INVALID_HANDLE_VALUE.as_raw();
    }

    // Write the first entry.
    unsafe { core::ptr::write(find_data, Win32FindDataA::from_entry(&entries[0])) };

    let h = handle_table().insert(HandleEntry::FindData(FindDataState { entries, cursor: 1 }));
    rine_types::dev_notify!(on_handle_created(h.as_raw() as i64, "FindData", &path_str));
    h.as_raw()
}

/// FindFirstFileW — begin searching for files matching a pattern (wide).
///
/// # Safety
/// `find_data` must point to a writable `WIN32_FIND_DATAW`.
#[allow(non_snake_case)]
pub unsafe extern "win64" fn FindFirstFileW(
    file_name: *const u16,
    find_data: *mut Win32FindDataW,
) -> isize {
    if file_name.is_null() || find_data.is_null() {
        return INVALID_HANDLE_VALUE.as_raw();
    }

    let mut len = 0;
    unsafe {
        while *file_name.add(len) != 0 {
            len += 1;
        }
    }
    let wide = unsafe { core::slice::from_raw_parts(file_name, len) };
    let path_str = String::from_utf16_lossy(wide);

    let (dir_part, pattern) = handles::split_find_path(&path_str);

    let linux_dir = common::file::translate_find_dir(dir_part);
    let entries = handles::collect_find_entries(&linux_dir, pattern);
    if entries.is_empty() {
        return INVALID_HANDLE_VALUE.as_raw();
    }

    unsafe { core::ptr::write(find_data, Win32FindDataW::from_entry(&entries[0])) };

    let h = handle_table().insert(HandleEntry::FindData(FindDataState { entries, cursor: 1 }));
    rine_types::dev_notify!(on_handle_created(h.as_raw() as i64, "FindData", &path_str));
    h.as_raw()
}

// ---------------------------------------------------------------------------
// FindNextFileA / FindNextFileW
// ---------------------------------------------------------------------------

/// FindNextFileA — continue a directory search (ANSI).
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn FindNextFileA(
    find_file: isize,
    find_data: *mut Win32FindDataA,
) -> WinBool {
    if find_data.is_null() {
        return WinBool::FALSE;
    }
    let handle = Handle::from_raw(find_file);

    let result = handle_table().with_find_data(handle, |state| {
        if state.cursor >= state.entries.len() {
            return false;
        }
        let entry = &state.entries[state.cursor];
        unsafe { core::ptr::write(find_data, Win32FindDataA::from_entry(entry)) };
        state.cursor += 1;
        true
    });

    match result {
        Some(true) => WinBool::TRUE,
        _ => WinBool::FALSE,
    }
}

/// FindNextFileW — continue a directory search (wide).
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn FindNextFileW(
    find_file: isize,
    find_data: *mut Win32FindDataW,
) -> WinBool {
    if find_data.is_null() {
        return WinBool::FALSE;
    }
    let handle = Handle::from_raw(find_file);

    let result = handle_table().with_find_data(handle, |state| {
        if state.cursor >= state.entries.len() {
            return false;
        }
        let entry = &state.entries[state.cursor];
        unsafe { core::ptr::write(find_data, Win32FindDataW::from_entry(entry)) };
        state.cursor += 1;
        true
    });

    match result {
        Some(true) => WinBool::TRUE,
        _ => WinBool::FALSE,
    }
}

// ---------------------------------------------------------------------------
// FindClose
// ---------------------------------------------------------------------------

/// FindClose — close a search handle opened by FindFirstFile.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn FindClose(find_file: isize) -> WinBool {
    unsafe { CloseHandle(find_file) }
}
