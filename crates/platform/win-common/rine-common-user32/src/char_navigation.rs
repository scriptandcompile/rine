//! Character navigation helpers for ANSI user32 APIs.

use std::sync::OnceLock;

use registry_runtime::registry_store;
use rine_types::registry::{HKEY_LOCAL_MACHINE, RegistryValue};
use rine_types::strings::LPCSTR;

const DEFAULT_ANSI_CODE_PAGE: u32 = 1252;
const ANSI_CODE_PAGE_REG_PATH: &str = "SYSTEM\\CurrentControlSet\\Control\\Nls\\CodePage";

static ANSI_CODE_PAGE: OnceLock<u32> = OnceLock::new();

/// CharNextA-compatible advancement over an ANSI string.
///
/// # Safety
/// `current` must be null or point to readable memory containing a valid
/// null-terminated byte string.
pub unsafe fn char_next_a(current: LPCSTR) -> LPCSTR {
    let next = unsafe { char_next_a_with_acp(current.as_ptr(), cached_ansi_code_page()) };
    ptr_to_lpcstr(next)
}

/// CharPrevA-compatible rewind over an ANSI string.
///
/// # Safety
/// `start` and `current` must be null or point into readable memory in the
/// same null-terminated byte string.
pub unsafe fn char_prev_a(start: LPCSTR, current: LPCSTR) -> LPCSTR {
    let previous =
        unsafe { char_prev_a_with_acp(start.as_ptr(), current.as_ptr(), cached_ansi_code_page()) };
    ptr_to_lpcstr(previous)
}

fn ptr_to_lpcstr(ptr: *const u8) -> LPCSTR {
    // LPCSTR is a transparent newtype around *const u8.
    unsafe { std::mem::transmute::<*const u8, LPCSTR>(ptr) }
}

fn cached_ansi_code_page() -> u32 {
    *ANSI_CODE_PAGE.get_or_init(resolve_ansi_code_page)
}

fn resolve_ansi_code_page() -> u32 {
    registry_store()
        .with_root(HKEY_LOCAL_MACHINE, |root| {
            root.open_subkey(ANSI_CODE_PAGE_REG_PATH)
                .and_then(|code_page| code_page.get_value("ACP"))
                .and_then(|value| match value {
                    RegistryValue::String(s) | RegistryValue::ExpandString(s) => s.parse().ok(),
                    RegistryValue::Dword(v) => Some(*v),
                    _ => None,
                })
        })
        .flatten()
        .unwrap_or(DEFAULT_ANSI_CODE_PAGE)
}

unsafe fn char_next_a_with_acp(current: *const u8, acp: u32) -> *const u8 {
    if current.is_null() {
        return current;
    }

    let byte = unsafe { *current };
    if byte == 0 {
        return current;
    }

    if dbcs::is_dbcs_lead_byte_for_code_page(acp, byte) {
        let next = unsafe { *current.add(1) };
        if next != 0 {
            return unsafe { current.add(2) };
        }
    }

    unsafe { current.add(1) }
}

unsafe fn char_prev_a_with_acp(start: *const u8, current: *const u8, acp: u32) -> *const u8 {
    if start.is_null() {
        return start;
    }

    if current.is_null() || current <= start {
        return start;
    }

    // Correctness-first strategy: scan from `start` to recover the previous
    // valid character boundary before `current` under the active ACP.
    // This is O(n), but robust for DBCS boundary recovery (including
    // misaligned interior pointers). A faster backward heuristic may be needed
    // later if profiling shows this to be hot.
    let mut cursor = start;
    let mut previous_boundary = start;

    while cursor < current {
        let next = unsafe { char_next_a_with_acp(cursor, acp) };
        if next <= cursor {
            break;
        }

        if next >= current {
            return cursor;
        }

        previous_boundary = cursor;
        cursor = next;
    }

    previous_boundary
}
#[cfg(test)]
mod tests {
    use super::*;

    fn set_registry_acp(value: &str) {
        registry_store().with_root_mut(HKEY_LOCAL_MACHINE, |root| {
            let cp = root.create_subkey(ANSI_CODE_PAGE_REG_PATH);
            cp.set_value("ACP".into(), RegistryValue::String(value.into()));
        });
    }

    #[test]
    fn char_next_null_pointer_returns_null() {
        let next = unsafe { char_next_a_with_acp(std::ptr::null(), 932) };
        assert!(next.is_null());
    }

    #[test]
    fn char_next_nul_stays_in_place() {
        let bytes = [0u8];
        let ptr = bytes.as_ptr();
        let next = unsafe { char_next_a_with_acp(ptr, 932) };
        assert_eq!(next, ptr);
    }

