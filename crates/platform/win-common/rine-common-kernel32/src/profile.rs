//! WIN.INI / private INI profile string APIs.
//!
//! `GetProfileString` and `WriteProfileString` route through the in-process
//! registry store using the standard Windows IniFileMapping rules.
//!
//! `GetPrivateProfileString` and `WritePrivateProfileString` redirect requests
//! for `win.ini` through the same registry path; all other file names are
//! serviced by real file I/O after translating the Windows path to a Linux one.

use rine_types::{
    errors::BOOL,
    registry::{HKEY_CURRENT_USER, RegistryValue, registry_store, win_ini_section_to_reg_path},
    strings::{LPSTR, LPWSTR},
};
use tracing::debug;

// ---------------------------------------------------------------------------
// Output buffer helpers (truncating — NOT the same as write_cstr/write_wstr)
// ---------------------------------------------------------------------------

/// Write `value` into a caller-supplied ANSI buffer, truncating to fit.
///
/// Returns the number of characters written excluding the null terminator, or
/// 0 if `buf` is null or `buf_size` is 0.
unsafe fn write_ansi_buf(buf: LPSTR, buf_size: u32, value: &str) -> u32 {
    if buf.is_null() || buf_size == 0 {
        return 0;
    }
    let ptr = buf.as_mut_ptr();
    let capacity = (buf_size - 1) as usize;
    let bytes = value.as_bytes();
    let write_len = bytes.len().min(capacity);
    unsafe {
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, write_len);
        *ptr.add(write_len) = 0;
    }
    write_len as u32
}

/// Write `value` into a caller-supplied wide (UTF-16LE) buffer, truncating to
/// fit.
///
/// Returns the number of WCHARs written excluding the null terminator, or 0 if
/// `buf` is null or `buf_size` is 0.
unsafe fn write_wide_buf(buf: LPWSTR, buf_size: u32, value: &str) -> u32 {
    if buf.is_null() || buf_size == 0 {
        return 0;
    }
    let ptr = buf.as_mut_ptr();
    let encoded: Vec<u16> = value.encode_utf16().collect();
    let capacity = (buf_size - 1) as usize;
    let write_len = encoded.len().min(capacity);
    unsafe {
        core::ptr::copy_nonoverlapping(encoded.as_ptr(), ptr, write_len);
        *ptr.add(write_len) = 0;
    }
    write_len as u32
}

// ---------------------------------------------------------------------------
// Registry helpers
// ---------------------------------------------------------------------------

fn registry_query_string(reg_path: &str, key: &str) -> Option<String> {
    registry_store()
        .with_root(HKEY_CURRENT_USER, |root| {
            root.open_subkey(reg_path)
                .and_then(|k| k.get_value(key))
                .and_then(|v| match v {
                    RegistryValue::String(s) | RegistryValue::ExpandString(s) => Some(s.clone()),
                    _ => None,
                })
        })
        .flatten()
}

fn registry_set_string(reg_path: &str, key: &str, value: &str) {
    registry_store().with_root_mut(HKEY_CURRENT_USER, |root| {
        root.create_subkey(reg_path)
            .set_value(key.to_string(), RegistryValue::String(value.to_string()));
    });
}

fn registry_delete_value(reg_path: &str, key: &str) {
    let lower = key.to_ascii_lowercase();
    registry_store().with_root_mut(HKEY_CURRENT_USER, |root| {
        let subkey = root.create_subkey(reg_path);
        subkey.values.retain(|k, _| k.to_ascii_lowercase() != lower);
    });
}

// ---------------------------------------------------------------------------
// GetProfileString / WriteProfileString
// ---------------------------------------------------------------------------

