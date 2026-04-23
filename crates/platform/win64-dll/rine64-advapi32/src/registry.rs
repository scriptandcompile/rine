use rine_common_advapi32 as common;
use rine_types::strings::{read_cstr, read_wstr};

/// Open a registry key, returning a handle to the key in `result_key`.
///
/// # Arguments
/// * `hkey`: Handle to an open registry key, or one of the predefined root keys.
/// * `sub_key`: Name of the subkey to open, relative to `hkey`.
/// * `_options`: Reserved, must be 0.
/// * `_desired`: Access rights, currently ignored.
/// * `result_key`: Pointer to a variable that receives the handle to the opened key.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers and interacts with the
/// Windows registry, which can lead to undefined behavior or system instability if used incorrectly.
/// The caller must ensure that the pointers are valid and that the registry operations are
/// performed with appropriate permissions and caution.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn RegOpenKeyExA(
    hkey: isize,
    sub_key: *const u8,
    _options: u32,
    _desired: u32,
    result_key: *mut isize,
) -> u32 {
    unsafe {
        let sub = read_cstr(sub_key).unwrap_or_default();
        common::registry::reg_open_key(hkey, &sub, _options, _desired, result_key)
    }
}

/// Open a registry key, returning a handle to the key in `result_key`.
///
/// # Arguments
/// * `hkey`: Handle to an open registry key, or one of the predefined root keys.
/// * `sub_key`: Name of the subkey to open, relative to `hkey`.
/// * `_options`: Reserved, must be 0.
/// * `_desired`: Access rights, currently ignored.
/// * `result_key`: Pointer to a variable that receives the handle to the opened key.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers and interacts with the
/// Windows registry, which can lead to undefined behavior or system instability if used incorrectly.
/// The caller must ensure that the pointers are valid and that the registry operations are
/// performed with appropriate permissions and caution.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn RegOpenKeyExW(
    hkey: isize,
    sub_key: *const u16,
    _options: u32,
    _desired: u32,
    result_key: *mut isize,
) -> u32 {
    unsafe {
        let sub = read_wstr(sub_key).unwrap_or_default();
        common::registry::reg_open_key(hkey, &sub, _options, _desired, result_key)
    }
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
#[rine_dlls::partial]
#[allow(non_snake_case, clippy::too_many_arguments)]
#[unsafe(no_mangle)]
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
    unsafe {
        let sub = read_cstr(sub_key).unwrap_or_default();
        let _class_type = read_cstr(_class).unwrap_or_default();
        common::registry::reg_create_key_ex(
            hkey,
            &sub,
            _reserved,
            &_class_type,
            _options,
            _desired,
            _security,
            result_key,
            _disposition,
        )
    }
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
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
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
    unsafe {
        let sub = read_wstr(sub_key).unwrap_or_default();
        let _class_type = read_wstr(_class).unwrap_or_default();
        common::registry::reg_create_key_ex(
            hkey,
            &sub,
            _reserved,
            &_class_type,
            _options,
            _desired,
            _security,
            result_key,
            _disposition,
        )
    }
}

/// Retrieve the type and data for a specified value name associated with an open registry key.
///
/// # Arguments
/// * `hkey`: Handle to an open registry key.
/// * `value_name`: Name of the registry value to query.
/// * `_reserved`: Reserved, must be null.
/// * `value_type`: Pointer to a variable that receives the type of the registry value.
/// * `data`: Pointer to a buffer that receives the data for the registry value.
/// * `data_size`: Pointer to a variable that specifies the size of the buffer pointed to by `data`,
///   and receives the size of the data returned.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers and interacts with the Windows registry,
/// which can lead to undefined behavior or system instability if used incorrectly.
/// The caller must ensure that the pointers are valid and that the registry operations are performed
/// with appropriate permissions and caution.
///
/// # Returns
/// Returns `ERROR_SUCCESS` if the function succeeds, or a nonzero error code if it fails.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn RegQueryValueExA(
    hkey: isize,
    value_name: *const u8,
    _reserved: *const u32,
    value_type: *mut u32,
    data: *mut u8,
    data_size: *mut u32,
) -> u32 {
    unsafe {
        let name = read_cstr(value_name).unwrap_or_default();
        common::registry::reg_query_value(hkey, &name, _reserved, value_type, data, data_size)
    }
}

/// Retrieve the type and data for a specified value name associated with an open registry key.
///
/// # Arguments
/// * `hkey`: Handle to an open registry key.
/// * `value_name`: Name of the registry value to query.
/// * `_reserved`: Reserved, must be null.
/// * `value_type`: Pointer to a variable that receives the type of the registry value.
/// * `data`: Pointer to a buffer that receives the data for the registry value.
/// * `data_size`: Pointer to a variable that specifies the size of the buffer pointed to by `data`,
///   and receives the size of the data returned.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers and interacts with the Windows registry,
/// which can lead to undefined behavior or system instability if used incorrectly.
/// The caller must ensure that the pointers are valid and that the registry operations are performed
/// with appropriate permissions and caution.
///
/// # Returns
/// Returns `ERROR_SUCCESS` if the function succeeds, or a nonzero error code if it fails.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn RegQueryValueExW(
    hkey: isize,
    value_name: *const u16,
    _reserved: *const u32,
    value_type: *mut u32,
    data: *mut u8,
    data_size: *mut u32,
) -> u32 {
    unsafe {
        let name = read_wstr(value_name).unwrap_or_default();
        common::registry::reg_query_value(hkey, &name, _reserved, value_type, data, data_size)
    }
}

/// Set the data and type of a specified value under an open registry key.
///
/// # Arguments
/// * `hkey`: Handle to an open registry key.
/// * `value_name`: Name of the registry value to set.
/// * `_reserved`: Reserved, must be 0.
/// * `value_type`: Type of the registry value.
/// * `data`: Pointer to a buffer that contains the data for the registry value.
/// * `data_size`: Size of the data pointed to by `data`.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers and interacts with the Windows registry,
/// which can lead to undefined behavior or system instability if used incorrectly.
/// The caller must ensure that the pointers are valid and that the registry operations are performed
/// with appropriate permissions and caution.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn RegSetValueExA(
    hkey: isize,
    value_name: *const u8,
    _reserved: u32,
    value_type: u32,
    data: *const u8,
    data_size: u32,
) -> u32 {
    unsafe {
        let name = read_cstr(value_name).unwrap_or_default();
        common::registry::reg_set_value(hkey, &name, _reserved, value_type, data, data_size)
    }
}

/// Set the data and type of a specified value under an open registry key.
///
/// # Arguments
/// * `hkey`: Handle to an open registry key.
/// * `value_name`: Name of the registry value to set.
/// * `_reserved`: Reserved, must be 0.
/// * `value_type`: Type of the registry value.
/// * `data`: Pointer to a buffer that contains the data for the registry value.
/// * `data_size`: Size of the data pointed to by `data`.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers and interacts with the Windows registry,
/// which can lead to undefined behavior or system instability if used incorrectly.
/// The caller must ensure that the pointers are valid and that the registry operations are performed
/// with appropriate permissions and caution.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn RegSetValueExW(
    hkey: isize,
    value_name: *const u16,
    _reserved: u32,
    value_type: u32,
    data: *const u8,
    data_size: u32,
) -> u32 {
    unsafe {
        let name = read_wstr(value_name).unwrap_or_default();
        common::registry::reg_set_value(hkey, &name, _reserved, value_type, data, data_size)
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
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn RegCloseKey(hkey: isize) -> u32 {
    unsafe { common::registry::reg_close_key(hkey) }
}
