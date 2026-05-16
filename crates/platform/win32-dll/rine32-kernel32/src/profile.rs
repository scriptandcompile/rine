use rine_common_kernel32::profile as common;
use rine_types::errors::BOOL;
use rine_types::strings::{LPCSTR, LPCWSTR, LPSTR, LPWSTR};

/// Get a string from the WIN.INI file (routed through the registry).
///
/// # Arguments
/// - `lpAppName`: The section name in the INI file (e.g. "Windows" or "boot").
/// - `lpKeyName`: The key name in the INI file (e.g. "shell" or "win"). If null, retrieves the section name itself.
/// - `lpDefault`: The default value to return if the section/key is not found.
/// - `lpReturnedString`: A buffer to receive the result string.
/// - `nSize`: The size of the buffer in characters.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers and writes to a buffer.
///
/// # Returns
/// The number of characters copied to the buffer, not including the null terminator.
/// If the buffer is too small, the return value is `nSize - 1` and the string is truncated.
/// If the section/key is not found, the default value is copied to the buffer and its length is returned.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetProfileStringA(
    lpAppName: LPCSTR,
    lpKeyName: LPCSTR,
    lpDefault: LPCSTR,
    lpReturnedString: LPSTR,
    nSize: u32,
) -> u32 {
    unsafe {
        let section = lpAppName.read_string().unwrap_or_default();
        let key = lpKeyName.read_string().unwrap_or_default();
        let default = lpDefault.read_string().unwrap_or_default();

        common::get_profile_string_a(&section, &key, &default, lpReturnedString, nSize)
    }
}

/// Get a string from the WIN.INI file (routed through the registry).
///
/// # Arguments
/// - `lpAppName`: The section name in the INI file (e.g. "Windows" or "boot").
/// - `lpKeyName`: The key name in the INI file (e.g. "shell" or "win"). If null, retrieves the section name itself.
/// - `lpDefault`: The default value to return if the section/key is not found.
/// - `lpReturnedString`: A buffer to receive the result string.
/// - `nSize`: The size of the buffer in characters.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers and writes to a buffer.
///
/// # Returns
/// The number of characters copied to the buffer, not including the null terminator.
/// If the buffer is too small, the return value is `nSize - 1` and the string is truncated.
/// If the section/key is not found, the default value is copied to the buffer and its length is returned.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetProfileStringW(
    lpAppName: LPCWSTR,
    lpKeyName: LPCWSTR,
    lpDefault: LPCWSTR,
    lpReturnedString: LPWSTR,
    nSize: u32,
) -> u32 {
    unsafe {
        let section = lpAppName.read_string().unwrap_or_default();
        let key = lpKeyName.read_string().unwrap_or_default();
        let default = lpDefault.read_string().unwrap_or_default();

        common::get_profile_string_w(&section, &key, &default, lpReturnedString, nSize)
    }
}

/// Write a string to the WIN.INI file (routed through the registry).
///
/// # Arguments
/// - `lpAppName`: The section name in the INI file (e.g. "Windows" or "boot").
/// - `lpKeyName`: The key name in the INI file (e.g. "shell" or "win").
/// - `lpString`: The string value to write. If null, the key will be deleted.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers and reads strings from them.
///
/// # Returns
/// `BOOL::TRUE` if the operation succeeded, or `BOOL::FALSE` if it failed.
///
/// # Notes
/// This function does not yet implement setting `GetLastError` on failure.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn WriteProfileStringA(
    lpAppName: LPCSTR,
    lpKeyName: LPCSTR,
    lpString: LPCSTR,
) -> BOOL {
    unsafe {
        let section = lpAppName.read_string().unwrap_or_default();
        let key = lpKeyName.read_string().unwrap_or_default();
        let value = lpString.read_string();

        common::write_profile_string(&section, &key, value.as_deref())
    }
}

/// Write a string to the WIN.INI file in wide form (routed through the registry).
///
/// # Arguments
/// - `lpAppName`: The section name in the INI file (e.g. "Windows" or "boot").
/// - `lpKeyName`: The key name in the INI file (e.g. "shell" or "win").
/// - `lpString`: The string value to write. If null, the key will be deleted.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers and reads strings from them.
///
/// # Returns
/// `BOOL::TRUE` if the operation succeeded, or `BOOL::FALSE` if it failed.
///
/// # Notes
/// This function does not yet implement setting `GetLastError` on failure.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn WriteProfileStringW(
    lpAppName: LPCWSTR,
    lpKeyName: LPCWSTR,
    lpString: LPCWSTR,
) -> BOOL {
    unsafe {
        let section = lpAppName.read_string().unwrap_or_default();
        let key = lpKeyName.read_string().unwrap_or_default();
        let value = lpString.read_string();

        common::write_profile_string(&section, &key, value.as_deref())
    }
}
/// Get a string from a private INI file (ANSI form).
///
/// # Arguments
/// - `lpAppName`: The section name in the INI file (e.g. "Windows" or "boot").
/// - `lpKeyName`: The key name in the INI file (e.g. "shell" or "win").
/// - `lpDefault`: The default value to return if the section/key is not found.
/// - `lpReturnedString`: A buffer to receive the result string.
/// - `nSize`: The size of the buffer in characters.
/// - `lpFileName`: The name of the INI file to read from.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers and writes to a buffer.
///
/// # Returns
/// The number of characters copied to the buffer, not including the null terminator.
/// If the buffer is too small, the return value is `nSize - 1` and the string is truncated.
/// If the section/key is not found, the default value is copied to the buffer and its length is returned.
///
/// # Notes
/// This function does not yet implement setting `GetLastError` on failure.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetPrivateProfileStringA(
    lpAppName: LPCSTR,
    lpKeyName: LPCSTR,
    lpDefault: LPCSTR,
    lpReturnedString: LPSTR,
    nSize: u32,
    lpFileName: LPCSTR,
) -> u32 {
    unsafe {
        let section = lpAppName.read_string().unwrap_or_default();
        let key = lpKeyName.read_string().unwrap_or_default();
        let default = lpDefault.read_string().unwrap_or_default();
        let file_name = lpFileName.read_string().unwrap_or_default();

        common::get_private_profile_string_a(
            &section,
            &key,
            &default,
            lpReturnedString,
            nSize,
            &file_name,
        )
    }
}