/// Reads a string value from the win.ini registry mapping (ANSI output).
///
/// # Arguments
/// * `section`: The INI section name to look up.
/// * `key`: The INI key name to look up.
/// * `default`: The default value to return if the section/key is not found.
/// * `buf`: A pointer to a caller-allocated buffer that receives the string value.
/// * `buf_size`: The size of the buffer in characters (including space for the null terminator).
///
/// # Safety
/// The caller must ensure that `buf` points to a valid writable buffer of at least `buf_size` bytes,
/// and that `section`, `key`, and `default` are valid UTF-8 strings.
/// The function does not perform any synchronization, so concurrent calls with the same section
/// and key may result in undefined behavior.
///
/// # Returns
/// The number of characters copied to the buffer, not including the null terminator.
/// If the buffer is too small, the return value is `buf_size - 1` and the string is truncated.
/// If the section/key is not found, the default value is copied to the buffer and its length is returned.
///
/// # Notes
/// This function does not yet implement setting `GetLastError` on failure.
pub unsafe fn get_profile_string_a(
    section: &str,
    key: &str,
    default: &str,
    buf: LPSTR,
    buf_size: u32,
) -> u32 {
    let reg_path = win_ini_section_to_reg_path(section);
    debug!(section, key, reg_path, "GetProfileString");
    let value = registry_query_string(&reg_path, key);
    let s = value.as_deref().unwrap_or(default);
    unsafe { write_ansi_buf(buf, buf_size, s) }
}

/// Reads a string value from the win.ini registry mapping (wide output).
///
/// # Arguments
/// * `section`: The INI section name to look up.
/// * `key`: The INI key name to look up.
/// * `default`: The default value to return if the section/key is not found.
/// * `buf`: A pointer to a caller-allocated buffer that receives the string value.
/// * `buf_size`: The size of the buffer in characters (including space for the null terminator).
///
/// # Safety
/// The caller must ensure that `buf` points to a valid writable buffer of at least `buf_size` WCHARs,
/// and that `section`, `key`, and `default` are valid UTF-8 strings that can be converted to UTF-16LE.
/// The function does not perform any synchronization, so concurrent calls with the same section and
/// key may result in undefined behavior.
///
/// # Returns
/// The number of characters copied to the buffer, not including the null terminator.
/// If the buffer is too small, the return value is `buf_size - 1` and the string is truncated.
/// If the section/key is not found, the default value is copied to the buffer and its length is returned.
///
/// # Notes
/// This function does not yet implement setting `GetLastError` on failure.
pub unsafe fn get_profile_string_w(
    section: &str,
    key: &str,
    default: &str,
    buf: LPWSTR,
    buf_size: u32,
) -> u32 {
    let reg_path = win_ini_section_to_reg_path(section);
    debug!(section, key, reg_path, "GetProfileStringW");
    let value = registry_query_string(&reg_path, key);
    let s = value.as_deref().unwrap_or(default);
    unsafe { write_wide_buf(buf, buf_size, s) }
}

/// Writes a string value to the win.ini registry mapping.
///
/// # Arguments
/// * `section`: The INI section name to write to.
/// * `key`: The INI key name to write.
/// * `value`: The value to write, or `None` to delete the key.
///
/// # Returns
/// `BOOL::TRUE` if the operation succeeded, or `BOOL::FALSE` if it failed.
///
/// # Notes
/// This function does not yet implement setting `GetLastError` on failure.
pub fn write_profile_string(section: &str, key: &str, value: Option<&str>) -> BOOL {
    let reg_path = win_ini_section_to_reg_path(section);
    debug!(section, key, reg_path, "WriteProfileString");
    match value {
        Some(v) => registry_set_string(&reg_path, key, v),
        None => registry_delete_value(&reg_path, key),
    }
    BOOL::TRUE
}

// ---------------------------------------------------------------------------
// Path translation for private profile I/O
// ---------------------------------------------------------------------------

/// Returns `true` when `file_name` refers to the global `win.ini`.
fn is_win_ini(file_name: &str) -> bool {
    let lower = file_name.to_ascii_lowercase();
    let normalized = lower.replace('\\', "/");
    normalized.ends_with("/win.ini") || normalized == "win.ini"
}

