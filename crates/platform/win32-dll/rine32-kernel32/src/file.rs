use rine_dlls::win32_stub;
use rine_types::errors::WinBool;
use rine_types::handles::{Handle, HandleEntry, handle_table, handle_to_fd};

win32_stub!(CreateFileA, "kernel32");
win32_stub!(CreateFileW, "kernel32");
win32_stub!(ReadFile, "kernel32");

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn WriteFile(
    file: isize,
    buffer: *const u8,
    bytes_to_write: u32,
    bytes_written: *mut u32,
    _overlapped: *mut core::ffi::c_void,
) -> WinBool {
    let handle = Handle::from_raw(file);
    let Some(fd) = handle_to_fd(handle) else {
        return WinBool::FALSE;
    };

    let written = unsafe { libc::write(fd, buffer.cast(), bytes_to_write as usize) };
    if written < 0 {
        return WinBool::FALSE;
    }

    if !bytes_written.is_null() {
        unsafe { *bytes_written = written as u32 };
    }
    WinBool::TRUE
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn CloseHandle(object: isize) -> WinBool {
    let handle = Handle::from_raw(object);
    match handle_table().remove(handle) {
        Some(HandleEntry::Thread(_)) => WinBool::TRUE,
        Some(HandleEntry::Event(_)) => WinBool::TRUE,
        Some(HandleEntry::Process(_)) => WinBool::TRUE,
        Some(HandleEntry::Mutex(_)) => WinBool::TRUE,
        Some(HandleEntry::Semaphore(_)) => WinBool::TRUE,
        Some(HandleEntry::Heap(_)) => WinBool::TRUE,
        Some(HandleEntry::RegistryKey(_)) => WinBool::TRUE,
        Some(HandleEntry::FindData(_)) => WinBool::TRUE,
        Some(HandleEntry::File(fd)) => {
            if fd <= 2 {
                WinBool::TRUE
            } else {
                unsafe { libc::close(fd) };
                WinBool::TRUE
            }
        }
        Some(HandleEntry::Window(_)) => WinBool::FALSE,
        None => WinBool::FALSE,
    }
}
