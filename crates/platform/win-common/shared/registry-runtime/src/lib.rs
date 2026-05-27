//! Shared process-wide Windows registry runtime.
//!
//! This crate owns the mutable registry store and persistence/import logic so
//! `rine-types` can remain focused on registry data structures and constants.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use rine_types::config::{self, WindowsVersion};
use rine_types::registry::{
    HKEY_CLASSES_ROOT, HKEY_CURRENT_CONFIG, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, HKEY_USERS,
    RegistryKey, RegistryValue, win_ini_section_to_reg_path,
};

/// The process-wide in-memory registry, keyed by predefined root HKEY.
pub struct RegistryStore {
    inner: Mutex<HashMap<isize, RegistryKey>>,
}

impl RegistryStore {
    fn new() -> Self {
        let mut roots = HashMap::new();

        let mut hklm = RegistryKey::new();
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

struct VersionDefaults {
    product_name: &'static str,
    build: &'static str,
    current_version: &'static str,
    major: u32,
    minor: u32,
}

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

fn populate_hklm(root: &mut RegistryKey, ver: VersionDefaults) {
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

    let cp = root.create_subkey("SYSTEM\\CurrentControlSet\\Control\\Nls\\CodePage");
    cp.set_value("ACP".into(), RegistryValue::String("1252".into()));
    cp.set_value("OEMCP".into(), RegistryValue::String("437".into()));

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
    let env = root.create_subkey("Environment");
    env.set_value(
        "TEMP".into(),
        RegistryValue::ExpandString("%USERPROFILE%\\AppData\\Local\\Temp".into()),
    );
    env.set_value(
        "TMP".into(),
        RegistryValue::ExpandString("%USERPROFILE%\\AppData\\Local\\Temp".into()),
    );

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

fn snapshot_to_store(snap: RegistryStoreSnapshot) -> RegistryStore {
    let mut roots = HashMap::new();
    roots.insert(HKEY_LOCAL_MACHINE, snap.hklm);
    roots.insert(HKEY_CURRENT_USER, snap.hkcu);
    roots.insert(HKEY_CLASSES_ROOT, snap.hkcr);
    roots.insert(HKEY_USERS, snap.hku);
    roots.insert(HKEY_CURRENT_CONFIG, snap.hkcc);
    RegistryStore::from_roots(roots)
}

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

pub fn init_registry_for_app(exe_path: &std::path::Path, version: WindowsVersion) {
    if REGISTRY_STORE.get().is_some() {
        return;
    }

    let store = load_registry_store_for_app(exe_path, version);
    let _ = REGISTRY_STORE.set(store);
}

pub fn reinit_registry_for_app(exe_path: &std::path::Path, version: WindowsVersion) {
    let store = load_registry_store_for_app(exe_path, version);

    if let Some(existing) = REGISTRY_STORE.get() {
        let mut existing_inner = existing.inner.lock().unwrap();
        let new_roots = store.inner.into_inner().unwrap_or_else(|e| e.into_inner());
        *existing_inner = new_roots;
        return;
    }

    let _ = REGISTRY_STORE.set(store);
}

fn load_registry_store_for_app(
    exe_path: &std::path::Path,
    version: WindowsVersion,
) -> RegistryStore {
    let path = config::registry_path(exe_path, version);

    if path.exists() {
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
    }
}

fn build_default_store_and_save(version: WindowsVersion, path: &std::path::Path) -> RegistryStore {
    let ver_data = match version {
        WindowsVersion::WinXP => VersionDefaults::WIN_XP,
        WindowsVersion::Win7 => VersionDefaults::WIN7,
        WindowsVersion::Win10 => VersionDefaults::WIN10,
        WindowsVersion::Win11 => VersionDefaults::WIN11,
    };
    let store = RegistryStore::new_for_version_data(ver_data);

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

pub fn save_registry_for_app(
    exe_path: &std::path::Path,
    version: WindowsVersion,
) -> Result<std::path::PathBuf, String> {
    let path = config::registry_path(exe_path, version);

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create {}: {e}", parent.display()))?;
    }

    let snap = store_to_snapshot(registry_store());
    let json = serde_json::to_string_pretty(&snap)
        .map_err(|e| format!("failed to serialize registry snapshot: {e}"))?;
    std::fs::write(&path, json).map_err(|e| format!("failed to write {}: {e}", path.display()))?;
    Ok(path)
}

pub fn import_win_ini(path: &std::path::Path) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let store = registry_store();
    let mut current_section = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with('[') {
            if let Some(end) = trimmed.find(']') {
                current_section = trimmed[1..end].trim().to_string();
            }
            continue;
        }
        if current_section.is_empty() {
            continue;
        }
        if let Some(eq) = trimmed.find('=') {
            let key = trimmed[..eq].trim();
            let value = trimmed[eq + 1..].trim();
            let reg_path = win_ini_section_to_reg_path(&current_section);
            store.with_root_mut(HKEY_CURRENT_USER, |root| {
                root.create_subkey(&reg_path)
                    .set_value(key.to_string(), RegistryValue::String(value.to_string()));
            });
        }
    }
}

pub fn try_import_win_ini_for_app(exe_path: &std::path::Path) {
    let root = config::rine_root();
    let app_ini = root
        .join("apps")
        .join(config::app_hash(exe_path))
        .join("win.ini");
    if app_ini.exists() {
        import_win_ini(&app_ini);
        return;
    }
    let global_ini = root.join("win.ini");
    if global_ini.exists() {
        import_win_ini(&global_ini);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
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
    #[serial]
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