/// Translate a Windows file path to the corresponding Linux path.
///
/// * Absolute Linux paths pass through unchanged.
/// * Drive-letter paths (`X:\...`) are mapped to `~/.rine/drives/x/...`.
/// * Extended-length (`\\?\`) and device (`\\.\`) prefixes are stripped first.
fn translate_ini_path(win_path: &str) -> std::path::PathBuf {
    if win_path.starts_with('/') {
        return std::path::PathBuf::from(win_path);
    }
    let normalized = win_path.replace('\\', "/");
    let stripped = normalized
        .strip_prefix("//?/")
        .or_else(|| normalized.strip_prefix("//./"))
        .unwrap_or(&normalized);
    let bytes = stripped.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        let drive = (bytes[0] as char).to_ascii_lowercase();
        let rest = &stripped[2..];
        let rest = rest.strip_prefix('/').unwrap_or(rest);
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        let mut path = std::path::PathBuf::from(home);
        path.push(".rine/drives");
        path.push(drive.to_string());
        if !rest.is_empty() {
            path.push(rest);
        }
        return path;
    }
    std::path::PathBuf::from(stripped)
}

// ---------------------------------------------------------------------------
// INI file I/O helpers
// ---------------------------------------------------------------------------

/// Look up `key` inside `section` within in-memory INI `content`.
fn parse_ini_value(content: &str, section: &str, key: &str) -> Option<String> {
    let mut in_section = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with('[') {
            if let Some(end) = trimmed.find(']') {
                let section_name = trimmed[1..end].trim();
                in_section = section_name.eq_ignore_ascii_case(section);
            }
            continue;
        }
        if !in_section {
            continue;
        }
        if let Some(eq) = trimmed.find('=')
            && trimmed[..eq].trim().eq_ignore_ascii_case(key)
        {
            return Some(trimmed[eq + 1..].trim().to_string());
        }
    }
    None
}

/// Return a modified copy of `content` with `key` set to `value` (or deleted
/// if `value` is `None`) inside `section`.
fn modify_ini_content(content: &str, section: &str, key: &str, value: Option<&str>) -> String {
    let section_lower = section.to_ascii_lowercase();
    let key_lower = key.to_ascii_lowercase();

    let mut result: Vec<String> = Vec::new();
    let mut in_section = false;
    let mut section_found = false;
    let mut key_handled = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with('[') {
            // Flush pending key write before entering the next section.
            if in_section && !key_handled {
                if let Some(v) = value {
                    result.push(format!("{}={}", key, v));
                }
                key_handled = true;
            }
            if let Some(end) = trimmed.find(']') {
                let sname = trimmed[1..end].trim();
                in_section = sname.to_ascii_lowercase() == section_lower;
                if in_section {
                    section_found = true;
                }
            } else {
                in_section = false;
            }
            result.push(line.to_string());
            continue;
        }

        if in_section
            && !key_handled
            && let Some(eq) = trimmed.find('=')
            && trimmed[..eq].trim().to_ascii_lowercase() == key_lower
        {
            // Replace existing key, or drop it on deletion.
            if let Some(v) = value {
                result.push(format!("{}={}", key, v));
            }
            key_handled = true;
            continue;
        }

        result.push(line.to_string());
    }

    // End of file while still in the section.
    if in_section
        && !key_handled
        && let Some(v) = value
    {
        result.push(format!("{}={}", key, v));
    }

    // Section was never found — append it.
    if !section_found && let Some(v) = value {
        if !result.is_empty() {
            result.push(String::new());
        }
        result.push(format!("[{}]", section));
        result.push(format!("{}={}", key, v));
    }

    let mut output = result.join("\n");
    if !output.is_empty() {
        output.push('\n');
    }
    output
}

// ---------------------------------------------------------------------------
// GetPrivateProfileString / WritePrivateProfileString
// ---------------------------------------------------------------------------