/// Get a string from a private INI file (wide form).
///
/// # Arguments
/// - `lpAppName`: The section name in the INI file (e.g. "Windows" or "boot").
/// - `lpKeyName`: The key name in the INI file (e.g. "shell" or "win").
/// - `lpDefault`: The default value to return if the section/key is not found.
/// - `lpReturnedString`: A buffer to receive the result string.
/// - `nSize`: The size of the buffer in characters.
/// - `lpFileName`: The name of the INI file to read from.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers and writes to a buffer.
///
/// # Returns
/// The number of characters copied to the buffer, not including the null terminator.
/// If the buffer is too small, the return value is `nSize - 1` and the string is truncated.
/// If the section/key is not found, the default value is copied to the buffer and its length is returned.
///
/// # Notes
/// This function does not yet implement setting `GetLastError` on failure.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetPrivateProfileStringW(
    lpAppName: LPCWSTR,
    lpKeyName: LPCWSTR,
    lpDefault: LPCWSTR,
    lpReturnedString: LPWSTR,
    nSize: u32,
    lpFileName: LPCWSTR,
) -> u32 {
    unsafe {
        let section = lpAppName.read_string().unwrap_or_default();
        let key = lpKeyName.read_string().unwrap_or_default();
        let default = lpDefault.read_string().unwrap_or_default();
        let file_name = lpFileName.read_string().unwrap_or_default();

        common::get_private_profile_string_w(
            &section,
            &key,
            &default,
            lpReturnedString,
            nSize,
            &file_name,
        )
    }
}

/// Write a string to a private INI file (ANSI form).
///
/// # Arguments
/// - `lpAppName`: The section name in the INI file (e.g. "Windows" or "boot").
/// - `lpKeyName`: The key name in the INI file (e.g. "shell" or "win").
/// - `lpString`: The string value to write. If null, the key will be deleted.
/// - `lpFileName`: The name of the INI file to write to.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers and reads strings from them.
///
/// # Returns
/// `BOOL::TRUE` if the operation succeeded, or `BOOL::FALSE` if it failed.
///
/// # Notes
/// This function does not yet implement setting `GetLastError` on failure.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn WritePrivateProfileStringA(
    lpAppName: LPCSTR,
    lpKeyName: LPCSTR,
    lpString: LPCSTR,
    lpFileName: LPCSTR,
) -> BOOL {
    unsafe {
        let section = lpAppName.read_string().unwrap_or_default();
        let key = lpKeyName.read_string().unwrap_or_default();
        let value = lpString.read_string();
        let file_name = lpFileName.read_string().unwrap_or_default();

        common::write_private_profile_string_a(&section, &key, value.as_deref(), &file_name)
    }
}

/// Write a string to a private INI file (wide form).
///
/// # Arguments
/// - `lpAppName`: The section name in the INI file (e.g. "Windows" or "boot").
/// - `lpKeyName`: The key name in the INI file (e.g. "shell" or "win").
/// - `lpString`: The string value to write. If null, the key will be deleted.
/// - `lpFileName`: The name of the INI file to write to.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers and reads strings from them.
///
/// # Returns
/// `BOOL::TRUE` if the operation succeeded, or `BOOL::FALSE` if it failed.
///
/// # Notes
/// This function does not yet implement setting `GetLastError` on failure.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn WritePrivateProfileStringW(
    lpAppName: LPCWSTR,
    lpKeyName: LPCWSTR,
    lpString: LPCWSTR,
    lpFileName: LPCWSTR,
) -> BOOL {
    unsafe {
        let section = lpAppName.read_string().unwrap_or_default();
        let key = lpKeyName.read_string().unwrap_or_default();
        let value = lpString.read_string();
        let file_name = lpFileName.read_string().unwrap_or_default();

        common::write_private_profile_string_a(&section, &key, value.as_deref(), &file_name)
    }
}
