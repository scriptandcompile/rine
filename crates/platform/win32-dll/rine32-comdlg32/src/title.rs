use rine_common_comdlg32 as common;
use rine_types::strings::{LPCSTR, LPSTR};

/// Retrieves the file title portion of an ANSI path.
///
/// # Arguments
/// * `file_name` - Pointer to a null-terminated ANSI path string.
/// * `title_buffer` - Pointer to an ANSI buffer that receives the extracted file title.
/// * `buffer_len` - Size of `title_buffer`, in characters.
///
/// # Safety
/// * `file_name` must be null or a valid pointer to a null-terminated ANSI string.
/// * `title_buffer` must be null or a valid writable pointer to at least `buffer_len` bytes.
/// * Pointers must remain valid for the duration of this call.
///
/// # Returns
/// * `0` when successful.
/// * A positive size requirement (including NUL terminator) when `title_buffer` is too small or null.
/// * A negative value when input is invalid.
///
/// # Notes
/// This is currently a partial implementation delegated to the common platform layer.
/// It extracts the last path component and does not yet emulate Windows shell display-name behavior.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetFileTitleA(
    file_name: LPCSTR,
    title_buffer: LPSTR,
    buffer_len: u16,
) -> i16 {
    unsafe {
        common::title::get_file_title_a(file_name.as_ptr(), title_buffer.as_mut_ptr(), buffer_len)
    }
}
