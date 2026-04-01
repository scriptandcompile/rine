//! In-memory Windows registry emulation.
//!
//! Provides a hierarchical key-value store matching the Windows registry
//! model.  Predefined root keys (HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER, etc.)
//! are pre-populated with common values that Windows applications query.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

// ---------------------------------------------------------------------------
// Predefined HKEY constants (same values Windows uses)
// ---------------------------------------------------------------------------

/// `HKEY_CLASSES_ROOT`
/// Predefined root keys (HKEY_*) are represented as `isize` values in the Windows API.  
/// We use the same values here so the advapi32 functions can recognize them.
/// These are actually negative values when interpreted as signed integers, but we store them as `isize` for convenience.
pub const HKEY_CLASSES_ROOT: isize = 0x8000_0000_u32 as i32 as isize;
/// `HKEY_CURRENT_USER`
/// Predefined root keys (HKEY_*) are represented as `isize` values in the Windows API.  
/// We use the same values here so the advapi32 functions can recognize them.
/// These are actually negative values when interpreted as signed integers, but we store them as `isize` for convenience.
pub const HKEY_CURRENT_USER: isize = 0x8000_0001_u32 as i32 as isize;
/// `HKEY_LOCAL_MACHINE`
/// Predefined root keys (HKEY_*) are represented as `isize` values in the Windows API.  
/// We use the same values here so the advapi32 functions can recognize them.
/// These are actually negative values when interpreted as signed integers, but we store them as `isize` for convenience.
pub const HKEY_LOCAL_MACHINE: isize = 0x8000_0002_u32 as i32 as isize;
/// `HKEY_USERS`
/// Predefined root keys (HKEY_*) are represented as `isize` values in the Windows API.  
/// We use the same values here so the advapi32 functions can recognize them.
/// These are actually negative values when interpreted as signed integers, but we store them as `isize` for convenience.
pub const HKEY_USERS: isize = 0x8000_0003_u32 as i32 as isize;
/// `HKEY_CURRENT_CONFIG`
/// Predefined root keys (HKEY_*) are represented as `isize` values in the Windows API.  
/// We use the same values here so the advapi32 functions can recognize them.
/// These are actually negative values when interpreted as signed integers, but we store them as `isize` for convenience.
pub const HKEY_CURRENT_CONFIG: isize = 0x8000_0005_u32 as i32 as isize;

// ---------------------------------------------------------------------------
// Registry value types (REG_*)
// ---------------------------------------------------------------------------

pub const REG_NONE: u32 = 0;
pub const REG_SZ: u32 = 1;
pub const REG_EXPAND_SZ: u32 = 2;
pub const REG_BINARY: u32 = 3;
pub const REG_DWORD: u32 = 4;
pub const REG_DWORD_BIG_ENDIAN: u32 = 5;
pub const REG_MULTI_SZ: u32 = 7;
pub const REG_QWORD: u32 = 11;

// ---------------------------------------------------------------------------
// Registry value
// ---------------------------------------------------------------------------

/// A single registry value with its type tag and data.
#[derive(Debug, Clone)]
pub enum RegistryValue {
    /// `REG_SZ` — null-terminated string.
    String(std::string::String),
    /// `REG_EXPAND_SZ` — string with `%VAR%` references.
    ExpandString(std::string::String),
    /// `REG_DWORD` — 32-bit integer.
    Dword(u32),
    /// `REG_QWORD` — 64-bit integer.
    Qword(u64),
    /// `REG_BINARY` — arbitrary bytes.
    Binary(Vec<u8>),
    /// `REG_MULTI_SZ` — list of strings.
    MultiString(Vec<std::string::String>),
}

impl RegistryValue {
    /// Return the `REG_*` type constant for this value.
    pub fn reg_type(&self) -> u32 {
        match self {
            Self::String(_) => REG_SZ,
            Self::ExpandString(_) => REG_EXPAND_SZ,
            Self::Dword(_) => REG_DWORD,
            Self::Qword(_) => REG_QWORD,
            Self::Binary(_) => REG_BINARY,
            Self::MultiString(_) => REG_MULTI_SZ,
        }
    }

