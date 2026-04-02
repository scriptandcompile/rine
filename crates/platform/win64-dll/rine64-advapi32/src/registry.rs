//! advapi32 registry functions: RegOpenKeyExA/W, RegQueryValueExA/W,
//! RegSetValueExA/W, RegCreateKeyExA/W, RegCloseKey.

use rine_types::errors::{
    ERROR_FILE_NOT_FOUND, ERROR_INVALID_HANDLE, ERROR_INVALID_PARAMETER, ERROR_SUCCESS,
};
use rine_types::handles::{Handle, HandleEntry, handle_table};
use rine_types::registry::{
    self, RegistryKeyState, RegistryValue, is_predefined_key, registry_store,
};
use rine_types::strings::{read_cstr, read_wstr};

// ---------------------------------------------------------------------------
// Win32 error codes specific to registry
// ---------------------------------------------------------------------------

const ERROR_MORE_DATA: u32 = 234;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Resolve a handle to (root_hkey, subkey_path).
/// Works for both predefined root keys and opened sub-key handles.
fn resolve_key(hkey: isize) -> Option<(isize, String)> {
    if is_predefined_key(hkey) {
        return Some((hkey, String::new()));
    }
    let handle = Handle::from_raw(hkey);
    handle_table().with_registry_key(handle, |state| (state.root, state.path.clone()))
}

/// Combine a base path with a sub-key name.
fn join_path(base: &str, sub: &str) -> String {
    if base.is_empty() {
        sub.to_string()
    } else if sub.is_empty() {
        base.to_string()
    } else {
        format!("{base}\\{sub}")
    }
}

// ---------------------------------------------------------------------------
// RegOpenKeyExA / RegOpenKeyExW
// ---------------------------------------------------------------------------

/// RegOpenKeyExA — open a registry key (ANSI).
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn RegOpenKeyExA(
    hkey: isize,
    sub_key: *const u8,
    _options: u32,
    _desired: u32,
    result_key: *mut isize,
) -> u32 {
    let sub = unsafe { read_cstr(sub_key) }.unwrap_or_default();
    reg_open_key_impl(hkey, &sub, result_key)
}

/// RegOpenKeyExW — open a registry key (wide).
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn RegOpenKeyExW(
    hkey: isize,
    sub_key: *const u16,
    _options: u32,
    _desired: u32,
    result_key: *mut isize,
) -> u32 {
    let sub = unsafe { read_wstr(sub_key) }.unwrap_or_default();
    reg_open_key_impl(hkey, &sub, result_key)
}

fn reg_open_key_impl(hkey: isize, sub_key: &str, result_key: *mut isize) -> u32 {
    if result_key.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    let (root, base_path) = match resolve_key(hkey) {
        Some(r) => r,
        None => return ERROR_INVALID_HANDLE,
    };

    let full_path = join_path(&base_path, sub_key);

    // Check the key exists.
    let exists = registry_store()
        .with_root(root, |root_key| root_key.open_subkey(&full_path).is_some())
        .unwrap_or(false);

    if !exists {
        tracing::debug!(root, path = %full_path, "RegOpenKeyEx: key not found");
        return ERROR_FILE_NOT_FOUND;
    }

    // Create a handle for this opened key.
    let handle = handle_table().insert(HandleEntry::RegistryKey(RegistryKeyState {
        root,
        path: full_path.clone(),
    }));
    rine_types::dev_notify!(on_handle_created(
        handle.as_raw() as i64,
        "RegistryKey",
        &full_path
    ));

    unsafe { *result_key = handle.as_raw() };
    ERROR_SUCCESS
}

// ---------------------------------------------------------------------------
// RegCreateKeyExA / RegCreateKeyExW
// ---------------------------------------------------------------------------

/// RegCreateKeyExA — create or open a registry key (ANSI).
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn RegCreateKeyExA(
    hkey: isize,
    sub_key: *const u8,
    _reserved: u32,
    _class: *const u8,
    _options: u32,
    _desired: u32,
    _security: usize,
    result_key: *mut isize,
    _disposition: *mut u32,
) -> u32 {
    let sub = unsafe { read_cstr(sub_key) }.unwrap_or_default();
    reg_create_key_impl(hkey, &sub, result_key)
}

