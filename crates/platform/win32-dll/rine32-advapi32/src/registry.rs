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
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn RegOpenKeyExA(
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
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn RegOpenKeyExW(
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
#[allow(non_snake_case, clippy::too_many_arguments)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn RegCreateKeyExA(
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
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn RegCreateKeyExW(
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

#[allow(non_snake_case, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn RegQueryValueExA(
    hkey: isize,
    value_name: *const u8,
    reserved: *const u32,
    value_type: *mut u32,
    data: *mut u8,
    data_size: *mut u32,
) -> u32 {
    unsafe {
        common::registry::RegQueryValueExA(hkey, value_name, reserved, value_type, data, data_size)
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn RegQueryValueExW(
    hkey: isize,
    value_name: *const u16,
    reserved: *const u32,
    value_type: *mut u32,
    data: *mut u8,
    data_size: *mut u32,
) -> u32 {
    unsafe {
        common::registry::RegQueryValueExW(hkey, value_name, reserved, value_type, data, data_size)
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn RegSetValueExA(
    hkey: isize,
    value_name: *const u8,
    reserved: u32,
    value_type: u32,
    data: *const u8,
    data_size: u32,
) -> u32 {
    unsafe {
        common::registry::RegSetValueExA(hkey, value_name, reserved, value_type, data, data_size)
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn RegSetValueExW(
    hkey: isize,
    value_name: *const u16,
    reserved: u32,
    value_type: u32,
    data: *const u8,
    data_size: u32,
) -> u32 {
    unsafe {
        common::registry::RegSetValueExW(hkey, value_name, reserved, value_type, data, data_size)
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn RegCloseKey(hkey: isize) -> u32 {
    unsafe { common::registry::RegCloseKey(hkey) }
}
