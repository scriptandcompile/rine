//! ntdll Rtl* utility functions: RtlInitUnicodeString.

use rine_types::strings::UnicodeString;

/// RtlInitUnicodeString — initialise a UNICODE_STRING from a null-terminated
/// wide-character (UTF-16LE) source string.
///
/// # Safety
/// `source` must either be null or point to a valid null-terminated `u16` array.
/// `dest` must be a valid, writable pointer to a `UNICODE_STRING`.
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn RtlInitUnicodeString(dest: *mut UnicodeString, source: *const u16) {
    if dest.is_null() {
        return;
    }

    if source.is_null() {
        unsafe {
            (*dest).length = 0;
            (*dest).maximum_length = 0;
            (*dest).buffer = core::ptr::null_mut();
        }
        return;
    }

    // Count u16 code units up to (but not including) the null terminator.
    let mut len: usize = 0;
    unsafe {
        while *source.add(len) != 0 {
            len += 1;
        }
    }

    // Byte lengths (u16 → 2 bytes each). Cap at u16::MAX.
    let byte_len = (len * 2).min(u16::MAX as usize);
    // maximum_length includes the null terminator (2 extra bytes).
    let max_byte_len = (byte_len + 2).min(u16::MAX as usize);

    unsafe {
        (*dest).length = byte_len as u16;
        (*dest).maximum_length = max_byte_len as u16;
        (*dest).buffer = source as *mut u16;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_unicode_string_from_source() {
        let wide: Vec<u16> = "hello".encode_utf16().chain(std::iter::once(0)).collect();
        let mut us = UnicodeString::empty();

        unsafe { RtlInitUnicodeString(&mut us, wide.as_ptr()) };

        assert_eq!(us.length, 10); // 5 chars × 2 bytes
        assert_eq!(us.maximum_length, 12); // includes null
        assert!(!us.buffer.is_null());
    }

    #[test]
    fn init_unicode_string_null_source() {
        let mut us = UnicodeString::empty();
        unsafe { RtlInitUnicodeString(&mut us, core::ptr::null()) };
        assert_eq!(us.length, 0);
        assert!(us.buffer.is_null());
    }
}
