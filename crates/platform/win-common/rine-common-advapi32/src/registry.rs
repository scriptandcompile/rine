//! advapi32 registry functions: RegOpenKeyExA/W, RegQueryValueExA/W,
//! RegSetValueExA/W, RegCreateKeyExA/W, RegCloseKey.

use rine_types::errors::{
    ERROR_FILE_NOT_FOUND, ERROR_INVALID_HANDLE, ERROR_INVALID_PARAMETER, ERROR_SUCCESS,
};
use rine_types::handles::{HANDLE, HandleEntry, handle_table};
use rine_types::registry::{
    self, RegistryKeyState, RegistryValue, is_predefined_key, registry_store,
};

const ERROR_MORE_DATA: u32 = 234;

fn resolve_key(hkey: isize) -> Option<(isize, String)> {
    if is_predefined_key(hkey) {
        return Some((hkey, String::new()));
    }
    let handle = HANDLE::from_raw(hkey);
    handle_table().with_registry_key(handle, |state| (state.root, state.path.clone()))
}

fn join_path(base: &str, sub: &str) -> String {
    if base.is_empty() {
        sub.to_string()
    } else if sub.is_empty() {
        base.to_string()
    } else {
        format!("{base}\\{sub}")
    }
}

/// Opens a registry key.
///
/// # Arguments
/// * `hkey`: Handle to an open registry key, or one of the predefined root keys.
/// * `sub_key`: Name of the subkey to open, relative to `hkey`.
/// * `_options`: Reserved, must be 0.
/// * `_desired`: Access rights, currently ignored.
/// * `result_key`: Pointer to a variable that receives the handle to the opened key.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers and interacts with global state.
///
/// # Returns
/// Returns `ERROR_SUCCESS` on success, or an appropriate error code on failure.
pub unsafe fn reg_open_key(
    hkey: isize,
    sub_key: &str,
    _options: u32,
    _desired: u32,
    result_key: *mut isize,
) -> u32 {
    if result_key.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    let (root, base_path) = match resolve_key(hkey) {
        Some(r) => r,
        None => return ERROR_INVALID_HANDLE,
    };

    let full_path = join_path(&base_path, sub_key);

    tracing::debug!(root, path = %full_path, "RegOpenKeyEx: opening key");

    let exists = registry_store()
        .with_root(root, |root_key| root_key.open_subkey(&full_path).is_some())
        .unwrap_or(false);

    if !exists {
        tracing::debug!(root, path = %full_path, "RegOpenKeyEx: key not found");
        return ERROR_FILE_NOT_FOUND;
    }

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

/// Create or open a registry key, returning a handle to the key in `result_key`.
///
/// # Arguments
/// * `hkey`: Handle to an open registry key, or one of the predefined root keys.
/// * `sub_key`: Name of the subkey to create or open, relative to `hkey`.
/// * `_reserved`: Reserved, must be 0.
/// * `_class_type`: Class string, currently ignored.
/// * `_options`: Key creation options, currently ignored.
/// * `_desired`: Access rights, currently ignored.
/// * `_security`: Security attributes, currently ignored.
/// * `result_key`: Pointer to a variable that receives the handle to the created or opened key.
/// * `_disposition`: Pointer to a variable that receives a value indicating whether the key was created or opened, currently ignored.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers and interacts with the
/// Windows registry, which can lead to undefined behavior or system instability if used incorrectly.
/// The caller must ensure that the pointers are valid and that the registry operations are
/// performed with appropriate permissions and caution.
#[allow(clippy::too_many_arguments)]
pub unsafe fn reg_create_key_ex(
    hkey: isize,
    sub_key: &str,
    _reserved: u32,
    _class_type: &str,
    _options: u32,
    _desired: u32,
    _security: usize,
    result_key: *mut isize,
    _disposition: *mut u32,
) -> u32 {
    if result_key.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    let (root, base_path) = match resolve_key(hkey) {
        Some(r) => r,
        None => return ERROR_INVALID_HANDLE,
    };

    let full_path = join_path(&base_path, sub_key);

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

/// Query the value of a registry key.
///
/// # Arguments
/// * `hkey`: Handle to an open registry key, or one of the predefined root keys.
/// * `value_name`: Name of the value to query.
/// * `_reserved`: Reserved, must be a null pointer.
/// * `value_type`: Pointer to a variable that receives the type of the value.
/// * `data`: Pointer to a buffer that receives the value data.
/// * `data_size`: Pointer to a variable that specifies the size of the buffer, and receives the size of the data returned.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers and interacts with the Windows registry,
/// which can lead to undefined behavior or system instability if used incorrectly. The caller must ensure that the
/// pointers are valid and that the registry operations are performed with appropriate permissions and caution.
///
/// # Returns
/// Returns `ERROR_SUCCESS` if the function succeeds, or a nonzero error code if it fails.
pub unsafe fn reg_query_value(
    hkey: isize,
    value_name: &str,
    _reserved: *const u32,
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

    if !value_type.is_null() {
        unsafe { *value_type = reg_type };
    }

    if !data_size.is_null() {
        let buf_size = unsafe { *data_size } as usize;
        let needed = bytes.len();

        unsafe { *data_size = needed as u32 };

        if data.is_null() {
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

/// Set the value of a registry key.
///
/// # Arguments
/// * `hkey`: Handle to an open registry key, or one of the predefined root keys.
/// * `value_name`: Name of the value to set.
/// * `_reserved`: Reserved, must be 0.
/// * `value_type`: Type of the value being set (e.g., REG_DWORD, REG_SZ).
/// * `data`: Pointer to the data to set for the value.
/// * `data_size`: Size of the data being set, in bytes.
///
/// # Returns
/// Returns `ERROR_SUCCESS` if the function succeeds, or a nonzero error code if it fails.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers and interacts with the Windows registry,
/// which can lead to undefined behavior or system instability if used incorrectly.
/// The caller must ensure that the pointers are valid and that the registry operations are performed
/// with appropriate permissions and caution.
pub unsafe fn reg_set_value(
    hkey: isize,
    value_name: &str,
    _reserved: u32,
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

/// Close a registry key handle.
///
/// # Arguments
/// * `hkey`: Handle to a registry key to close.
///   This can be a handle returned by `reg_open_key` or `reg_create_key_ex`, but not a predefined root key handle.
///
/// # Safety
/// This function is unsafe because it interacts with global state and can lead to undefined behavior if used incorrectly.
/// The caller must ensure that the handle is valid and that it is not a predefined root key handle, as closing a predefined
/// root key is not allowed and will not have any effect.
/// Additionally, the caller must ensure that the handle is not used after it has been closed, as this can lead to undefined
/// behavior.
///
/// # Returns
/// Returns `ERROR_SUCCESS` if the function succeeds, or `ERROR_INVALID_HANDLE` if the handle is invalid or refers to a
/// predefined root key.
pub unsafe fn reg_close_key(hkey: isize) -> u32 {
    if is_predefined_key(hkey) {
        return ERROR_SUCCESS;
    }

    let handle = HANDLE::from_raw(hkey);
    match handle_table().remove(handle) {
        Some(HandleEntry::RegistryKey(_)) => ERROR_SUCCESS,
        Some(other) => {
            handle_table().insert(other);
            ERROR_INVALID_HANDLE
        }
        None => ERROR_INVALID_HANDLE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reg_open_key_existing() {
        let mut result: isize = 0;
        let sub = "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion";
        let err = unsafe { reg_open_key(registry::HKEY_LOCAL_MACHINE, sub, 0, 0, &mut result) };
        assert_eq!(err, ERROR_SUCCESS);
        assert_ne!(result, 0);
        unsafe { reg_close_key(result) };
    }

    #[test]
    fn reg_query_dword_value() {
        let mut hkey: isize = 0;
        let sub = "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion";
        let err = unsafe { reg_open_key(registry::HKEY_LOCAL_MACHINE, sub, 0, 0, &mut hkey) };
        assert_eq!(err, ERROR_SUCCESS);

        let name = "CurrentMajorVersionNumber";
        let mut reg_type: u32 = 0;
        let mut data = [0u8; 4];
        let mut size: u32 = 4;
        let err = unsafe {
            reg_query_value(
                hkey,
                name,
                std::ptr::null(),
                &mut reg_type,
                data.as_mut_ptr(),
                &mut size,
            )
        };
        assert_eq!(err, ERROR_SUCCESS);
        assert_eq!(reg_type, registry::REG_DWORD);
        assert_eq!(u32::from_le_bytes(data), 10);

        unsafe { reg_close_key(hkey) };
    }
}
