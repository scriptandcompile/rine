use rine_types::errors::WinBool;
use rine_types::strings::{LPSTR, LPWSTR};
use rine_types::windows::{DropFiles, HDROP, Hwnd, WINDOW_MANAGER, window_style_ex};

const DRAGQUERYFILE_COUNT: u32 = 0xFFFF_FFFF;

/// Enables or disables file-drop acceptance for a window.
///
/// # Arguments
/// * `hwnd` - Target window handle.
/// * `accept` - Nonzero enables file drops; zero disables them.
///
/// # Return
/// This function returns no value.
///
/// # Notes
/// This updates the tracked extended style bit (`WS_EX_ACCEPTFILES`) for
/// windows known to the current runtime.
pub fn drag_accept_files(hwnd: Hwnd, accept: WinBool) {
    let _ = WINDOW_MANAGER.update_window(hwnd, |state| {
        if accept.is_true() {
            state.ex_style |= window_style_ex::WS_EX_ACCEPTFILES;
        } else {
            state.ex_style &= !window_style_ex::WS_EX_ACCEPTFILES;
        }
    });
}

unsafe fn count_ansi_entries(mut cursor: *const u8) -> u32 {
    let mut count = 0u32;
    loop {
        if unsafe { *cursor } == 0 {
            break;
        }

        while unsafe { *cursor } != 0 {
            cursor = unsafe { cursor.add(1) };
        }
        count += 1;
        cursor = unsafe { cursor.add(1) };
    }

    count
}

unsafe fn get_ansi_entry(mut cursor: *const u8, index: u32) -> Option<Vec<u8>> {
    let mut current = 0u32;
    loop {
        if unsafe { *cursor } == 0 {
            return None;
        }

        let start = cursor;
        let mut len = 0usize;
        while unsafe { *cursor } != 0 {
            cursor = unsafe { cursor.add(1) };
            len += 1;
        }

        if current == index {
            return Some(unsafe { std::slice::from_raw_parts(start, len) }.to_vec());
        }

        current += 1;
        cursor = unsafe { cursor.add(1) };
    }
}

unsafe fn count_wide_entries(mut cursor: *const u16) -> u32 {
    let mut count = 0u32;
    loop {
        if unsafe { *cursor } == 0 {
            break;
        }

        while unsafe { *cursor } != 0 {
            cursor = unsafe { cursor.add(1) };
        }
        count += 1;
        cursor = unsafe { cursor.add(1) };
    }

    count
}

unsafe fn get_wide_entry(mut cursor: *const u16, index: u32) -> Option<Vec<u16>> {
    let mut current = 0u32;
    loop {
        if unsafe { *cursor } == 0 {
            return None;
        }

        let start = cursor;
        let mut len = 0usize;
        while unsafe { *cursor } != 0 {
            cursor = unsafe { cursor.add(1) };
            len += 1;
        }

        if current == index {
            let units = unsafe { std::slice::from_raw_parts(start, len) };
            return Some(units.to_vec());
        }

        current += 1;
        cursor = unsafe { cursor.add(1) };
    }
}

enum DragQueryFileEntry {
    Ansi(Vec<u8>),
    Wide(Vec<u16>),
}

pub enum DragQueryFileBuffer {
    Ansi { lpsz_file: LPSTR, cch: u32 },
    Wide { lpsz_file: LPWSTR, cch: u32 },
}

/// Queries file paths from a shell drag-and-drop handle.
///
/// # Arguments
/// * `hdrop` - Handle identifying the dropped-file list.
/// * `i_file` - Zero-based file index to query, or `0xFFFFFFFF` to request file count.
/// * `buffer` - Target output mode and buffer.
///
/// # Safety
/// `hdrop` must reference a valid DROPFILES memory block. When an output buffer
/// is provided, it must point to writable memory of at least `cch` elements.
///
/// # Return
/// Returns file count when `i_file == 0xFFFFFFFF`; otherwise returns the selected
/// file path length in characters excluding the null terminator. Returns `0` on
/// failure or when `i_file` is out of range.
pub unsafe fn drag_query_file(hdrop: HDROP, i_file: u32, buffer: DragQueryFileBuffer) -> u32 {
    if hdrop.is_null() {
        return 0;
    }

    let base = hdrop.as_raw() as *const u8;
    let drop_files = unsafe { &*(base as *const DropFiles) };
    let list_base = unsafe { base.add(drop_files.p_files as usize) };

    let count = if drop_files.f_wide.is_true() {
        unsafe { count_wide_entries(list_base as *const u16) }
    } else {
        unsafe { count_ansi_entries(list_base) }
    };

    if i_file == DRAGQUERYFILE_COUNT {
        return count;
    }

    if i_file >= count {
        return 0;
    }

    let entry = if drop_files.f_wide.is_true() {
        match unsafe { get_wide_entry(list_base as *const u16, i_file) } {
            Some(value) => DragQueryFileEntry::Wide(value),
            None => return 0,
        }
    } else {
        match unsafe { get_ansi_entry(list_base, i_file) } {
            Some(value) => DragQueryFileEntry::Ansi(value),
            None => return 0,
        }
    };

    match buffer {
        DragQueryFileBuffer::Ansi { lpsz_file, cch } => {
            let ansi = match entry {
                DragQueryFileEntry::Ansi(value) => value,
                DragQueryFileEntry::Wide(value) => String::from_utf16_lossy(&value).into_bytes(),
            };

            let needed_len = ansi.len() as u32;
            if !lpsz_file.is_null() && cch > 0 {
                let ptr = lpsz_file.as_mut_ptr();
                let copy_len = usize::min(ansi.len(), cch.saturating_sub(1) as usize);
                unsafe {
                    std::ptr::copy_nonoverlapping(ansi.as_ptr(), ptr, copy_len);
                    *ptr.add(copy_len) = 0;
                }
            }
            needed_len
        }
        DragQueryFileBuffer::Wide { lpsz_file, cch } => {
            let wide = match entry {
                DragQueryFileEntry::Wide(value) => value,
                DragQueryFileEntry::Ansi(value) => String::from_utf8_lossy(&value)
                    .encode_utf16()
                    .collect::<Vec<u16>>(),
            };

            let needed_len = wide.len() as u32;
            if !lpsz_file.is_null() && cch > 0 {
                let ptr = lpsz_file.as_mut_ptr();
                let copy_len = usize::min(wide.len(), cch.saturating_sub(1) as usize);
                unsafe {
                    std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr, copy_len);
                    *ptr.add(copy_len) = 0;
                }
            }
            needed_len
        }
    }
}
