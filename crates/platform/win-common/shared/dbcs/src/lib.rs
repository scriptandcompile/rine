use rine_types::registry::{HKEY_LOCAL_MACHINE, RegistryValue, registry_store};

pub const CP_ACP: u32 = 0;
pub const CP_OEMCP: u32 = 1;
pub const CP_THREAD_ACP: u32 = 3;

const DEFAULT_ANSI_CODE_PAGE: u32 = 1252;
const DEFAULT_OEM_CODE_PAGE: u32 = 437;
const ANSI_CODE_PAGE_REG_PATH: &str = "SYSTEM\\CurrentControlSet\\Control\\Nls\\CodePage";

/// Resolve a Win32 code page value, including ACP aliases.
///
/// # Arguments
/// * `code_page` - The requested code page value.
///
/// # Returns
/// A concrete code page number. `CP_ACP`, `CP_THREAD_ACP`, and `CP_OEMCP`
/// are resolved from the emulated registry with Windows-style defaults.
pub fn resolve_code_page(code_page: u32) -> u32 {
    match code_page {
        CP_ACP | CP_THREAD_ACP => {
            resolve_registry_code_page("ACP").unwrap_or(DEFAULT_ANSI_CODE_PAGE)
        }
        CP_OEMCP => resolve_registry_code_page("OEMCP").unwrap_or(DEFAULT_OEM_CODE_PAGE),
        other => other,
    }
}

/// Determine whether a byte is a DBCS lead byte for a Win32 code page value.
///
/// # Arguments
/// * `code_page` - A Win32 code page value. ACP aliases are accepted.
/// * `test_char` - Byte value to classify.
///
/// # Returns
/// `true` when `test_char` is a lead byte in the resolved code page;
/// otherwise `false`.
///
/// # Notes
/// Partial implementation: only known DBCS lead-byte ranges are modeled.
/// Unsupported code pages currently return `false`.
pub fn is_dbcs_lead_byte_ex(code_page: u32, test_char: u8) -> bool {
    let resolved_code_page = resolve_code_page(code_page);
    is_dbcs_lead_byte_for_code_page(resolved_code_page, test_char)
}

/// Determine whether a byte is a DBCS lead byte for a concrete code page.
///
/// # Arguments
/// * `code_page` - Concrete numeric code page.
/// * `byte` - Byte value to classify.
///
/// # Returns
/// `true` when `byte` is a known lead byte for `code_page`; otherwise `false`.
pub fn is_dbcs_lead_byte_for_code_page(code_page: u32, byte: u8) -> bool {
    match code_page {
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

fn resolve_registry_code_page(value_name: &str) -> Option<u32> {
    registry_store()
        .with_root(HKEY_LOCAL_MACHINE, |root| {
            root.open_subkey(ANSI_CODE_PAGE_REG_PATH)
                .and_then(|code_page| code_page.get_value(value_name))
                .and_then(|value| match value {
                    RegistryValue::String(s) | RegistryValue::ExpandString(s) => s.parse().ok(),
                    RegistryValue::Dword(v) => Some(*v),
                    _ => None,
                })
        })
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shift_jis_lead_returns_true() {
        assert!(is_dbcs_lead_byte_ex(932, 0x81));
    }

    #[test]
    fn shift_jis_trailing_returns_false() {
        assert!(!is_dbcs_lead_byte_ex(932, 0x40));
    }

    #[test]
    fn single_byte_code_page_returns_false() {
        assert!(!is_dbcs_lead_byte_ex(1252, 0x81));
    }

    #[test]
    fn utf8_returns_false() {
        assert!(!is_dbcs_lead_byte_ex(65001, 0xE2));
    }

    #[test]
    fn acp_defaults_to_non_dbcs() {
        assert!(!is_dbcs_lead_byte_ex(CP_ACP, 0x81));
    }

    #[test]
    fn thread_acp_defaults_to_non_dbcs() {
        assert!(!is_dbcs_lead_byte_ex(CP_THREAD_ACP, 0x81));
    }
}
