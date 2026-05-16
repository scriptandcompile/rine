use serde::Serialize;
use std::collections::BTreeMap;

use rine_types::registry::{
    self, HKEY_CLASSES_ROOT, HKEY_CURRENT_CONFIG, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE,
    HKEY_USERS, RegistryKey, RegistryValue,
};

/// A registry value formatted for UI display.
#[derive(Debug, Clone, Serialize)]
pub struct RegistryValueUI {
    /// Value name (empty string for default value)
    pub name: String,
    /// Value type as string ("SZ", "DWORD", "QWORD", etc.)
    pub type_name: String,
    /// Value data as display string
    pub data: String,
    /// Whether this value is locked to the Windows version
    pub locked: bool,
}

/// A registry key node formatted for UI display.
#[derive(Debug, Clone, Serialize)]
pub struct RegistryKeyUI {
    /// Key path (e.g., "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion")
    pub path: String,
    /// Values in this key
    pub values: Vec<RegistryValueUI>,
    /// Subkey names (not fully expanded; expanded on demand by frontend)
    pub subkey_names: Vec<String>,
}

/// Full registry export for UI display.
#[derive(Debug, Clone, Serialize)]
pub struct RegistryExportUI {
    /// Root key snapshots keyed by root name ("HKEY_LOCAL_MACHINE", etc.)
    pub roots: BTreeMap<String, RegistryKeyUI>,
    /// Set of registry paths that are locked to the Windows version.
    /// Paths use backslash separators, e.g., "HKEY_LOCAL_MACHINE\\...\\Value"
    pub locked_paths: Vec<String>,
}

/// Get the registry export for UI display.
pub fn get_registry_export_for_ui() -> RegistryExportUI {
    let store = registry::registry_store();
    let locked_paths = get_locked_registry_paths();

    let mut roots = BTreeMap::new();

    let root_names = [
        (HKEY_LOCAL_MACHINE, "HKEY_LOCAL_MACHINE"),
        (HKEY_CURRENT_USER, "HKEY_CURRENT_USER"),
        (HKEY_CLASSES_ROOT, "HKEY_CLASSES_ROOT"),
        (HKEY_USERS, "HKEY_USERS"),
        (HKEY_CURRENT_CONFIG, "HKEY_CURRENT_CONFIG"),
    ];

    for (hkey, root_name) in root_names {
        let key_ui = store.with_root(hkey, |root| registry_key_to_ui(root, root_name));
        if let Some(key_ui) = key_ui {
            roots.insert(root_name.to_string(), key_ui);
        }
    }

    let prefixed_locked = locked_paths
        .iter()
        .map(|p| {
            if p.contains('\\') {
                p.clone()
            } else {
                format!("HKEY_LOCAL_MACHINE\\{}", p)
            }
        })
        .collect();

    RegistryExportUI {
        roots,
        locked_paths: prefixed_locked,
    }
}

/// Get a single registry key snapshot for UI display by full key path.
pub fn get_registry_key_for_ui(path: &str) -> Option<RegistryKeyUI> {
    let (root_hkey, root_name, subpath) = parse_registry_ui_path(path)?;
    let store = registry::registry_store();

    store.with_root(root_hkey, |root| {
        if subpath.is_empty() {
            return Some(registry_key_to_ui(root, root_name));
        }

        root.open_subkey(subpath)
            .map(|key| registry_key_to_ui(key, path))
    })?
}

/// Check if a specific registry value is locked (read-only due to Windows version).
pub fn is_locked_registry_value(key_path: &str, value_name: &str) -> bool {
    let locked_entries = [
        (
            "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion",
            "CurrentBuild",
        ),
        (
            "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion",
            "CurrentBuildNumber",
        ),
        (
            "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion",
            "CurrentVersion",
        ),
        (
            "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion",
            "ProductName",
        ),
        (
            "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion",
            "CurrentMajorVersionNumber",
        ),
        (
            "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion",
            "CurrentMinorVersionNumber",
        ),
    ];

    locked_entries.iter().any(|(locked_key, locked_value)| {
        key_path.eq_ignore_ascii_case(locked_key) && value_name.eq_ignore_ascii_case(locked_value)
    })
}

fn parse_registry_ui_path(path: &str) -> Option<(isize, &'static str, &str)> {
    let trimmed = path.trim_matches('\\');
    if trimmed.is_empty() {
        return None;
    }

    let (root_part, subpath) = match trimmed.split_once('\\') {
        Some((root, rest)) => (root, rest),
        None => (trimmed, ""),
    };

    let root_hkey = match root_part.to_ascii_uppercase().as_str() {
        "HKEY_LOCAL_MACHINE" | "HKLM" => HKEY_LOCAL_MACHINE,
        "HKEY_CURRENT_USER" | "HKCU" => HKEY_CURRENT_USER,
        "HKEY_CLASSES_ROOT" | "HKCR" => HKEY_CLASSES_ROOT,
        "HKEY_USERS" | "HKU" => HKEY_USERS,
        "HKEY_CURRENT_CONFIG" | "HKCC" => HKEY_CURRENT_CONFIG,
        _ => return None,
    };

    let canonical_root = match root_hkey {
        HKEY_LOCAL_MACHINE => "HKEY_LOCAL_MACHINE",
        HKEY_CURRENT_USER => "HKEY_CURRENT_USER",
        HKEY_CLASSES_ROOT => "HKEY_CLASSES_ROOT",
        HKEY_USERS => "HKEY_USERS",
        HKEY_CURRENT_CONFIG => "HKEY_CURRENT_CONFIG",
        _ => return None,
    };

    Some((root_hkey, canonical_root, subpath))
}

fn registry_key_to_ui(key: &RegistryKey, path: &str) -> RegistryKeyUI {
    let values = key
        .values
        .iter()
        .map(|(name, value)| RegistryValueUI {
            name: name.clone(),
            type_name: value_type_name(value),
            data: value_to_display_string(value),
            locked: is_locked_registry_value(path, name),
        })
        .collect();

    let subkey_names = key.subkeys.keys().cloned().collect();

    RegistryKeyUI {
        path: path.to_string(),
        values,
        subkey_names,
    }
}

fn value_to_display_string(value: &RegistryValue) -> String {
    match value {
        RegistryValue::String(s) | RegistryValue::ExpandString(s) => s.clone(),
        RegistryValue::Dword(d) => d.to_string(),
        RegistryValue::Qword(q) => q.to_string(),
        RegistryValue::Binary(b) => b
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<Vec<_>>()
            .join(" "),
        RegistryValue::MultiString(ss) => ss.join("; "),
    }
}

fn value_type_name(value: &RegistryValue) -> String {
    match value {
        RegistryValue::String(_) => "REG_SZ".into(),
        RegistryValue::ExpandString(_) => "REG_EXPAND_SZ".into(),
        RegistryValue::Dword(_) => "REG_DWORD".into(),
        RegistryValue::Qword(_) => "REG_QWORD".into(),
        RegistryValue::Binary(_) => "REG_BINARY".into(),
        RegistryValue::MultiString(_) => "REG_MULTI_SZ".into(),
    }
}

fn get_locked_registry_paths() -> Vec<String> {
    vec![
        "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\CurrentBuild",
        "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\CurrentBuildNumber",
        "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\CurrentVersion",
        "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\ProductName",
        "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\CurrentMajorVersionNumber",
        "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\CurrentMinorVersionNumber",
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect()
}
