use rine_types::registry::{RegistryValue, registry_store};

/// Read a registry value as either `REG_SZ` or `REG_EXPAND_SZ`.
///
/// Returns `None` if the root/path/value cannot be resolved or if the value is
/// not a string-like registry type.
pub fn read_string_or_expand(root: isize, subkey_path: &str, value_name: &str) -> Option<String> {
    registry_store()
        .with_root(root, |root_key| {
            root_key
                .open_subkey(subkey_path)
                .and_then(|key| key.get_value(value_name))
                .and_then(|value| match value {
                    RegistryValue::String(s) | RegistryValue::ExpandString(s) => Some(s.clone()),
                    _ => None,
                })
        })
        .flatten()
}

/// Parse a registry value into a `u32` from either `REG_DWORD` or a
/// string-like decimal representation (`REG_SZ`/`REG_EXPAND_SZ`).
pub fn parse_u32_from_string_or_dword(value: &RegistryValue) -> Option<u32> {
    match value {
        RegistryValue::String(s) | RegistryValue::ExpandString(s) => s.parse().ok(),
        RegistryValue::Dword(v) => Some(*v),
        _ => None,
    }
}

/// Set a string value under `root\\subkey_path`.
pub fn set_string_value(root: isize, subkey_path: &str, value_name: &str, value: &str) {
    registry_store().with_root_mut(root, |root_key| {
        root_key.create_subkey(subkey_path).set_value(
            value_name.to_string(),
            RegistryValue::String(value.to_string()),
        );
    });
}

/// Delete a value under `root\\subkey_path` using case-insensitive name matching.
pub fn delete_value_case_insensitive(root: isize, subkey_path: &str, value_name: &str) {
    let lower = value_name.to_ascii_lowercase();
    registry_store().with_root_mut(root, |root_key| {
        let subkey = root_key.create_subkey(subkey_path);
        subkey
            .values
            .retain(|name, _| name.to_ascii_lowercase() != lower);
    });
}

#[cfg(test)]
mod tests {
    use super::{
        delete_value_case_insensitive, parse_u32_from_string_or_dword, read_string_or_expand,
        set_string_value,
    };
    use rine_types::registry::{HKEY_CURRENT_USER, RegistryValue, registry_store};

    #[test]
    fn parse_u32_handles_string_and_dword() {
        assert_eq!(
            parse_u32_from_string_or_dword(&RegistryValue::String("123".into())),
            Some(123)
        );
        assert_eq!(
            parse_u32_from_string_or_dword(&RegistryValue::ExpandString("456".into())),
            Some(456)
        );
        assert_eq!(
            parse_u32_from_string_or_dword(&RegistryValue::Dword(789)),
            Some(789)
        );
        assert_eq!(
            parse_u32_from_string_or_dword(&RegistryValue::Binary(vec![1, 2, 3])),
            None
        );
    }

    #[test]
    fn read_string_or_expand_reads_set_string_value() {
        let path = "Software\\rine\\tests\\registry_utils_read";
        set_string_value(HKEY_CURRENT_USER, path, "ExampleName", "ExampleValue");

        let value = read_string_or_expand(HKEY_CURRENT_USER, path, "examplename");
        assert_eq!(value.as_deref(), Some("ExampleValue"));
    }

    #[test]
    fn delete_value_case_insensitive_removes_value() {
        let path = "Software\\rine\\tests\\registry_utils_delete";
        set_string_value(HKEY_CURRENT_USER, path, "MiXeD", "value");
        delete_value_case_insensitive(HKEY_CURRENT_USER, path, "mixed");

        let remaining = registry_store().with_root(HKEY_CURRENT_USER, |root| {
            root.open_subkey(path)
                .and_then(|k| k.get_value("MIXED"))
                .cloned()
        });
        assert!(matches!(remaining, Some(None)));
    }
}