    /// Encode the value as raw bytes (UTF-16LE for strings, little-endian
    /// for integers) matching what `RegQueryValueEx` would return.
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::String(s) | Self::ExpandString(s) => {
                let wide: Vec<u16> = s.encode_utf16().chain(std::iter::once(0)).collect();
                wide.iter().flat_map(|w| w.to_le_bytes()).collect()
            }
            Self::Dword(v) => v.to_le_bytes().to_vec(),
            Self::Qword(v) => v.to_le_bytes().to_vec(),
            Self::Binary(b) => b.clone(),
            Self::MultiString(ss) => {
                let mut out: Vec<u16> = Vec::new();
                for s in ss {
                    out.extend(s.encode_utf16());
                    out.push(0);
                }
                out.push(0); // double-null terminator
                out.iter().flat_map(|w| w.to_le_bytes()).collect()
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Registry key node
// ---------------------------------------------------------------------------

/// A single registry key (analogous to a directory in the registry tree).
#[derive(Debug, Clone, Default)]
pub struct RegistryKey {
    /// Named values under this key.  The default value uses an empty
    /// string key `""`.
    pub values: HashMap<String, RegistryValue>,
    /// Sub-keys (case-insensitive; stored with the original case but
    /// looked up via `to_ascii_lowercase()`).
    pub subkeys: HashMap<String, RegistryKey>,
}

impl RegistryKey {
    pub fn new() -> Self {
        Self::default()
    }

    /// Look up a sub-key by path (backslash-separated).  Case-insensitive.
    pub fn open_subkey(&self, path: &str) -> Option<&RegistryKey> {
        let mut current = self;
        for component in path.split('\\').filter(|c| !c.is_empty()) {
            let lower = component.to_ascii_lowercase();
            let found = current
                .subkeys
                .iter()
                .find(|(k, _)| k.to_ascii_lowercase() == lower);
            match found {
                Some((_, child)) => current = child,
                None => return None,
            }
        }
        Some(current)
    }

    /// Look up a sub-key mutably, creating intermediate keys as needed.
    pub fn create_subkey(&mut self, path: &str) -> &mut RegistryKey {
        let mut current = self;
        for component in path.split('\\').filter(|c| !c.is_empty()) {
            let lower = component.to_ascii_lowercase();
            // Find existing key case-insensitively
            let existing_key = current
                .subkeys
                .keys()
                .find(|k| k.to_ascii_lowercase() == lower)
                .cloned();
            if let Some(key) = existing_key {
                current = current.subkeys.get_mut(&key).unwrap();
            } else {
                current = current.subkeys.entry(component.to_string()).or_default();
            }
        }
        current
    }

    /// Get a value by name (case-insensitive).
    pub fn get_value(&self, name: &str) -> Option<&RegistryValue> {
        let lower = name.to_ascii_lowercase();
        self.values
            .iter()
            .find(|(k, _)| k.to_ascii_lowercase() == lower)
            .map(|(_, v)| v)
    }

    /// Set a value (uses the original-case name).
    pub fn set_value(&mut self, name: String, value: RegistryValue) {
        // Remove any existing entry with different case
        let lower = name.to_ascii_lowercase();
        self.values.retain(|k, _| k.to_ascii_lowercase() != lower);
        self.values.insert(name, value);
    }
}

// ---------------------------------------------------------------------------
// Global registry store
// ---------------------------------------------------------------------------

/// The process-wide in-memory registry, keyed by predefined root HKEY.
pub struct RegistryStore {
    inner: Mutex<HashMap<isize, RegistryKey>>,
}

impl RegistryStore {
    fn new() -> Self {
        let mut roots = HashMap::new();

        // Pre-populate with common keys that Windows apps query.
        let mut hklm = RegistryKey::new();
        Self::populate_hklm(&mut hklm);

        let mut hkcu = RegistryKey::new();
        Self::populate_hkcu(&mut hkcu);

        roots.insert(HKEY_LOCAL_MACHINE, hklm);
        roots.insert(HKEY_CURRENT_USER, hkcu);
        roots.insert(HKEY_CLASSES_ROOT, RegistryKey::new());
        roots.insert(HKEY_USERS, RegistryKey::new());
        roots.insert(HKEY_CURRENT_CONFIG, RegistryKey::new());

        Self {
            inner: Mutex::new(roots),
        }
    }

    fn populate_hklm(root: &mut RegistryKey) {
        // SOFTWARE\Microsoft\Windows NT\CurrentVersion
        let cv = root.create_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion");
        cv.set_value(
            "ProductName".into(),
            RegistryValue::String("Windows 10 Pro".into()),
        );
        cv.set_value("CurrentBuild".into(), RegistryValue::String("19045".into()));
        cv.set_value(
            "CurrentBuildNumber".into(),
            RegistryValue::String("19045".into()),
        );
        cv.set_value("CurrentVersion".into(), RegistryValue::String("6.3".into()));
        cv.set_value("CurrentMajorVersionNumber".into(), RegistryValue::Dword(10));
        cv.set_value("CurrentMinorVersionNumber".into(), RegistryValue::Dword(0));
        cv.set_value(
            "SystemRoot".into(),
            RegistryValue::String("C:\\Windows".into()),
        );

        // SYSTEM\CurrentControlSet\Control\Nls\CodePage
        let cp = root.create_subkey("SYSTEM\\CurrentControlSet\\Control\\Nls\\CodePage");
        cp.set_value("ACP".into(), RegistryValue::String("1252".into()));
        cp.set_value("OEMCP".into(), RegistryValue::String("437".into()));

        // SOFTWARE\Microsoft\Windows\CurrentVersion
        let wcv = root.create_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion");
        wcv.set_value(
            "ProgramFilesDir".into(),
            RegistryValue::String("C:\\Program Files".into()),
        );
        wcv.set_value(
            "CommonFilesDir".into(),
            RegistryValue::String("C:\\Program Files\\Common Files".into()),
        );
    }

    fn populate_hkcu(root: &mut RegistryKey) {
        // Environment
        let env = root.create_subkey("Environment");
        env.set_value(
            "TEMP".into(),
            RegistryValue::ExpandString("%USERPROFILE%\\AppData\\Local\\Temp".into()),
        );
        env.set_value(
            "TMP".into(),
            RegistryValue::ExpandString("%USERPROFILE%\\AppData\\Local\\Temp".into()),
        );

        // Software\Microsoft\Windows\CurrentVersion\Explorer\Shell Folders
        let sf = root
            .create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Shell Folders");
        sf.set_value(
            "Desktop".into(),
            RegistryValue::String("C:\\Users\\user\\Desktop".into()),
        );
        sf.set_value(
            "Personal".into(),
            RegistryValue::String("C:\\Users\\user\\Documents".into()),
        );
        sf.set_value(
            "AppData".into(),
            RegistryValue::String("C:\\Users\\user\\AppData\\Roaming".into()),
        );
        sf.set_value(
            "Local AppData".into(),
            RegistryValue::String("C:\\Users\\user\\AppData\\Local".into()),
        );
    }

    /// Run a closure with access to a root key.
    pub fn with_root<F, R>(&self, hkey: isize, f: F) -> Option<R>
    where
        F: FnOnce(&RegistryKey) -> R,
    {
        let inner = self.inner.lock().unwrap();
        inner.get(&hkey).map(f)
    }

    /// Run a closure with mutable access to a root key.
    pub fn with_root_mut<F, R>(&self, hkey: isize, f: F) -> Option<R>
    where
        F: FnOnce(&mut RegistryKey) -> R,
    {
        let mut inner = self.inner.lock().unwrap();
        inner.get_mut(&hkey).map(f)
    }
}

static REGISTRY_STORE: LazyLock<RegistryStore> = LazyLock::new(RegistryStore::new);

/// Access the process-wide registry store.
pub fn registry_store() -> &'static RegistryStore {
    &REGISTRY_STORE
}

/// Check whether an `isize` is a predefined root handle.
pub fn is_predefined_key(hkey: isize) -> bool {
    matches!(
        hkey,
        HKEY_CLASSES_ROOT
            | HKEY_CURRENT_USER
            | HKEY_LOCAL_MACHINE
            | HKEY_USERS
            | HKEY_CURRENT_CONFIG
    )
}

// ---------------------------------------------------------------------------
// Registry key state for opened handles
// ---------------------------------------------------------------------------

/// State for an opened registry key handle (not a predefined root).
///
/// Stores the root HKEY and the sub-key path so the advapi32 functions
/// can resolve queries.
#[derive(Debug, Clone)]
pub struct RegistryKeyState {
    /// Which root this key is under (e.g. `HKEY_LOCAL_MACHINE`).
    pub root: isize,
    /// Sub-key path from root (backslash-separated), or empty for root itself.
    pub path: String,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_key_create_and_lookup() {
        let mut root = RegistryKey::new();
        root.create_subkey("A\\B\\C");
        assert!(root.open_subkey("A\\B\\C").is_some());
        assert!(root.open_subkey("A\\B").is_some());
        assert!(root.open_subkey("A").is_some());
        assert!(root.open_subkey("A\\B\\D").is_none());
    }