/// Reads a string from a private profile INI file (ANSI output).
///
/// # Arguments
/// * `section`: The INI section name to look up.
/// * `key`: The INI key name to look up.
/// * `default`: The default value to return if the section/key is not found.
/// * `buf`: A pointer to a caller-allocated buffer that receives the string value.
/// * `buf_size`: The size of the buffer in characters (including space for the null terminator).
/// * `file_name`: The name of the INI file to read from.  
///   If this is `win.ini` the request is routed through the registry instead.
///
/// # Safety
/// The caller must ensure that `buf` points to a valid writable buffer of at least `buf_size` bytes,
/// and that `section`, `key`, `default`, and `file_name` are valid UTF-8 strings.
/// The function does not perform any synchronization, so concurrent calls with the same section and key may result in undefined behavior.
///
/// # Returns
/// The number of characters copied to the buffer, not including the null terminator.
/// If the buffer is too small, the return value is `buf_size - 1` and the string is truncated.
/// If the section/key is not found, the default value is copied to the buffer and its length is returned.
///
/// # Notes
/// This function does not yet implement setting `GetLastError` on failure.
pub unsafe fn get_private_profile_string_a(
    section: &str,
    key: &str,
    default: &str,
    buf: LPSTR,
    buf_size: u32,
    file_name: &str,
) -> u32 {
    if is_win_ini(file_name) {
        return unsafe { get_profile_string_a(section, key, default, buf, buf_size) };
    }
    debug!(section, key, file_name, "GetPrivateProfileString");
    let path = translate_ini_path(file_name);
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let value = parse_ini_value(&content, section, key);
    let s = value.as_deref().unwrap_or(default);
    unsafe { write_ansi_buf(buf, buf_size, s) }
}

/// Writes a string to a private profile INI file.
///
/// # Arguments
/// * `section`: The INI section name to write to.
/// * `key`: The INI key name to write.
/// * `value`: The value to write, or `None` to delete the key.
/// * `file_name`: The name of the INI file to write to.  
///   If this is `win.ini` the request is routed through the registry instead.
///
/// # Returns
/// `BOOL::TRUE` if the operation succeeded, or `BOOL::FALSE` if it failed.
///
/// # Notes
/// This function does not yet implement setting `GetLastError` on failure.
pub fn write_private_profile_string_a(
    section: &str,
    key: &str,
    value: Option<&str>,
    file_name: &str,
) -> BOOL {
    if is_win_ini(file_name) {
        return write_profile_string(section, key, value);
    }
    debug!(section, key, file_name, "WritePrivateProfileString");
    let path = translate_ini_path(file_name);
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let new_content = modify_ini_content(&content, section, key, value);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match std::fs::write(&path, new_content) {
        Ok(_) => BOOL::TRUE,
        Err(_) => BOOL::FALSE,
    }
}

