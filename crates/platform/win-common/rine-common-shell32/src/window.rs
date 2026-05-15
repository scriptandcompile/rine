use rine_types::errors::BOOL;
use rine_types::strings::{LPSTR, LPWSTR};
use rine_types::windows::{DropFiles, HDROP, HWND, WINDOW_MANAGER, window_style_ex};

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
pub fn drag_accept_files(hwnd: HWND, accept: BOOL) {
    let _ = WINDOW_MANAGER.update_window(hwnd, |state| {
        if accept.is_true() {
            state.ex_style |= window_style_ex::WS_EX_ACCEPTFILES;
        } else {
            state.ex_style &= !window_style_ex::WS_EX_ACCEPTFILES;
        }
    });
}

/// Releases a shell drag-and-drop handle.
///
/// # Arguments
/// * `hdrop` - Handle identifying the dropped-file list.
///
/// # Safety
/// `hdrop` must either be `HDROP::NULL` or a pointer allocated with a
/// C-compatible allocator for drag-drop data.
pub unsafe fn drag_finish(hdrop: HDROP) {
    if hdrop.is_null() {
        return;
    }

    unsafe {
        libc::free(hdrop.as_raw() as *mut libc::c_void);
    }
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

#[cfg(test)]
mod tests {
    use super::{DragQueryFileBuffer, drag_query_file};
    use rine_types::errors::BOOL;
    use rine_types::strings::{LPSTR, LPWSTR};
    use rine_types::windows::{DropFiles, HDROP, Point};

    #[repr(C)]
    struct AnsiDropBlock<const N: usize> {
        header: DropFiles,
        files: [u8; N],
    }

    #[repr(C)]
    struct WideDropBlock<const N: usize> {
        header: DropFiles,
        files: [u16; N],
    }

    fn hdrop_from_ref<T>(value: &T) -> HDROP {
        HDROP::from_raw(value as *const T as usize)
    }

    #[test]
    fn drag_query_file_ansi_count_length_and_copy() {
        let block = AnsiDropBlock {
            header: DropFiles {
                p_files: std::mem::size_of::<DropFiles>() as u32,
                pt: Point { x: 0, y: 0 },
                f_nc: BOOL::FALSE,
                f_wide: BOOL::FALSE,
            },
            files: *b"first.txt\0second.bin\0\0",
        };

        let hdrop = hdrop_from_ref(&block);

        let count = unsafe {
            drag_query_file(
                hdrop,
                0xFFFF_FFFF,
                DragQueryFileBuffer::Ansi {
                    lpsz_file: LPSTR::NULL,
                    cch: 0,
                },
            )
        };
        assert_eq!(count, 2);

        let second_len = unsafe {
            drag_query_file(
                hdrop,
                1,
                DragQueryFileBuffer::Ansi {
                    lpsz_file: LPSTR::NULL,
                    cch: 0,
                },
            )
        };
        assert_eq!(second_len, 10);

        let mut out = [0u8; 7];
        let copied_len = unsafe {
            drag_query_file(
                hdrop,
                1,
                DragQueryFileBuffer::Ansi {
                    // LPSTR has no public constructor; repr(C) wrapper conversion is used in tests only.
                    lpsz_file: std::mem::transmute::<*mut u8, LPSTR>(out.as_mut_ptr()),
                    cch: out.len() as u32,
                },
            )
        };
        assert_eq!(copied_len, 10);
        assert_eq!(&out[..6], b"second");
        assert_eq!(out[6], 0);

        let missing_len = unsafe {
            drag_query_file(
                hdrop,
                3,
                DragQueryFileBuffer::Ansi {
                    lpsz_file: LPSTR::NULL,
                    cch: 0,
                },
            )
        };
        assert_eq!(missing_len, 0);
    }

    #[test]
    fn drag_query_file_wide_count_and_wide_copy() {
        let block = WideDropBlock {
            header: DropFiles {
                p_files: std::mem::size_of::<DropFiles>() as u32,
                pt: Point { x: 0, y: 0 },
                f_nc: BOOL::FALSE,
                f_wide: BOOL::TRUE,
            },
            files: [
                b'w' as u16,
                b'i' as u16,
                b'd' as u16,
                b'e' as u16,
                b'.' as u16,
                b't' as u16,
                b'x' as u16,
                b't' as u16,
                0,
                b'o' as u16,
                b't' as u16,
                b'h' as u16,
                b'e' as u16,
                b'r' as u16,
                b'.' as u16,
                b'l' as u16,
                b'o' as u16,
                b'g' as u16,
                0,
                0,
            ],
        };

        let hdrop = hdrop_from_ref(&block);

        let count = unsafe {
            drag_query_file(
                hdrop,
                0xFFFF_FFFF,
                DragQueryFileBuffer::Wide {
                    lpsz_file: LPWSTR::NULL,
                    cch: 0,
                },
            )
        };
        assert_eq!(count, 2);

        let mut out = [0u16; 10];
        let copied_len = unsafe {
            drag_query_file(
                hdrop,
                0,
                DragQueryFileBuffer::Wide {
                    // LPWSTR has no public constructor; repr(C) wrapper conversion is used in tests only.
                    lpsz_file: std::mem::transmute::<*mut u16, LPWSTR>(out.as_mut_ptr()),
                    cch: out.len() as u32,
                },
            )
        };
        assert_eq!(copied_len, 8);
        assert_eq!(out[0], b'w' as u16);
        assert_eq!(out[7], b't' as u16);
        assert_eq!(out[8], 0);
    }
}
