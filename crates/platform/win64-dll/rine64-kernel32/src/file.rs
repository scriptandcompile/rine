//! kernel32 file I/O: CreateFileA/W, ReadFile, WriteFile, CloseHandle,
//! GetFileSize, SetFilePointer, FindFirstFileA/W, FindNextFileA/W, FindClose.

use rine_common_kernel32 as common;
use rine_types::errors::WinBool;
use rine_types::handles::{
    self, FindDataState, Handle, HandleEntry, INVALID_FILE_SIZE, INVALID_HANDLE_VALUE,
    Win32FindDataA, Win32FindDataW, handle_table,
};
use rine_types::strings::{read_cstr, read_wstr};

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
/// The caller must ensure that the file path is valid and that the desired
/// access and creation disposition are appropriate.
///
/// # Returns
/// A file handle on success, or INVALID_HANDLE_VALUE on failure.
///
/// # Note
/// This implementation does not support all features of the Windows API, such as
/// sharing modes, security attributes, or file attributes. It focuses on basic
/// file creation and opening functionality.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
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
/// # Arguments
/// * `file_name`: pointer to a null-terminated UTF-16LE string with the file path.
/// * `desired_access`: bitmask of GENERIC_READ, GENERIC_WRITE, etc.
/// * `creation_disposition`: action to take on files that exist or do not exist.
/// * _share_mode - ignored
/// * _security_attributes - ignored
/// * _flags_and_attributes - ignored
/// * _template_file - ignored
///
/// # Safety
/// `file_name` must be a valid null-terminated UTF-16LE string.
/// The caller must ensure that the file path is valid and that the desired
/// access and creation disposition are appropriate.
///
/// # Returns
/// A file handle on success, or INVALID_HANDLE_VALUE on failure.
///
/// # Note
/// This implementation does not support all features of the Windows API, such as
/// sharing modes, security attributes, or file attributes. It focuses on basic
/// file creation and opening functionality.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn CreateFileW(
    file_name: *const u16,
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

    let wide_file_name = unsafe { read_wstr(file_name).unwrap_or_default() };
    let path_str = wide_file_name.to_string();

    common::file::create_file(&path_str, desired_access, creation_disposition)
}

/// DeleteFileW — delete a file (wide/UTF-16 path).
///
/// # Arguments
/// * `file_name`: pointer to a null-terminated UTF-16LE string with the file path.
///
/// # Safety
/// `file_name` must be a valid null-terminated UTF-16LE string.
/// The caller must ensure that the file path is valid and that the file can be deleted.
///
/// # Returns
/// `WinBool::TRUE` on success, `WinBool::FALSE` on failure.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn DeleteFileW(file_name: *const u16) -> WinBool {
    if file_name.is_null() {
        return WinBool::FALSE;
    }

    let wide_file_name = unsafe { read_wstr(file_name).unwrap_or_default() };
    let path_str = wide_file_name.to_string();
    common::file::delete_file(&path_str)
}

/// DeleteFileA — delete a file (ANSI path).
///
/// # Arguments
/// * `file_name`: pointer to a null-terminated ANSI string with the file path.
///
/// # Safety
/// `file_name` must be a valid null-terminated ANSI string.
/// The caller must ensure that the file path is valid and that the file can be deleted.
///
/// # Returns
/// `WinBool::TRUE` on success, `WinBool::FALSE` on failure.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn DeleteFileA(file_name: *const u8) -> WinBool {
    if file_name.is_null() {
        return WinBool::FALSE;
    }

    let c_str = unsafe { read_cstr(file_name).unwrap_or_default() };
    let path_str = c_str.to_string();
    common::file::delete_file(&path_str)
}

/// ReadFile — read data from a file.
///
/// # Arguments
/// * `file` - The file handle to read from. Must be a valid file handle returned by `CreateFile`.
/// * `buffer` - Pointer to a buffer that receives the data read from the file.
/// * `bytes_to_read` - The number of bytes to read.
/// * `bytes_read` - Optional pointer to a variable that receives the number of bytes read.
/// * `_overlapped` - ignored (asynchronous I/O is not supported).
///
/// # Safety
/// `file` must be a valid file handle returned by `CreateFile`.
/// `buffer` must be writable for at least `bytes_to_read` bytes.
/// `bytes_read` must be null or point to a valid u32 variable.
/// `_overlapped` must be null or point to a valid OVERLAPPED structure,
/// but asynchronous I/O is not supported so it will be ignored.
///
/// # Returns
/// `WinBool::TRUE` on success, `WinBool::FALSE` on failure.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn ReadFile(
    file: isize,
    buffer: *mut u8,
    bytes_to_read: u32,
    bytes_read: *mut u32,
    _overlapped: *mut core::ffi::c_void,
) -> WinBool {
    let handle = Handle::from_raw(file);

    unsafe { common::file::read_file(handle, buffer, bytes_to_read, bytes_read, _overlapped) }
}