    #[test]
    fn registry_key_case_insensitive_lookup() {
        let mut root = RegistryKey::new();
        root.create_subkey("Software\\Microsoft");
        assert!(root.open_subkey("SOFTWARE\\MICROSOFT").is_some());
        assert!(root.open_subkey("software\\microsoft").is_some());
    }

    #[test]
    fn registry_value_set_get() {
        let mut key = RegistryKey::new();
        key.set_value("TestVal".into(), RegistryValue::Dword(42));
        let val = key.get_value("testval").unwrap();
        assert!(matches!(val, RegistryValue::Dword(42)));
    }

    #[test]
    fn registry_value_case_insensitive_replace() {
        let mut key = RegistryKey::new();
        key.set_value("Name".into(), RegistryValue::String("old".into()));
        key.set_value("name".into(), RegistryValue::String("new".into()));
        assert_eq!(key.values.len(), 1);
        assert!(matches!(key.get_value("NAME").unwrap(), RegistryValue::String(s) if s == "new"));
    }

    #[test]
    fn registry_value_dword_bytes() {
        let val = RegistryValue::Dword(0x12345678);
        assert_eq!(val.to_bytes(), vec![0x78, 0x56, 0x34, 0x12]);
        assert_eq!(val.reg_type(), REG_DWORD);
    }

