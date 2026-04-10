use rine_common_advapi32 as common;
use rine_types::strings::{read_cstr, read_wstr};

#[allow(non_snake_case, clippy::missing_safety_doc)]
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

#[allow(non_snake_case, clippy::missing_safety_doc)]
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

#[allow(non_snake_case, clippy::missing_safety_doc)]
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
        common::registry::RegCreateKeyExA(
            hkey,
            sub_key,
            _reserved,
            _class,
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
        common::registry::RegCreateKeyExW(
            hkey,
            sub_key,
            _reserved,
            _class,
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
pub unsafe extern "win64" fn RegQueryValueExA(
    hkey: isize,
    value_name: *const u8,
    _reserved: *const u32,
    value_type: *mut u32,
    data: *mut u8,
    data_size: *mut u32,
) -> u32 {
    unsafe {
        common::registry::RegQueryValueExA(hkey, value_name, _reserved, value_type, data, data_size)
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
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
        common::registry::RegQueryValueExW(hkey, value_name, _reserved, value_type, data, data_size)
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
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
        common::registry::RegSetValueExA(hkey, value_name, _reserved, value_type, data, data_size)
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
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
        common::registry::RegSetValueExW(hkey, value_name, _reserved, value_type, data, data_size)
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn RegCloseKey(hkey: isize) -> u32 {
    unsafe { common::registry::RegCloseKey(hkey) }
}
