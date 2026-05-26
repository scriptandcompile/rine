use rine_types::strings::{read_cstr, write_cstr};

fn extract_file_title(path: &str) -> Option<&str> {
    if path.is_empty() {
        return None;
    }

    let trimmed = path.trim_end_matches(['/', '\\']);
    if trimmed.is_empty() {
        return None;
    }

    Some(trimmed.rsplit(['/', '\\']).next().unwrap_or(trimmed))
}

/// Retrieves a display-friendly file title from a path string.
///
/// # Arguments
/// * `file_name` - Pointer to a null-terminated ANSI path string.
/// * `title_buffer` - Pointer to a caller-provided ANSI output buffer that receives the title.
/// * `buffer_len` - Size of `title_buffer`, in characters.
///
/// # Safety
/// * `file_name` must be null or point to a valid, null-terminated ANSI string.
/// * `title_buffer` must be null or point to a writable buffer of at least `buffer_len` bytes.
/// * The pointers must remain valid for the duration of the call.
///
/// # Returns
/// * `0` on success.
/// * A positive value when `title_buffer` is null/too small, indicating required size (including NUL).
/// * A negative value on invalid input.
///
/// # Notes
/// This is a partial implementation. It currently returns the last path component split on `/` and `\\`.
/// It does not query shell display-name preferences and may differ from Windows for unusual/illegal file names.
pub unsafe fn get_file_title_a(
    file_name: *const u8,
    title_buffer: *mut u8,
    buffer_len: u16,
) -> i16 {
    let Some(path) = (unsafe { read_cstr(file_name) }) else {
        return -1;
    };

    let Some(title) = extract_file_title(&path) else {
        return -1;
    };

    let required = (title.len() as u32).saturating_add(1);
    let required_short = required.min(i16::MAX as u32) as i16;

    if title_buffer.is_null() || buffer_len == 0 || required > buffer_len as u32 {
        return required_short;
    }

    let _ = unsafe { write_cstr(title_buffer, buffer_len as u32, title) };
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_title_from_windows_path() {
        let input = b"C:\\windows\\notepad.exe\0";
        let mut out = [0_u8; 64];

        let result =
            unsafe { get_file_title_a(input.as_ptr(), out.as_mut_ptr(), out.len() as u16) };

        assert_eq!(result, 0);
        let got = unsafe { read_cstr(out.as_ptr()) };
        assert_eq!(got.as_deref(), Some("notepad.exe"));
    }

    #[test]
    fn returns_required_size_when_buffer_is_too_small() {
        let input = b"C:\\tmp\\hello.txt\0";
        let mut out = [0_u8; 5];

        let result =
            unsafe { get_file_title_a(input.as_ptr(), out.as_mut_ptr(), out.len() as u16) };

        assert_eq!(result, "hello.txt".len() as i16 + 1);
    }

    #[test]
    fn query_size_with_null_output_buffer() {
        let input = b"/tmp/archive.zip\0";

        let result = unsafe { get_file_title_a(input.as_ptr(), core::ptr::null_mut(), 0) };

        assert_eq!(result, "archive.zip".len() as i16 + 1);
    }

    #[test]
    fn null_input_returns_negative_error() {
        let mut out = [0_u8; 8];
        let result =
            unsafe { get_file_title_a(core::ptr::null(), out.as_mut_ptr(), out.len() as u16) };
        assert!(result < 0);
    }

    #[test]
    fn root_only_path_returns_negative_error() {
        let input = b"\\\0";
        let mut out = [0_u8; 8];

        let result =
            unsafe { get_file_title_a(input.as_ptr(), out.as_mut_ptr(), out.len() as u16) };

        assert!(result < 0);
    }
}