    #[test]
    fn registry_value_qword_bytes() {
        let val = RegistryValue::Qword(0x0102030405060708);
        let bytes = val.to_bytes();
        assert_eq!(bytes.len(), 8);
        assert_eq!(bytes, vec![0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]);
    }

    #[test]
    fn registry_value_string_bytes_utf16() {
        let val = RegistryValue::String("AB".into());
        let bytes = val.to_bytes();
        // 'A' = 0x41, 'B' = 0x42, null = 0x00 — all UTF-16LE
        assert_eq!(bytes, vec![0x41, 0x00, 0x42, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn registry_value_multi_string_bytes() {
        let val = RegistryValue::MultiString(vec!["A".into(), "B".into()]);
        let bytes = val.to_bytes();
        // "A\0B\0\0" in UTF-16LE
        assert_eq!(
            bytes,
            vec![0x41, 0x00, 0x00, 0x00, 0x42, 0x00, 0x00, 0x00, 0x00, 0x00]
        );
    }

    #[test]
    fn global_store_has_hklm() {
        let store = registry_store();
        let result = store.with_root(HKEY_LOCAL_MACHINE, |root| {
            root.open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion")
                .and_then(|k| k.get_value("ProductName"))
                .is_some()
        });
        assert_eq!(result, Some(true));
    }

    #[test]
    fn global_store_has_hkcu() {
        let store = registry_store();
        let result = store.with_root(HKEY_CURRENT_USER, |root| {
            root.open_subkey("Environment")
                .and_then(|k| k.get_value("TEMP"))
                .is_some()
        });
        assert_eq!(result, Some(true));
    }

    #[test]
    fn is_predefined_key_check() {
        assert!(is_predefined_key(HKEY_LOCAL_MACHINE));
        assert!(is_predefined_key(HKEY_CURRENT_USER));
        assert!(!is_predefined_key(0x1000));
        assert!(!is_predefined_key(0));
    }

    #[test]
    fn write_to_global_store() {
        let store = registry_store();
        store.with_root_mut(HKEY_CURRENT_USER, |root| {
            let key = root.create_subkey("Software\\TestApp");
            key.set_value("Setting".into(), RegistryValue::Dword(99));
        });
        let val = store.with_root(HKEY_CURRENT_USER, |root| {
            root.open_subkey("Software\\TestApp")
                .and_then(|k| k.get_value("Setting"))
                .map(|v| matches!(v, RegistryValue::Dword(99)))
        });
        assert_eq!(val, Some(Some(true)));
    }
}
