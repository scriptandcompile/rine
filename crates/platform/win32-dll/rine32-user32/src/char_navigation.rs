use rine_common_user32 as common;

/// Move to the next character in an ANSI string.
///
/// # Arguments
/// * `current` - Pointer to the current byte in a null-terminated ANSI string.
///
/// # Safety
/// `current` must be null or point to readable memory containing a
/// null-terminated ANSI string.
///
/// # Return
/// A pointer to the next character according to ANSI DBCS rules for the active
/// code page. Returns null when `current` is null.
///
/// # Notes
/// This call externally depends on DBCS ANSI code pages (ACPs). The current
/// implementation does not use a fully implemented CPINFO path and instead
/// relies on partial lead-byte range tables. Full compatibility depends on a
/// complete `IsDBCSLeadByte` implementation rather than the partial table path.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn CharNextA(current: *const u8) -> *const u8 {
    unsafe { common::char_next_a(current) }
}