/// WriteFile — write data to a file or I/O device.
///
/// # Arguments
/// * `file` - The file handle to write to. Must be a valid file handle returned by `CreateFile`.
/// * `buffer` - Pointer to the data to be written to the file.
/// * `bytes_to_write` - The number of bytes to write.
/// * `bytes_written` - Optional pointer to a variable that receives the number of bytes written.
/// * `_overlapped` - ignored (asynchronous I/O is not supported).
///
/// # Safety
/// `file` must be a valid file handle returned by `CreateFile`.
/// `buffer` must point to at least `bytes_to_write` readable bytes.
/// `bytes_written` must be null or point to a valid u32 variable.
/// `_overlapped` must be null or point to a valid OVERLAPPED structure,
/// but asynchronous I/O is not supported so it will be ignored.
///
/// # Returns
/// `WinBool::TRUE` on success, `WinBool::FALSE` on failure.
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
    unsafe { common::file::write_file(handle, buffer, bytes_to_write, bytes_written, _overlapped) }
}

/// FlushFileBuffers — flush file buffers to disk.
///
/// # Arguments
/// * `file` - The file handle to flush. Must be a valid file handle returned by `CreateFile`.
///
/// # Safety
/// `file` must be a valid file handle returned by `CreateFile`.
///
/// # Returns
/// `WinBool::TRUE` on success, `WinBool::FALSE` on failure.
///
/// # Note
/// This implementation does not support flushing of non-file handles (e.g. pipes, consoles).
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn FlushFileBuffers(file: isize) -> WinBool {
    let handle = Handle::from_raw(file);
    common::file::flush_file_buffers(handle)
}

/// CloseHandle — close an open object handle (e.g. file handle).
///
/// # Arguments
/// * `object` - The handle to close. Must be a valid handle returned by `CreateFile` or other handle-returning function.
///
/// # Safety
/// `object` must be a valid handle returned by `CreateFile` or other handle-returning function.
/// After this call, `object` must not be used again.
///
/// # Returns
/// `WinBool::TRUE` on success, `WinBool::FALSE` on failure.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn CloseHandle(object: isize) -> WinBool {
    let handle = Handle::from_raw(object);

    rine_types::dev_notify!(on_handle_closed(object as i64));

    common::file::close_handle(handle)
}

/// GetFileSize — return the size of a file in bytes.
///
/// Returns the low 32 bits. If `file_size_high` is non-null, the high
/// 32 bits are written there.
///
/// # Arguments
/// * `file` - The file handle to query. Must be a valid file handle returned by `CreateFile`.
/// * `file_size_high` - Optional pointer to receive the high 32 bits of the file size.
///   If the file size exceeds 4GB, this must be non-null and will be set to the high bits of the file size.
///   If the file size is 4GB or less, this can be null or will be set to zero.
///
/// # Safety
/// * `file` must be a valid file handle returned by `CreateFile`.
/// * `file_size_high` must be null or point to a valid u32 variable.
///
/// # Returns
/// The low 32 bits of the file size on success, or INVALID_FILE_SIZE (0xFFFFFFFF) on failure.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetFileSize(file: isize, file_size_high: *mut u32) -> u32 {
    let handle = Handle::from_raw(file);

    let Some(size) = common::file::get_file_size(handle) else {
        return INVALID_FILE_SIZE;
    };

    if !file_size_high.is_null() {
        unsafe { *file_size_high = (size >> 32) as u32 };
    }
    size as u32
}

/// SetFilePointer — move the file pointer for a file handle.
///
/// # Arguments
/// * `file` - The file handle whose pointer to move.
/// * `distance_to_move` - The low 32 bits of the distance to move, in bytes. Can be negative to move backwards.
/// * `distance_to_move_high` - Optional pointer to the high 32 bits of the distance to move.
///   If non-null, this is an input/output parameter that should be initialized to the high bits of the distance
///   before the call, and will be updated to the high bits of the new file pointer after the call.
/// * `move_method` - The starting point for the move. Must be one of `FILE_BEGIN`, `FILE_CURRENT`, or `FILE_END`.
///
/// # Safety
/// * `file` must be a valid file handle returned by `CreateFile`.
/// * `distance_to_move_high` must be null or point to a valid i32 variable if `distance_to_move` is negative
///   or the distance exceeds 2GB.
#[allow(non_snake_case, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn SetFilePointer(
    file: isize,
    distance_to_move: i32,           // low 32 bits
    distance_to_move_high: *mut i32, // high 32 bits (in/out, optional)
    move_method: u32,
) -> u32 {
    let handle = Handle::from_raw(file);

    unsafe {
        common::file::set_file_pointer(handle, distance_to_move, distance_to_move_high, move_method)
    }
}

// ---------------------------------------------------------------------------
// FindFirstFileA / FindFirstFileW
// ---------------------------------------------------------------------------

/// FindFirstFileA — begin searching for files matching a pattern (ANSI).
///
/// # Safety
/// `find_data` must point to a writable `WIN32_FIND_DATAA`.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
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
#[unsafe(no_mangle)]
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
#[unsafe(no_mangle)]
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
#[unsafe(no_mangle)]
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

/// FindClose — close a search handle opened by FindFirstFile.
///
/// # Arguments
/// * `find_file` - The search handle returned by `FindFirstFile`.
///
/// # Safety
/// * `find_file` must be a valid search handle returned by `FindFirstFile`.
/// * After this call, `find_file` must not be used again.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn FindClose(find_file: isize) -> WinBool {
    unsafe { CloseHandle(find_file) }
}
