//! In-memory Windows registry emulation.
//!
//! Provides a hierarchical key-value store matching the Windows registry
//! model.  Predefined root keys (HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER, etc.)
//! are pre-populated with common values that Windows applications query.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
        // Default fallback: minimal registry with basic keys.
        // When config feature is enabled, init_registry_for_app should be called
        // to load version-specific defaults from the JSON file.
        let mut roots = HashMap::new();

        let mut hklm = RegistryKey::new();
        // Basic HKLM keys without version-specific data
        let cv = hklm.create_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion");
        cv.set_value(
            "SystemRoot".into(),
            RegistryValue::String("C:\\Windows".into()),
        );
        cv.set_value(
            "ProductName".into(),
            RegistryValue::String("Windows".into()),
        );
        cv.set_value("CurrentBuild".into(), RegistryValue::String("19045".into()));
        cv.set_value(
            "CurrentBuildNumber".into(),
            RegistryValue::String("19045".into()),
        );
        cv.set_value("CurrentVersion".into(), RegistryValue::String("6.3".into()));
        cv.set_value("CurrentMajorVersionNumber".into(), RegistryValue::Dword(10));
        cv.set_value("CurrentMinorVersionNumber".into(), RegistryValue::Dword(0));
        let cp = hklm.create_subkey("SYSTEM\\CurrentControlSet\\Control\\Nls\\CodePage");
        cp.set_value("ACP".into(), RegistryValue::String("1252".into()));
        cp.set_value("OEMCP".into(), RegistryValue::String("437".into()));
        let wcv = hklm.create_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion");
        wcv.set_value(
            "ProgramFilesDir".into(),
            RegistryValue::String("C:\\Program Files".into()),
        );
        wcv.set_value(
            "CommonFilesDir".into(),
            RegistryValue::String("C:\\Program Files\\Common Files".into()),
        );

        let mut hkcu = RegistryKey::new();
        let env = hkcu.create_subkey("Environment");
        env.set_value(
            "TEMP".into(),
            RegistryValue::ExpandString("%USERPROFILE%\\AppData\\Local\\Temp".into()),
        );
        env.set_value(
            "TMP".into(),
            RegistryValue::ExpandString("%USERPROFILE%\\AppData\\Local\\Temp".into()),
        );
        let sf = hkcu
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

        roots.insert(HKEY_LOCAL_MACHINE, hklm);
        roots.insert(HKEY_CURRENT_USER, hkcu);
        roots.insert(HKEY_CLASSES_ROOT, RegistryKey::new());
        roots.insert(HKEY_USERS, RegistryKey::new());
        roots.insert(HKEY_CURRENT_CONFIG, RegistryKey::new());

        Self {
            inner: Mutex::new(roots),
        }
    }

    #[cfg(feature = "config")]
    fn new_for_version_data(ver: VersionDefaults) -> Self {
        let mut roots = HashMap::new();

        let mut hklm = RegistryKey::new();
        populate_hklm(&mut hklm, ver);

        let mut hkcu = RegistryKey::new();
        populate_hkcu(&mut hkcu);

        roots.insert(HKEY_LOCAL_MACHINE, hklm);
        roots.insert(HKEY_CURRENT_USER, hkcu);
        roots.insert(HKEY_CLASSES_ROOT, RegistryKey::new());
        roots.insert(HKEY_USERS, RegistryKey::new());
        roots.insert(HKEY_CURRENT_CONFIG, RegistryKey::new());

        Self {
            inner: Mutex::new(roots),
        }
    }

    #[cfg(feature = "config")]
    fn from_roots(roots: HashMap<isize, RegistryKey>) -> Self {
        Self {
            inner: Mutex::new(roots),
        }
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

// Version data used to populate the registry defaults.
#[cfg(feature = "config")]
struct VersionDefaults {
    product_name: &'static str,
    build: &'static str,
    current_version: &'static str,
    major: u32,
    minor: u32,
}

#[cfg(feature = "config")]
impl VersionDefaults {
    const WIN_XP: Self = Self {
        product_name: "Windows XP Professional",
        build: "2600",
        current_version: "5.1",
        major: 5,
        minor: 1,
    };
    const WIN7: Self = Self {
        product_name: "Windows 7 Professional",
        build: "7601",
        current_version: "6.1",
        major: 6,
        minor: 1,
    };
    const WIN10: Self = Self {
        product_name: "Windows 10 Pro",
        build: "19045",
        current_version: "6.3",
        major: 10,
        minor: 0,
    };
    const WIN11: Self = Self {
        product_name: "Windows 11 Pro",
        build: "22631",
        current_version: "6.3",
        major: 10,
        minor: 0,
    };
}

#[cfg(feature = "config")]
fn populate_hklm(root: &mut RegistryKey, ver: VersionDefaults) {
    // SOFTWARE\Microsoft\Windows NT\CurrentVersion
    let cv = root.create_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion");
    cv.set_value(
        "ProductName".into(),
        RegistryValue::String(ver.product_name.into()),
    );
    cv.set_value(
        "CurrentBuild".into(),
        RegistryValue::String(ver.build.into()),
    );
    cv.set_value(
        "CurrentBuildNumber".into(),
        RegistryValue::String(ver.build.into()),
    );
    cv.set_value(
        "CurrentVersion".into(),
        RegistryValue::String(ver.current_version.into()),
    );
    cv.set_value(
        "CurrentMajorVersionNumber".into(),
        RegistryValue::Dword(ver.major),
    );
    cv.set_value(
        "CurrentMinorVersionNumber".into(),
        RegistryValue::Dword(ver.minor),
    );
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

#[cfg(feature = "config")]
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
    let sf =
        root.create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Shell Folders");
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

static REGISTRY_STORE: OnceLock<RegistryStore> = OnceLock::new();

/// Access the process-wide registry store.
///
/// If [`init_registry_for_app`] was not called first, returns a store
/// pre-populated with Win11 defaults.
pub fn registry_store() -> &'static RegistryStore {
    REGISTRY_STORE.get_or_init(RegistryStore::new)
}

// ---------------------------------------------------------------------------
// Per-app, per-version registry persistence
// ---------------------------------------------------------------------------

/// JSON snapshot of all root keys, used for on-disk storage.
#[cfg(feature = "config")]
#[derive(serde::Serialize, serde::Deserialize, Default)]
struct RegistryStoreSnapshot {
    #[serde(default)]
    hklm: RegistryKey,
    #[serde(default)]
    hkcu: RegistryKey,
    #[serde(default)]
    hkcr: RegistryKey,
    #[serde(default)]
    hku: RegistryKey,
    #[serde(default)]
    hkcc: RegistryKey,
}

#[cfg(feature = "config")]
fn snapshot_to_store(snap: RegistryStoreSnapshot) -> RegistryStore {
    let mut roots = HashMap::new();
    roots.insert(HKEY_LOCAL_MACHINE, snap.hklm);
    roots.insert(HKEY_CURRENT_USER, snap.hkcu);
    roots.insert(HKEY_CLASSES_ROOT, snap.hkcr);
    roots.insert(HKEY_USERS, snap.hku);
    roots.insert(HKEY_CURRENT_CONFIG, snap.hkcc);
    RegistryStore::from_roots(roots)
}

#[cfg(feature = "config")]
fn store_to_snapshot(store: &RegistryStore) -> RegistryStoreSnapshot {
    let inner = store.inner.lock().unwrap();
    RegistryStoreSnapshot {
        hklm: inner.get(&HKEY_LOCAL_MACHINE).cloned().unwrap_or_default(),
        hkcu: inner.get(&HKEY_CURRENT_USER).cloned().unwrap_or_default(),
        hkcr: inner.get(&HKEY_CLASSES_ROOT).cloned().unwrap_or_default(),
        hku: inner.get(&HKEY_USERS).cloned().unwrap_or_default(),
        hkcc: inner.get(&HKEY_CURRENT_CONFIG).cloned().unwrap_or_default(),
    }
}

/// Initialise the process-wide registry store from the per-app JSON file.
///
/// Must be called before any registry access, ideally immediately after the
/// app config is loaded. If the JSON file for this `(exe_path, version)` pair
/// does not exist, a default registry for the given Windows version is written
/// to disk and then loaded. Switching `version` in the config will therefore
/// automatically produce a fresh default file for the new version.
///
/// # Arguments
/// * `exe_path` - Path to the Windows executable being run.
/// * `version` - The Windows version specified in the app config.
#[cfg(feature = "config")]
pub fn init_registry_for_app(exe_path: &std::path::Path, version: crate::config::WindowsVersion) {
    use crate::config;

    // If already initialised (e.g. called twice), do nothing.
    if REGISTRY_STORE.get().is_some() {
        return;
    }

    let path = config::registry_path(exe_path, version);

    let store = if path.exists() {
        match std::fs::read_to_string(&path)
            .map_err(|e| e.to_string())
            .and_then(|s| {
                serde_json::from_str::<RegistryStoreSnapshot>(&s).map_err(|e| e.to_string())
            }) {
            Ok(snap) => snapshot_to_store(snap),
            Err(e) => {
                eprintln!(
                    "rine: failed to parse registry file {}: {}, regenerating defaults",
                    path.display(),
                    e
                );
                build_default_store_and_save(version, &path)
            }
        }
    } else {
        build_default_store_and_save(version, &path)
    };

    let _ = REGISTRY_STORE.set(store);
}

#[cfg(feature = "config")]
fn build_default_store_and_save(
    version: crate::config::WindowsVersion,
    path: &std::path::Path,
) -> RegistryStore {
    use crate::config::WindowsVersion;

    let ver_data = match version {
        WindowsVersion::WinXP => VersionDefaults::WIN_XP,
        WindowsVersion::Win7 => VersionDefaults::WIN7,
        WindowsVersion::Win10 => VersionDefaults::WIN10,
        WindowsVersion::Win11 => VersionDefaults::WIN11,
    };
    let store = RegistryStore::new_for_version_data(ver_data);

    // Save to disk so the user can inspect and customise the defaults.
    let snap = store_to_snapshot(&store);
    if let Some(parent) = path.parent()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        eprintln!(
            "rine: failed to create registry dir {}: {}",
            parent.display(),
            e
        );
        return store;
    }
    match serde_json::to_string_pretty(&snap) {
        Ok(json) => {
            if let Err(e) = std::fs::write(path, &json) {
                eprintln!(
                    "rine: failed to write registry file {}: {}",
                    path.display(),
                    e
                );
            }
        }
        Err(e) => {
            eprintln!("rine: failed to serialise registry defaults: {}", e);
        }
    }
    store
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
// UI Export — Registry data for display/editing in frontend
// ---------------------------------------------------------------------------

/// A registry value formatted for UI display.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RegistryKeyUI {
    /// Key path (e.g., "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion")
    pub path: String,
    /// Values in this key
    pub values: Vec<RegistryValueUI>,
    /// Subkey names (not fully expanded; expanded on demand by frontend)
    pub subkey_names: Vec<String>,
}