/// RegCreateKeyExW — create or open a registry key (wide).
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn RegCreateKeyExW(
    hkey: isize,
    sub_key: *const u16,
    _reserved: u32,
    _class: *const u16,
    _options: u32,
    _desired: u32,
    _security: usize,
    result_key: *mut isize,
    _disposition: *mut u32,
) -> u32 {
    let sub = unsafe { read_wstr(sub_key) }.unwrap_or_default();
    reg_create_key_impl(hkey, &sub, result_key)
}

fn reg_create_key_impl(hkey: isize, sub_key: &str, result_key: *mut isize) -> u32 {
    if result_key.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    let (root, base_path) = match resolve_key(hkey) {
        Some(r) => r,
        None => return ERROR_INVALID_HANDLE,
    };

    let full_path = join_path(&base_path, sub_key);

    // Create the key (and any intermediates).
    registry_store().with_root_mut(root, |root_key| {
        root_key.create_subkey(&full_path);
    });

    let handle = handle_table().insert(HandleEntry::RegistryKey(RegistryKeyState {
        root,
        path: full_path.clone(),
    }));
    rine_types::dev_notify!(on_handle_created(
        handle.as_raw() as i64,
        "RegistryKey",
        &full_path
    ));

    unsafe { *result_key = handle.as_raw() };
    ERROR_SUCCESS
}

// ---------------------------------------------------------------------------
// RegQueryValueExA / RegQueryValueExW
// ---------------------------------------------------------------------------

/// RegQueryValueExA — query a registry value (ANSI).
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn RegQueryValueExA(
    hkey: isize,
    value_name: *const u8,
    _reserved: *const u32,
    value_type: *mut u32,
    data: *mut u8,
    data_size: *mut u32,
) -> u32 {
    let name = unsafe { read_cstr(value_name) }.unwrap_or_default();
    reg_query_value_impl(hkey, &name, value_type, data, data_size)
}

/// RegQueryValueExW — query a registry value (wide).
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn RegQueryValueExW(
    hkey: isize,
    value_name: *const u16,
    _reserved: *const u32,
    value_type: *mut u32,
    data: *mut u8,
    data_size: *mut u32,
) -> u32 {
    let name = unsafe { read_wstr(value_name) }.unwrap_or_default();
    reg_query_value_impl(hkey, &name, value_type, data, data_size)
}

fn reg_query_value_impl(
    hkey: isize,
    value_name: &str,
    value_type: *mut u32,
    data: *mut u8,
    data_size: *mut u32,
) -> u32 {
    let (root, path) = match resolve_key(hkey) {
        Some(r) => r,
        None => return ERROR_INVALID_HANDLE,
    };

    let result = registry_store().with_root(root, |root_key| {
        let key = if path.is_empty() {
            root_key
        } else {
            match root_key.open_subkey(&path) {
                Some(k) => k,
                None => return Err(ERROR_FILE_NOT_FOUND),
            }
        };

        match key.get_value(value_name) {
            Some(val) => Ok((val.reg_type(), val.to_bytes())),
            None => Err(ERROR_FILE_NOT_FOUND),
        }
    });

    let (reg_type, bytes) = match result {
        Some(Ok(v)) => v,
        Some(Err(e)) => return e,
        None => return ERROR_INVALID_HANDLE,
    };

    // Write type if requested.
    if !value_type.is_null() {
        unsafe { *value_type = reg_type };
    }

    // Write data if buffer provided.
    if !data_size.is_null() {
        let buf_size = unsafe { *data_size } as usize;
        let needed = bytes.len();

        // Always report the needed size.
        unsafe { *data_size = needed as u32 };

        if data.is_null() {
            // Query size only — success.
            return ERROR_SUCCESS;
        }

        if buf_size < needed {
            return ERROR_MORE_DATA;
        }

        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), data, needed);
        }
    }

    ERROR_SUCCESS
}

// ---------------------------------------------------------------------------
// RegSetValueExA / RegSetValueExW
// ---------------------------------------------------------------------------

