//! Process-wide Windows environment variable store.
//!
//! Windows PE executables reference variables like `%USERPROFILE%`,
//! `%TEMP%`, `%SYSTEMROOT%`, etc. This module maintains a global store
//! seeded with sensible defaults that map well-known Windows variables
//! to rine's Linux drive layout (`~/.rine/drives/c/…`).
//!
//! The store is case-insensitive on variable names, matching Windows
//! behaviour. It is safe to call from any thread (protected by a mutex).

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

// ---------------------------------------------------------------------------
// Global environment store
// ---------------------------------------------------------------------------

struct EnvStore {
    vars: HashMap<String, String>,
}

impl EnvStore {
    fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        let user = std::env::var("USER").unwrap_or_else(|_| "user".into());
        let rine_root = format!("{home}/.rine/drives/c");

        let mut vars = HashMap::new();

        // Well-known Windows variables mapped to rine paths.
        vars.insert("SYSTEMROOT".into(), format!("{rine_root}/Windows"));
        vars.insert("WINDIR".into(), format!("{rine_root}/Windows"));
        vars.insert("SYSTEMDRIVE".into(), "C:".into());
        vars.insert("PROGRAMFILES".into(), format!("{rine_root}/Program Files"));
        vars.insert(
            "PROGRAMFILES(X86)".into(),
            format!("{rine_root}/Program Files (x86)"),
        );
        vars.insert(
            "COMMONPROGRAMFILES".into(),
            format!("{rine_root}/Program Files/Common Files"),
        );
        vars.insert("PROGRAMDATA".into(), format!("{rine_root}/ProgramData"));
        vars.insert("USERPROFILE".into(), format!("{rine_root}/Users/{user}"));
        vars.insert("HOMEDRIVE".into(), "C:".into());
        vars.insert("HOMEPATH".into(), format!("\\Users\\{user}"));
        vars.insert(
            "APPDATA".into(),
            format!("{rine_root}/Users/{user}/AppData/Roaming"),
        );
        vars.insert(
            "LOCALAPPDATA".into(),
            format!("{rine_root}/Users/{user}/AppData/Local"),
        );
        vars.insert(
            "TEMP".into(),
            format!("{rine_root}/Users/{user}/AppData/Local/Temp"),
        );
        vars.insert(
            "TMP".into(),
            format!("{rine_root}/Users/{user}/AppData/Local/Temp"),
        );
        vars.insert("USERNAME".into(), user);
        vars.insert("COMPUTERNAME".into(), "RINE".into());
        vars.insert(
            "COMSPEC".into(),
            format!("{rine_root}/Windows/System32/cmd.exe"),
        );
        vars.insert("OS".into(), "Windows_NT".into());
        vars.insert("PATHEXT".into(), ".COM;.EXE;.BAT;.CMD".into());
        vars.insert("NUMBER_OF_PROCESSORS".into(), num_cpus().to_string());
        vars.insert("PROCESSOR_ARCHITECTURE".into(), "AMD64".into());

        // Seed the Windows PATH with System32 and friends.
        vars.insert(
            "PATH".into(),
            format!(
                "{rine_root}/Windows/System32;\
                 {rine_root}/Windows;\
                 {rine_root}/Windows/System32/Wbem"
            ),
        );

        Self { vars }
    }
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

static ENV: LazyLock<Mutex<EnvStore>> = LazyLock::new(|| Mutex::new(EnvStore::new()));

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Retrieve a Windows environment variable (case-insensitive).
///
/// Returns `None` if the variable is not set.
pub fn get_var(name: &str) -> Option<String> {
    let store = ENV.lock().unwrap();
    store.vars.get(&name.to_ascii_uppercase()).cloned()
}

/// Set a Windows environment variable (case-insensitive key).
///
/// If `value` is `None` the variable is removed (matching
/// `SetEnvironmentVariable` semantics when `lpValue` is NULL).
pub fn set_var(name: &str, value: Option<&str>) {
    let mut store = ENV.lock().unwrap();
    let key = name.to_ascii_uppercase();
    match value {
        Some(v) => {
            store.vars.insert(key, v.to_string());
        }
        None => {
            store.vars.remove(&key);
        }
    }
}

