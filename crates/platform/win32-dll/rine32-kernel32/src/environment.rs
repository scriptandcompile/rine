use std::sync::OnceLock;

use rine_types::errors::WinBool;
use rine_types::strings::{read_cstr, read_wstr, write_cstr, write_wstr};

struct SyncPtr(*mut u16);
unsafe impl Send for SyncPtr {}
unsafe impl Sync for SyncPtr {}

static ENV_BLOCK_W: OnceLock<SyncPtr> = OnceLock::new();

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn GetEnvironmentVariableA(
    name: *const u8,
    buffer: *mut u8,
    size: u32,
) -> u32 {
    let var_name = match unsafe { read_cstr(name) } {
        Some(n) => n,
        None => return 0,
    };

    match rine_types::environment::get_var(&var_name) {
        Some(val) => unsafe { write_cstr(buffer, size, &val) },
        None => 0,
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn GetEnvironmentVariableW(
    name: *const u16,
    buffer: *mut u16,
    size: u32,
) -> u32 {
    let var_name = match unsafe { read_wstr(name) } {
        Some(n) => n,
        None => return 0,
    };

    match rine_types::environment::get_var(&var_name) {
        Some(val) => unsafe { write_wstr(buffer, size, &val) },
        None => 0,
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn SetEnvironmentVariableA(
    name: *const u8,
    value: *const u8,
) -> WinBool {
    let var_name = match unsafe { read_cstr(name) } {
        Some(n) => n,
        None => return WinBool::FALSE,
    };
    let var_value = unsafe { read_cstr(value) };
    rine_types::environment::set_var(&var_name, var_value.as_deref());
    WinBool::TRUE
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn SetEnvironmentVariableW(
    name: *const u16,
    value: *const u16,
) -> WinBool {
    let var_name = match unsafe { read_wstr(name) } {
        Some(n) => n,
        None => return WinBool::FALSE,
    };
    let var_value = unsafe { read_wstr(value) };
    rine_types::environment::set_var(&var_name, var_value.as_deref());
    WinBool::TRUE
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn ExpandEnvironmentStringsA(
    src: *const u8,
    dst: *mut u8,
    dst_size: u32,
) -> u32 {
    let input = match unsafe { read_cstr(src) } {
        Some(s) => s,
        None => return 0,
    };

    let expanded = rine_types::environment::expand_vars(&input);
    let needed = expanded.len() as u32 + 1;

    if dst.is_null() || dst_size < needed {
        return needed;
    }

    unsafe {
        core::ptr::copy_nonoverlapping(expanded.as_ptr(), dst, expanded.len());
        *dst.add(expanded.len()) = 0;
    }
    needed
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn ExpandEnvironmentStringsW(
    src: *const u16,
    dst: *mut u16,
    dst_size: u32,
) -> u32 {
    let input = match unsafe { read_wstr(src) } {
        Some(s) => s,
        None => return 0,
    };

    let expanded = rine_types::environment::expand_vars(&input);
    let encoded: Vec<u16> = expanded.encode_utf16().collect();
    let needed = encoded.len() as u32 + 1;

    if dst.is_null() || dst_size < needed {
        return needed;
    }

    unsafe {
        core::ptr::copy_nonoverlapping(encoded.as_ptr(), dst, encoded.len());
        *dst.add(encoded.len()) = 0;
    }
    needed
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn GetEnvironmentStringsW() -> *mut u16 {
    ENV_BLOCK_W
        .get_or_init(|| {
            let block = rine_types::environment::build_wide_block();
            let boxed = block.into_boxed_slice();
            SyncPtr(Box::into_raw(boxed) as *mut u16)
        })
        .0
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn FreeEnvironmentStringsW(_block: *mut u16) -> WinBool {
    WinBool::TRUE
}