/// RegSetValueExA — set a registry value (ANSI).
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn RegSetValueExA(
    hkey: isize,
    value_name: *const u8,
    _reserved: u32,
    value_type: u32,
    data: *const u8,
    data_size: u32,
) -> u32 {
    let name = unsafe { read_cstr(value_name) }.unwrap_or_default();
    reg_set_value_impl(hkey, &name, value_type, data, data_size)
}

/// RegSetValueExW — set a registry value (wide).
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn RegSetValueExW(
    hkey: isize,
    value_name: *const u16,
    _reserved: u32,
    value_type: u32,
    data: *const u8,
    data_size: u32,
) -> u32 {
    let name = unsafe { read_wstr(value_name) }.unwrap_or_default();
    reg_set_value_impl(hkey, &name, value_type, data, data_size)
}

fn reg_set_value_impl(
    hkey: isize,
    value_name: &str,
    value_type: u32,
    data: *const u8,
    data_size: u32,
) -> u32 {
    if data.is_null() && data_size > 0 {
        return ERROR_INVALID_PARAMETER;
    }

    let (root, path) = match resolve_key(hkey) {
        Some(r) => r,
        None => return ERROR_INVALID_HANDLE,
    };

    let bytes = if data.is_null() {
        Vec::new()
    } else {
        unsafe { std::slice::from_raw_parts(data, data_size as usize) }.to_vec()
    };

    let value = match value_type {
        registry::REG_DWORD => {
            if bytes.len() >= 4 {
                RegistryValue::Dword(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
            } else {
                return ERROR_INVALID_PARAMETER;
            }
        }
        registry::REG_QWORD => {
            if bytes.len() >= 8 {
                RegistryValue::Qword(u64::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                ]))
            } else {
                return ERROR_INVALID_PARAMETER;
            }
        }
        registry::REG_SZ | registry::REG_EXPAND_SZ => {
            // Decode UTF-16LE, strip trailing null.
            let wide: Vec<u16> = bytes
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            let s = String::from_utf16_lossy(&wide);
            let s = s.trim_end_matches('\0').to_string();
            if value_type == registry::REG_EXPAND_SZ {
                RegistryValue::ExpandString(s)
            } else {
                RegistryValue::String(s)
            }
        }
        registry::REG_BINARY => RegistryValue::Binary(bytes),
        _ => RegistryValue::Binary(bytes),
    };

    let result = registry_store().with_root_mut(root, |root_key| {
        let key = if path.is_empty() {
            root_key
        } else {
            root_key.create_subkey(&path)
        };
        key.set_value(value_name.to_string(), value);
    });

    match result {
        Some(()) => ERROR_SUCCESS,
        None => ERROR_INVALID_HANDLE,
    }
}

// ---------------------------------------------------------------------------
// RegCloseKey
// ---------------------------------------------------------------------------

