use rine_common_user32 as common;
use rine_types::strings::LPCSTR;

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
/// implementation routes lead-byte classification through the shared common
/// DBCS helper used by both user32 and `kernel32!IsDBCSLeadByteEx`.
/// Full compatibility still depends on a complete CPINFO-backed path.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn CharNextA(current: LPCSTR) -> LPCSTR {
    unsafe { common::char_next_a(current) }
}

/// Move to the previous character in an ANSI string.
///
/// # Arguments
/// * `start` - Pointer to the first byte of the null-terminated ANSI string.
/// * `current` - Pointer to a byte position within the same ANSI string.
///
/// # Safety
/// `start` and `current` must be null or point into readable memory in the
/// same null-terminated ANSI string.
///
/// # Return
/// A pointer to the preceding character boundary in the string, or `start` when
/// `current` is less than or equal to `start`.
///
/// # Notes
/// This call externally depends on DBCS ANSI code pages (ACPs). The current
/// implementation routes lead-byte classification through the shared common
/// DBCS helper used by both user32 and `kernel32!IsDBCSLeadByteEx`.
/// Full compatibility still depends on a complete CPINFO-backed path.
///
/// The implementation chooses correctness over speed by scanning forward from
/// `start` to recover the previous boundary (O(n)). A backward heuristic may be
/// needed later if profiling shows this to be a hot path.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn CharPrevA(start: LPCSTR, current: LPCSTR) -> LPCSTR {
    unsafe { common::char_prev_a(start, current) }
}
