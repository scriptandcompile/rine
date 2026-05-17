//! Character navigation helpers for ANSI user32 APIs.

use std::sync::OnceLock;

use rine_types::registry::{HKEY_LOCAL_MACHINE, RegistryValue, registry_store};

const DEFAULT_ANSI_CODE_PAGE: u32 = 1252;
const ANSI_CODE_PAGE_REG_PATH: &str = "SYSTEM\\CurrentControlSet\\Control\\Nls\\CodePage";

static ANSI_CODE_PAGE: OnceLock<u32> = OnceLock::new();

/// CharNextA-compatible advancement over an ANSI string.
///
/// # Safety
/// `current` must be null or point to readable memory containing a valid
/// null-terminated byte string.
pub unsafe fn char_next_a(current: *const u8) -> *const u8 {
    unsafe { char_next_a_with_acp(current, cached_ansi_code_page()) }
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

    if is_dbcs_lead_byte(acp, byte) {
        let next = unsafe { *current.add(1) };
        if next != 0 {
            return unsafe { current.add(2) };
        }
    }

    unsafe { current.add(1) }
}

fn is_dbcs_lead_byte(acp: u32, byte: u8) -> bool {
    match acp {
        // Shift-JIS
        932 => (0x81..=0x9F).contains(&byte) || (0xE0..=0xFC).contains(&byte),
        // GBK
        936 => (0x81..=0xFE).contains(&byte),
        // Korean
        949 => (0x81..=0xFE).contains(&byte),
        // Big5
        950 => (0x81..=0xFE).contains(&byte),
        // Johab
        1361 => (0x84..=0xD3).contains(&byte) || (0xD8..=0xDE).contains(&byte),
        _ => false,
    }
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
        let first_next = unsafe { char_next_a(first_ptr) };
        assert_eq!(unsafe { first_next.offset_from(first_ptr) }, 1);

        // Once loaded, ACP should remain stable for the process lifetime.
        set_registry_acp("932");

        let second = [0x81u8, 0x40u8, 0u8];
        let second_ptr = second.as_ptr();
        let second_next = unsafe { char_next_a(second_ptr) };
        assert_eq!(unsafe { second_next.offset_from(second_ptr) }, 1);
    }
}