/// RegCloseKey — close a registry key handle.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn RegCloseKey(hkey: isize) -> u32 {
    // Predefined root keys cannot be closed.
    if is_predefined_key(hkey) {
        return ERROR_SUCCESS;
    }

    let handle = Handle::from_raw(hkey);
    match handle_table().remove(handle) {
        Some(HandleEntry::RegistryKey(_)) => ERROR_SUCCESS,
        Some(other) => {
            // Not a registry key — put it back.
            handle_table().insert(other);
            ERROR_INVALID_HANDLE
        }
        None => ERROR_INVALID_HANDLE,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reg_open_key_existing() {
        let mut result: isize = 0;
        let sub = b"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\0";
        let err = unsafe {
            RegOpenKeyExA(
                registry::HKEY_LOCAL_MACHINE,
                sub.as_ptr(),
                0,
                0,
                &mut result,
            )
        };
        assert_eq!(err, ERROR_SUCCESS);
        assert_ne!(result, 0);
        unsafe { RegCloseKey(result) };
    }

    #[test]
    fn reg_open_key_nonexistent() {
        let mut result: isize = 0;
        let sub = b"NONEXISTENT\\KEY\\PATH\0";
        let err = unsafe {
            RegOpenKeyExA(
                registry::HKEY_LOCAL_MACHINE,
                sub.as_ptr(),
                0,
                0,
                &mut result,
            )
        };
        assert_eq!(err, ERROR_FILE_NOT_FOUND);
    }

    #[test]
    fn reg_query_dword_value() {
        // Open HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion
        let mut hkey: isize = 0;
        let sub = b"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\0";
        let err =
            unsafe { RegOpenKeyExA(registry::HKEY_LOCAL_MACHINE, sub.as_ptr(), 0, 0, &mut hkey) };
        assert_eq!(err, ERROR_SUCCESS);

        let name = b"CurrentMajorVersionNumber\0";
        let mut reg_type: u32 = 0;
        let mut data = [0u8; 4];
        let mut size: u32 = 4;
        let err = unsafe {
            RegQueryValueExA(
                hkey,
                name.as_ptr(),
                std::ptr::null(),
                &mut reg_type,
                data.as_mut_ptr(),
                &mut size,
            )
        };
        assert_eq!(err, ERROR_SUCCESS);
        assert_eq!(reg_type, registry::REG_DWORD);
        assert_eq!(u32::from_le_bytes(data), 10);

        unsafe { RegCloseKey(hkey) };
    }

    #[test]
    fn reg_query_string_value() {
        let mut hkey: isize = 0;
        let sub = b"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\0";
        let err =
            unsafe { RegOpenKeyExA(registry::HKEY_LOCAL_MACHINE, sub.as_ptr(), 0, 0, &mut hkey) };
        assert_eq!(err, ERROR_SUCCESS);

        let name = b"ProductName\0";
        let mut reg_type: u32 = 0;
        let mut size: u32 = 0;

        // First call: query size.
        let err = unsafe {
            RegQueryValueExA(
                hkey,
                name.as_ptr(),
                std::ptr::null(),
                &mut reg_type,
                std::ptr::null_mut(),
                &mut size,
            )
        };
        assert_eq!(err, ERROR_SUCCESS);
        assert_eq!(reg_type, registry::REG_SZ);
        assert!(size > 0);

        // Second call: read data.
        let mut buf = vec![0u8; size as usize];
        let err = unsafe {
            RegQueryValueExA(
                hkey,
                name.as_ptr(),
                std::ptr::null(),
                &mut reg_type,
                buf.as_mut_ptr(),
                &mut size,
            )
        };
        assert_eq!(err, ERROR_SUCCESS);

        // Decode UTF-16LE
        let wide: Vec<u16> = buf
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        let s = String::from_utf16_lossy(&wide);
        let s = s.trim_end_matches('\0');
        assert_eq!(s, "Windows 10 Pro");

        unsafe { RegCloseKey(hkey) };
    }

    #[test]
    fn reg_query_buffer_too_small() {
        let mut hkey: isize = 0;
        let sub = b"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\0";
        unsafe { RegOpenKeyExA(registry::HKEY_LOCAL_MACHINE, sub.as_ptr(), 0, 0, &mut hkey) };

        let name = b"ProductName\0";
        let mut reg_type: u32 = 0;
        let mut data = [0u8; 2]; // too small
        let mut size: u32 = 2;
        let err = unsafe {
            RegQueryValueExA(
                hkey,
                name.as_ptr(),
                std::ptr::null(),
                &mut reg_type,
                data.as_mut_ptr(),
                &mut size,
            )
        };
        assert_eq!(err, ERROR_MORE_DATA);
        assert!(size > 2, "should report needed size");

        unsafe { RegCloseKey(hkey) };
    }

    #[test]
    fn reg_create_and_set_value() {
        let mut hkey: isize = 0;
        let sub = b"Software\\RineTestApp\0";
        let err = unsafe {
            RegCreateKeyExA(
                registry::HKEY_CURRENT_USER,
                sub.as_ptr(),
                0,
                std::ptr::null(),
                0,
                0,
                0,
                &mut hkey,
                std::ptr::null_mut(),
            )
        };
        assert_eq!(err, ERROR_SUCCESS);

        // Set a DWORD value.
        let name = b"TestSetting\0";
        let value: u32 = 12345;
        let err = unsafe {
            RegSetValueExA(
                hkey,
                name.as_ptr(),
                0,
                registry::REG_DWORD,
                &value as *const u32 as *const u8,
                4,
            )
        };
        assert_eq!(err, ERROR_SUCCESS);

        // Read it back.
        let mut out_type: u32 = 0;
        let mut out_data = [0u8; 4];
        let mut out_size: u32 = 4;
        let err = unsafe {
            RegQueryValueExA(
                hkey,
                name.as_ptr(),
                std::ptr::null(),
                &mut out_type,
                out_data.as_mut_ptr(),
                &mut out_size,
            )
        };
        assert_eq!(err, ERROR_SUCCESS);
        assert_eq!(out_type, registry::REG_DWORD);
        assert_eq!(u32::from_le_bytes(out_data), 12345);

        unsafe { RegCloseKey(hkey) };
    }

    #[test]
    fn reg_close_predefined_key_succeeds() {
        let err = unsafe { RegCloseKey(registry::HKEY_LOCAL_MACHINE) };
        assert_eq!(err, ERROR_SUCCESS);
    }

    #[test]
    fn reg_close_invalid_handle() {
        let err = unsafe { RegCloseKey(0xDEAD) };
        assert_eq!(err, ERROR_INVALID_HANDLE);
    }

    #[test]
    fn reg_open_on_predefined_root_directly() {
        // Using predefined key directly as parent with empty sub-key
        let mut result: isize = 0;
        let sub = b"\0"; // empty string
        let err = unsafe {
            RegOpenKeyExA(
                registry::HKEY_LOCAL_MACHINE,
                sub.as_ptr(),
                0,
                0,
                &mut result,
            )
        };
        assert_eq!(err, ERROR_SUCCESS);
        unsafe { RegCloseKey(result) };
    }

    #[test]
    fn reg_query_on_predefined_root() {
        // Query directly on HKEY_LOCAL_MACHINE should work
        // (though there are no values on the root itself)
        let name = b"NonExistent\0";
        let mut reg_type: u32 = 0;
        let mut size: u32 = 0;
        let err = unsafe {
            RegQueryValueExA(
                registry::HKEY_LOCAL_MACHINE,
                name.as_ptr(),
                std::ptr::null(),
                &mut reg_type,
                std::ptr::null_mut(),
                &mut size,
            )
        };
        assert_eq!(err, ERROR_FILE_NOT_FOUND);
    }

    #[test]
    fn reg_set_and_query_string() {
        let mut hkey: isize = 0;
        let sub = b"Software\\RineTestStrings\0";
        unsafe {
            RegCreateKeyExA(
                registry::HKEY_CURRENT_USER,
                sub.as_ptr(),
                0,
                std::ptr::null(),
                0,
                0,
                0,
                &mut hkey,
                std::ptr::null_mut(),
            )
        };

        // Set a REG_SZ value via UTF-16LE bytes.
        let name = b"Greeting\0";
        let wide: Vec<u16> = "Hello\0".encode_utf16().collect();
        let bytes: Vec<u8> = wide.iter().flat_map(|w| w.to_le_bytes()).collect();
        let err = unsafe {
            RegSetValueExA(
                hkey,
                name.as_ptr(),
                0,
                registry::REG_SZ,
                bytes.as_ptr(),
                bytes.len() as u32,
            )
        };
        assert_eq!(err, ERROR_SUCCESS);

        // Read it back.
        let mut out_type: u32 = 0;
        let mut out_size: u32 = 0;
        unsafe {
            RegQueryValueExA(
                hkey,
                name.as_ptr(),
                std::ptr::null(),
                &mut out_type,
                std::ptr::null_mut(),
                &mut out_size,
            )
        };
        let mut out_buf = vec![0u8; out_size as usize];
        unsafe {
            RegQueryValueExA(
                hkey,
                name.as_ptr(),
                std::ptr::null(),
                &mut out_type,
                out_buf.as_mut_ptr(),
                &mut out_size,
            )
        };
        assert_eq!(out_type, registry::REG_SZ);
        let out_wide: Vec<u16> = out_buf
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        let s = String::from_utf16_lossy(&out_wide)
            .trim_end_matches('\0')
            .to_string();
        assert_eq!(s, "Hello");

        unsafe { RegCloseKey(hkey) };
    }
}