/// Expand `%VAR%` references in `src`.
///
/// Unrecognised variables are left as-is (including the `%` delimiters),
/// matching Windows `ExpandEnvironmentStrings` behaviour.
pub fn expand_vars(src: &str) -> String {
    let store = ENV.lock().unwrap();
    let mut result = String::with_capacity(src.len());
    let mut chars = src.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            let mut var_name = String::new();
            let mut found_close = false;
            for inner in chars.by_ref() {
                if inner == '%' {
                    found_close = true;
                    break;
                }
                var_name.push(inner);
            }
            if found_close && !var_name.is_empty() {
                if let Some(val) = store.vars.get(&var_name.to_ascii_uppercase()) {
                    result.push_str(val);
                } else {
                    // Unknown variable — preserve verbatim.
                    result.push('%');
                    result.push_str(&var_name);
                    result.push('%');
                }
            } else {
                // Lone '%' or '%%' — emit literally.
                result.push('%');
                result.push_str(&var_name);
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Build a Windows-style environment block (null-separated, double-null
/// terminated) as ANSI bytes.
pub fn build_ansi_block() -> Vec<u8> {
    let store = ENV.lock().unwrap();
    let mut block = Vec::new();
    let mut sorted: Vec<_> = store.vars.iter().collect();
    sorted.sort_by_key(|(k, _)| k.to_ascii_uppercase());
    for (k, v) in sorted {
        block.extend_from_slice(k.as_bytes());
        block.push(b'=');
        block.extend_from_slice(v.as_bytes());
        block.push(0);
    }
    block.push(0); // double-null terminator
    block
}

/// Build a Windows-style environment block encoded as UTF-16LE.
pub fn build_wide_block() -> Vec<u16> {
    let store = ENV.lock().unwrap();
    let mut block = Vec::new();
    let mut sorted: Vec<_> = store.vars.iter().collect();
    sorted.sort_by_key(|(k, _)| k.to_ascii_uppercase());
    for (k, v) in sorted {
        let entry = format!("{k}={v}");
        block.extend(entry.encode_utf16());
        block.push(0);
    }
    block.push(0); // double-null terminator
    block
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_store() -> EnvStore {
        EnvStore::new()
    }

    #[test]
    fn default_vars_populated() {
        let s = fresh_store();
        assert!(s.vars.contains_key("SYSTEMROOT"));
        assert!(s.vars.contains_key("USERPROFILE"));
        assert!(s.vars.contains_key("TEMP"));
        assert!(s.vars.contains_key("OS"));
        assert_eq!(s.vars.get("OS").unwrap(), "Windows_NT");
    }

    #[test]
    fn get_set_delete() {
        let name = "RINE_TEST_ENV_18374";
        assert!(get_var(name).is_none());
        set_var(name, Some("hello"));
        assert_eq!(get_var(name).unwrap(), "hello");
        // Case-insensitive read.
        assert_eq!(get_var("rine_test_env_18374").unwrap(), "hello");
        // Delete.
        set_var(name, None);
        assert!(get_var(name).is_none());
    }

    #[test]
    fn expand_known_var() {
        set_var("RINE_EXP_A", Some("replaced"));
        let result = expand_vars("prefix_%RINE_EXP_A%_suffix");
        assert_eq!(result, "prefix_replaced_suffix");
        set_var("RINE_EXP_A", None);
    }

    #[test]
    fn expand_unknown_preserved() {
        let result = expand_vars("%NONEXISTENT_XYZ_42%");
        assert_eq!(result, "%NONEXISTENT_XYZ_42%");
    }

    #[test]
    fn expand_empty_percent_pair() {
        let result = expand_vars("100%%");
        assert_eq!(result, "100%");
    }

    #[test]
    fn expand_lone_percent() {
        let result = expand_vars("50% off");
        assert_eq!(result, "50% off");
    }

    #[test]
    fn expand_multiple_vars() {
        set_var("RINE_M1", Some("X"));
        set_var("RINE_M2", Some("Y"));
        let result = expand_vars("%RINE_M1%-%RINE_M2%");
        assert_eq!(result, "X-Y");
        set_var("RINE_M1", None);
        set_var("RINE_M2", None);
    }

    #[test]
    fn ansi_block_double_null() {
        let block = build_ansi_block();
        assert!(block.len() >= 2);
        assert_eq!(block[block.len() - 1], 0);
        assert_eq!(block[block.len() - 2], 0);
    }

    #[test]
    fn wide_block_double_null() {
        let block = build_wide_block();
        assert!(block.len() >= 2);
        assert_eq!(block[block.len() - 1], 0);
        assert_eq!(block[block.len() - 2], 0);
    }
}