/// Full registry export for UI display.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RegistryExportUI {
    /// Root key snapshots keyed by root name ("HKEY_LOCAL_MACHINE", etc.)
    pub roots: std::collections::BTreeMap<String, RegistryKeyUI>,
    /// Set of registry paths that are locked to the Windows version.
    /// Paths use backslash separators, e.g., "HKEY_LOCAL_MACHINE\\...\\Value"
    pub locked_paths: Vec<String>,
}

/// Get the registry export for UI display.
#[cfg(feature = "config")]
pub fn get_registry_export_for_ui() -> RegistryExportUI {
    let store = registry_store();
    let locked_paths = get_locked_registry_paths();

    let mut roots = std::collections::BTreeMap::new();

    // Map of root HKEY to name
    let root_names = vec![
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

    // Prefix locked paths with their root names
    let prefixed_locked = locked_paths
        .iter()
        .map(|p| {
            if p.contains('\\') {
                p.clone()
            } else {
                // Bare path, determine root
                format!("HKEY_LOCAL_MACHINE\\{}", p)
            }
        })
        .collect();

    RegistryExportUI {
        roots,
        locked_paths: prefixed_locked,
    }
}
/// Convert a RegistryKey to UI representation (non-recursive; only immediate contents).
#[cfg(feature = "config")]
fn registry_key_to_ui(key: &RegistryKey, path: &str) -> RegistryKeyUI {
    let values = key
        .values
        .iter()
        .map(|(name, value)| {
            let is_locked = is_locked_registry_value(path, name);
            RegistryValueUI {
                name: name.clone(),
                type_name: value_type_name(value),
                data: value_to_display_string(value),
                locked: is_locked,
            }
        })
        .collect();

    let subkey_names = key.subkeys.keys().cloned().collect();

    RegistryKeyUI {
        path: path.to_string(),
        values,
        subkey_names,
    }
}

/// Get display string for a registry value.
#[cfg(feature = "config")]
fn value_to_display_string(value: &RegistryValue) -> String {
    match value {
        RegistryValue::String(s) | RegistryValue::ExpandString(s) => s.clone(),
        RegistryValue::Dword(d) => d.to_string(),
        RegistryValue::Qword(q) => q.to_string(),
        RegistryValue::Binary(b) => b
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<Vec<_>>()
            .join(" ")
            .to_string(),
        RegistryValue::MultiString(ss) => ss.join("; "),
    }
}

/// Get type name for display.
#[cfg(feature = "config")]
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

/// Check if a specific registry value is locked (read-only due to Windows version).
#[cfg(feature = "config")]
pub fn is_locked_registry_value(key_path: &str, value_name: &str) -> bool {
    let locked_entries = vec![
        // Build/version info locked to Windows version
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

    for (locked_key, locked_value) in locked_entries {
        let key_matches = key_path.eq_ignore_ascii_case(locked_key);
        let value_matches = value_name.eq_ignore_ascii_case(locked_value);
        if key_matches && value_matches {
            return true;
        }
    }
    false
}

/// Get the set of registry paths that are locked to the Windows version.
#[cfg(feature = "config")]
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