/// Reads a string from a private profile INI file (wide output).
///
/// # Arguments
/// * `section`: The INI section name to look up.
/// * `key`: The INI key name to look up.
/// * `default`: The default value to return if the section/key is not found.
/// * `buf`: A pointer to a caller-allocated buffer that receives the string value.
/// * `buf_size`: The size of the buffer in characters (including space for the null terminator).
/// * `file_name`: The name of the INI file to read from.  
///   If this is `win.ini` the request is routed through the registry instead.
///
/// # Safety
/// The caller must ensure that `buf` points to a valid writable buffer of at least `buf_size` bytes,
/// and that `section`, `key`, `default`, and `file_name` are valid UTF-8 strings.
/// The function does not perform any synchronization, so concurrent calls with the same section and key may result in undefined behavior.
///
/// # Returns
/// The number of characters copied to the buffer, not including the null terminator.
/// If the buffer is too small, the return value is `buf_size - 1` and the string is truncated.
/// If the section/key is not found, the default value is copied to the buffer and its length is returned.
///
/// # Notes
/// This function does not yet implement setting `GetLastError` on failure.
pub unsafe fn get_private_profile_string_w(
    section: &str,
    key: &str,
    default: &str,
    buf: LPWSTR,
    buf_size: u32,
    file_name: &str,
) -> u32 {
    if is_win_ini(file_name) {
        return unsafe { get_profile_string_w(section, key, default, buf, buf_size) };
    }
    debug!(section, key, file_name, "GetPrivateProfileStringW");
    let path = translate_ini_path(file_name);
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let value = parse_ini_value(&content, section, key);
    let s = value.as_deref().unwrap_or(default);
    unsafe { write_wide_buf(buf, buf_size, s) }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_win_ini_detects_bare_name() {
        assert!(is_win_ini("win.ini"));
        assert!(is_win_ini("WIN.INI"));
        assert!(is_win_ini("Win.Ini"));
    }

    #[test]
    fn is_win_ini_detects_path() {
        assert!(is_win_ini("C:\\Windows\\win.ini"));
        assert!(is_win_ini("C:/windows/WIN.INI"));
    }

    #[test]
    fn is_win_ini_rejects_non_win_ini() {
        assert!(!is_win_ini("myapp.ini"));
        assert!(!is_win_ini("C:\\myapp\\myapp.ini"));
    }

    #[test]
    fn parse_ini_value_finds_key() {
        let content = "[section]\nfoo=bar\nbaz=qux\n";
        assert_eq!(
            parse_ini_value(content, "section", "foo"),
            Some("bar".into())
        );
        assert_eq!(
            parse_ini_value(content, "section", "baz"),
            Some("qux".into())
        );
    }

    #[test]
    fn parse_ini_value_case_insensitive() {
        let content = "[Section]\nFoo=Bar\n";
        assert_eq!(
            parse_ini_value(content, "SECTION", "FOO"),
            Some("Bar".into())
        );
    }

    #[test]
    fn parse_ini_value_missing_key() {
        let content = "[section]\nfoo=bar\n";
        assert_eq!(parse_ini_value(content, "section", "missing"), None);
    }

    #[test]
    fn parse_ini_value_missing_section() {
        let content = "[other]\nfoo=bar\n";
        assert_eq!(parse_ini_value(content, "section", "foo"), None);
    }

    #[test]
    fn modify_ini_sets_existing_key() {
        let content = "[section]\nfoo=old\n";
        let result = modify_ini_content(content, "section", "foo", Some("new"));
        assert!(result.contains("foo=new"));
        assert!(!result.contains("foo=old"));
    }

    #[test]
    fn modify_ini_adds_key_to_existing_section() {
        let content = "[section]\nexisting=val\n";
        let result = modify_ini_content(content, "section", "newkey", Some("newval"));
        assert!(result.contains("newkey=newval"));
        assert!(result.contains("existing=val"));
    }

    #[test]
    fn modify_ini_creates_new_section() {
        let content = "[other]\nfoo=bar\n";
        let result = modify_ini_content(content, "newsec", "key", Some("val"));
        assert!(result.contains("[newsec]"));
        assert!(result.contains("key=val"));
        assert!(result.contains("[other]"));
    }

    #[test]
    fn modify_ini_deletes_key() {
        let content = "[section]\nfoo=bar\nbaz=qux\n";
        let result = modify_ini_content(content, "section", "foo", None);
        assert!(!result.contains("foo=bar"));
        assert!(result.contains("baz=qux"));
    }

    #[test]
    fn translate_ini_path_linux() {
        let p = translate_ini_path("/etc/foo.ini");
        assert_eq!(p, std::path::PathBuf::from("/etc/foo.ini"));
    }

    #[test]
    fn translate_ini_path_drive_letter() {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        let p = translate_ini_path("C:\\foo\\bar.ini");
        assert_eq!(
            p,
            std::path::PathBuf::from(format!("{}/.rine/drives/c/foo/bar.ini", home))
        );
    }
}