    #[test]
    fn char_next_non_dbcs_advances_one_byte() {
        let bytes = [b'A', 0u8];
        let ptr = bytes.as_ptr();
        let next = unsafe { char_next_a_with_acp(ptr, 1252) };
        assert_eq!(unsafe { next.offset_from(ptr) }, 1);
    }

    #[test]
    fn char_next_dbcs_pair_advances_two_bytes() {
        let bytes = [0x81u8, 0x40u8, 0u8];
        let ptr = bytes.as_ptr();
        let next = unsafe { char_next_a_with_acp(ptr, 932) };
        assert_eq!(unsafe { next.offset_from(ptr) }, 2);
    }

    #[test]
    fn char_next_orphan_lead_byte_advances_to_nul() {
        let bytes = [0x81u8, 0u8];
        let ptr = bytes.as_ptr();
        let next = unsafe { char_next_a_with_acp(ptr, 932) };
        assert_eq!(unsafe { next.offset_from(ptr) }, 1);
    }

    #[test]
    fn char_next_uses_cached_acp() {
        set_registry_acp("1252");

        let first = [0x81u8, 0x40u8, 0u8];
        let first_ptr = first.as_ptr();
        let first_next = unsafe { char_next_a(ptr_to_lpcstr(first_ptr)) };
        assert_eq!(unsafe { first_next.as_ptr().offset_from(first_ptr) }, 1);

        // Once loaded, ACP should remain stable for the process lifetime.
        set_registry_acp("932");

        let second = [0x81u8, 0x40u8, 0u8];
        let second_ptr = second.as_ptr();
        let second_next = unsafe { char_next_a(ptr_to_lpcstr(second_ptr)) };
        assert_eq!(unsafe { second_next.as_ptr().offset_from(second_ptr) }, 1);
    }

    #[test]
    fn char_prev_null_start_returns_null() {
        let current = [b'A', 0u8];
        let prev = unsafe { char_prev_a_with_acp(std::ptr::null(), current.as_ptr(), 932) };
        assert!(prev.is_null());
    }

    #[test]
    fn char_prev_current_at_or_before_start_clamps_to_start() {
        let bytes = [b'A', b'B', 0u8];
        let start = bytes.as_ptr();

        let at_start = unsafe { char_prev_a_with_acp(start, start, 932) };
        assert_eq!(at_start, start);

        let before_start = unsafe { char_prev_a_with_acp(start, start.wrapping_sub(1), 932) };
        assert_eq!(before_start, start);
    }

    #[test]
    fn char_prev_ascii_returns_previous_character() {
        let bytes = [b'A', b'B', b'C', 0u8];
        let start = bytes.as_ptr();
        let current = unsafe { start.add(2) };
        let prev = unsafe { char_prev_a_with_acp(start, current, 1252) };
        assert_eq!(unsafe { prev.offset_from(start) }, 1);
    }

    #[test]
    fn char_prev_dbcs_boundary_steps_to_lead_byte() {
        let bytes = [b'A', 0x81u8, 0x40u8, b'B', 0u8];
        let start = bytes.as_ptr();
        let current = unsafe { start.add(3) };
        let prev = unsafe { char_prev_a_with_acp(start, current, 932) };
        assert_eq!(unsafe { prev.offset_from(start) }, 1);
    }

    #[test]
    fn char_prev_misaligned_trailing_byte_repairs_to_lead() {
        let bytes = [b'A', 0x81u8, 0x40u8, b'B', 0u8];
        let start = bytes.as_ptr();
        let current = unsafe { start.add(2) };
        let prev = unsafe { char_prev_a_with_acp(start, current, 932) };
        assert_eq!(unsafe { prev.offset_from(start) }, 1);
    }

    #[test]
    fn char_prev_uses_cached_acp() {
        set_registry_acp("1252");

        let first = [b'A', 0x81u8, 0x40u8, b'B', 0u8];
        let first_start = first.as_ptr();
        let first_current = unsafe { first_start.add(3) };
        let first_prev =
            unsafe { char_prev_a(ptr_to_lpcstr(first_start), ptr_to_lpcstr(first_current)) };
        assert_eq!(unsafe { first_prev.as_ptr().offset_from(first_start) }, 2);

        // Once loaded, ACP should remain stable for the process lifetime.
        set_registry_acp("932");

        let second = [b'A', 0x81u8, 0x40u8, b'B', 0u8];
        let second_start = second.as_ptr();
        let second_current = unsafe { second_start.add(3) };
        let second_prev =
            unsafe { char_prev_a(ptr_to_lpcstr(second_start), ptr_to_lpcstr(second_current)) };
        assert_eq!(unsafe { second_prev.as_ptr().offset_from(second_start) }, 2);
    }
}
