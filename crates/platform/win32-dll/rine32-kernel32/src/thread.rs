use std::time::Duration;

use rine_common_kernel32::thread as common_thread;
use rine_types::errors::WinBool;
use rine_types::handles::Handle;
use rine_types::threading;

#[repr(C)]
struct UserDesc {
    entry_number: u32,
    base_addr: u32,
    limit: u32,
    flags: u32,
}

unsafe fn setup_child_teb() {
    const TEB_SIZE: usize = 0x1000;
    const PEB_SIZE: usize = 0x1000;
    const TEB_STACK_BASE: usize = 0x04;
    const TEB_STACK_LIMIT: usize = 0x08;
    const TEB_SELF: usize = 0x18;
    const TEB_PEB: usize = 0x30;
    const WINDOWS_TEB_STACK_BASE_HEADROOM_BYTES: u32 = 0x100000;
    const PAGE_ALIGNMENT_MASK: u32 = 0xFFF;
    const WINDOWS_TEB_STACK_RESERVE_BYTES: u32 = 0x200000;
    const USER_DESC_SEG_32BIT: u32 = 1 << 0;
    const USER_DESC_LIMIT_IN_PAGES: u32 = 1 << 4;
    const USER_DESC_USEABLE: u32 = 1 << 6;

    let teb_layout = std::alloc::Layout::from_size_align(TEB_SIZE, 16).unwrap();
    let peb_layout = std::alloc::Layout::from_size_align(PEB_SIZE, 16).unwrap();

    let teb = unsafe { std::alloc::alloc_zeroed(teb_layout) };
    assert!(!teb.is_null(), "failed to allocate child TEB");

    let peb = unsafe { std::alloc::alloc_zeroed(peb_layout) };
    assert!(!peb.is_null(), "failed to allocate child PEB");

    let stack_base: u32;
    unsafe {
        core::arch::asm!("mov {0:e}, esp", out(reg) stack_base);
    }
    let stack_base =
        stack_base.saturating_add(WINDOWS_TEB_STACK_BASE_HEADROOM_BYTES) & !PAGE_ALIGNMENT_MASK;
    let stack_limit = stack_base.saturating_sub(WINDOWS_TEB_STACK_RESERVE_BYTES);

    unsafe {
        std::ptr::write(teb.add(TEB_STACK_BASE) as *mut u32, stack_base);
        std::ptr::write(teb.add(TEB_STACK_LIMIT) as *mut u32, stack_limit);
        std::ptr::write(teb.add(TEB_SELF) as *mut u32, teb as u32);
        std::ptr::write(teb.add(TEB_PEB) as *mut u32, peb as u32);
    }

    let mut user_desc = UserDesc {
        entry_number: u32::MAX,
        base_addr: teb as u32,
        limit: 0xFFFFF,
        flags: USER_DESC_SEG_32BIT | USER_DESC_LIMIT_IN_PAGES | USER_DESC_USEABLE,
    };

    let ret = unsafe {
        // x86 Linux installs thread-local segment descriptors via set_thread_area.
        // For Windows PE32 emulation, we bind FS to our fake TEB descriptor.
        libc::syscall(
            libc::SYS_set_thread_area,
            &mut user_desc as *mut UserDesc as *mut libc::c_void,
        )
    };
    assert!(ret == 0, "set_thread_area failed in child thread");

    let selector = ((user_desc.entry_number << 3) | 0x3) as u16;
    unsafe {
        // Load the descriptor selector into FS so PE32 TEB reads via fs: work.
        core::arch::asm!("mov fs, ax", in("ax") selector, options(nostack, preserves_flags));
    }
}

/// Windows thread start routine: `DWORD WINAPI ThreadProc(LPVOID)`.
type ThreadStartRoutine = unsafe extern "stdcall" fn(usize) -> u32;

/// Create and start a new thread, returning a handle and thread ID.
///
/// # Arguments
/// * `_security_attrs`: Ignored, must be zero.
/// * `_stack_size`: Ignored, must be zero (default stack size is used).
/// * `start_address`: Function pointer to the thread entry point, which must follow the windows x86 stdcall
///   calling convention and return a `DWORD` exit code.
/// * `parameter`: Argument passed to the thread entry point.
/// * `creation_flags`: Creation flags, currently ignored except for `CREATE_SUSPENDED` which is not supported but
///   will prevent a warning.
/// * `thread_id_out`: Optional pointer to receive the new thread's ID.
///
/// # Safety
/// The caller must ensure that `start_address` is a valid function pointer with the correct signature and calling convention.
/// The caller must not pass invalid or non-zero values for `_security_attrs` and `_stack_size`.
/// The caller must ensure that `thread_id_out` is either null or a valid pointer to a `u32` variable.
/// The caller is responsible for closing the returned thread handle when it is no longer needed.
///
/// # Returns
/// A handle to the new thread, or `INVALID_HANDLE_VALUE` on failure.
/// The caller can use this handle with other synchronization functions and must close it with `CloseHandle` when done.
///
/// # Notes
/// Missing implementation features:
/// - `CREATE_SUSPENDED` is not supported; the thread starts immediately.
/// - `_security_attrs` and `_stack_size` semantics are ignored.
/// - Most creation flags beyond basic launch are not implemented.
/// - No Win32-accurate `GetLastError` mapping is provided for failure paths.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn CreateThread(
    _security_attrs: usize,
    _stack_size: usize,
    start_address: usize,
    parameter: usize,
    creation_flags: u32,
    thread_id_out: *mut u32,
) -> Handle {
    let thread_id_out = unsafe { thread_id_out.as_mut() };
    common_thread::create_thread(
        start_address,
        parameter,
        creation_flags,
        thread_id_out,
        |start_address, parameter| {
            unsafe {
                setup_child_teb();
            }

            let start_fn: ThreadStartRoutine = unsafe { core::mem::transmute(start_address) };
            unsafe { start_fn(parameter) }
        },
    )
}

// Allocate a TLS index.
///
/// # Safety
/// The caller is responsible for calling `TlsFree` to release the index when it is no longer needed.
///
/// # Returns
/// A TLS index, or `TLS_OUT_OF_INDEXES` (0xFFFFFFFF) on failure.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn TlsAlloc() -> u32 {
    common_thread::tls_alloc()
}

/// Free a TLS index.
///
/// # Arguments
/// * `tls_index`: The TLS index to free, which must have been previously allocated by `TlsAlloc` and not already freed.
///
/// # Safety
/// The caller must ensure that `tls_index` is a valid index previously allocated by `TlsAlloc` and not already freed.
/// The caller is responsible for ensuring that no threads are currently using the TLS index before freeing it.
///
/// # Returns
/// `WinBool::TRUE` on success, `WinBool::FALSE` on failure (e.g., invalid index).
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn TlsFree(tls_index: u32) -> WinBool {
    common_thread::tls_free(tls_index)
}

/// Get the current thread's value for a TLS slot.
///
/// # Arguments
/// * `tls_index`: The TLS index to query, which must have been previously allocated by `TlsAlloc` and not already freed.
///
/// # Safety
/// The caller must ensure that `tls_index` is a valid index previously allocated by `TlsAlloc` and not already freed.
/// The caller is responsible for ensuring that the returned value is interpreted correctly based on how it was set with `TlsSetValue`.
///
/// # Returns
/// The value associated with the TLS index for the current thread, or 0 if the index is invalid or has not been set.
///
/// # Note
/// A 0 can also be a valid value set by `TlsSetValue`, so the caller should use `GetLastError` to distinguish between an
/// error and a valid 0 value if needed.
/// Currently, we do not set `GetLastError` to provide more information.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn TlsGetValue(tls_index: u32) -> usize {
    common_thread::tls_get_value(tls_index)
}

/// Set the current thread's value for a slot.
///
/// # Arguments
/// * `tls_index`: The TLS index to set, which must have been previously allocated by `TlsAlloc` and not already freed.
/// * `value`: The value to associate with the TLS index for the current thread. This can be any `usize` value, including 0.
///
/// # Safety
/// The caller must ensure that `tls_index` is a valid index previously allocated by `TlsAlloc` and not already freed.
/// The caller is responsible for ensuring that the value is interpreted correctly based on how it will be used when
/// retrieved with `TlsGetValue`.
/// The caller should also ensure that any necessary synchronization is performed when setting and getting TLS values
/// across threads, as appropriate for the application's logic.
///
/// # Returns
/// `WinBool::TRUE` on success, `WinBool::FALSE` on failure (e.g., invalid index).
///
/// # Notes
/// Missing implementation features:
/// - No Win32-accurate `GetLastError` mapping is provided for invalid TLS index failures.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn TlsSetValue(tls_index: u32, value: usize) -> WinBool {
    common_thread::tls_set_value(tls_index, value)
}

/// Cause the current thread to sleep for the specified duration.
///
/// # Arguments
/// * `duration`: The duration to sleep for.
///
/// # Safety
/// The caller is responsible for ensuring that sleeping is appropriate in the current context
/// (e.g., not holding locks that would cause deadlocks).
/// The caller should also be aware that sleeping does not guarantee precise timing and may be affected by system scheduling and load.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn Sleep(milliseconds: u32) {
    let duration = Duration::from_millis(milliseconds as u64);
    common_thread::sleep(duration);
}

/// Get a pseudo-handle for the calling thread.
///
/// # Safety
/// This is not a real handle and cannot be used with all handle functions, but it can be used with certain functions that
/// specifically support it (e.g., `GetExitCodeThread`).
/// The caller should not attempt to close this handle or use it in contexts that require a real handle.
///
/// # Returns
/// A pseudo-handle value that represents the current thread.
/// This value is not a real handle and should only be used with functions that explicitly support it.
///
/// # Notes
/// Missing implementation features:
/// - The pseudo-handle is not currently mapped to a concrete thread entry in
///   the internal handle table.
/// - APIs expecting a queryable thread handle may still reject this pseudo-
///   handle instead of treating it as `GetCurrentThread()`.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetCurrentThread() -> Handle {
    common_thread::current_thread()
}

/// Get the current thread's ID.
///
/// # Safety
/// This is a unique identifier for the thread that can be used with certain APIs and for debugging purposes.
/// It is not a handle and cannot be used with handle-based APIs.
/// The thread ID is assigned by our implementation and is not guaranteed to match any OS-level thread ID,
/// but it is unique within the process and can be used to identify threads in logs and debugging tools.
///
/// # Returns
/// The current thread's ID as a `u32`.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetCurrentThreadId() -> u32 {
    common_thread::current_thread_id()
}

/// Get the exit code of a thread.
///
/// # Arguments
/// * `thread_handle`: A handle to the thread, which can be a real thread handle or the pseudo-handle returned by `current_thread()`.
/// * `exit_code_out`: Optional pointer to receive the thread's exit code.
///   If the thread is still active, this will be set to `STILL_ACTIVE` (259).
///
/// # Safety
/// The caller must ensure that `thread_handle` is a valid thread handle or the pseudo-handle for the current thread.
/// The caller must also ensure that `exit_code_out` is either null or a valid pointer to a `u32` variable.
/// The caller is responsible for interpreting the returned exit code correctly, especially if the thread is still
/// active (in which case it will be set to `STILL_ACTIVE`).
///
/// # Returns
/// `WinBool::TRUE` on success, with the thread's exit code written to `exit_code_out` if it is not null.
/// `WinBool::FALSE` on failure (e.g., invalid handle).
///
/// # Notes
/// Missing implementation features:
/// - No Win32-accurate `GetLastError` mapping is provided for invalid-handle
///   or null-output-pointer failures.
/// - No explicit access-right checks are enforced against per-handle granted
///   permissions.
/// - Pseudo-handle semantics (`GetCurrentThread`) are not normalized here.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetExitCodeThread(
    thread_handle: Handle,
    exit_code_out: *mut u32,
) -> WinBool {
    let exit_code_out = unsafe { exit_code_out.as_mut() };
    common_thread::get_exit_code_thread(thread_handle, exit_code_out)
}

/// Block the current thread until the specified handle is signalled or the timeout elapses.
///
/// # Arguments
/// * `handle`: A handle to wait on, which can be a thread handle, process handle, or synchronization object handle.
/// * `timeout_ms`: The timeout in milliseconds to wait, or `INFINITE` (0xFFFFFFFF) to wait indefinitely.
///
/// # Safety
/// The caller should be aware that the actual wait time may be affected by system scheduling and load, and that the
/// function may return before the timeout elapses if the handle is signalled.
///
/// # Returns
/// `WAIT_OBJECT_0` if the handle was signalled, `WAIT_TIMEOUT` if the timeout elapsed, or `WAIT_FAILED` on error.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn WaitForSingleObject(handle: Handle, timeout_ms: u32) -> u32 {
    let duration = Duration::from_millis(timeout_ms as u64);
    common_thread::wait_for_single_object(handle, duration)
}

/// Block the current thread until one or all of the specified handles are signalled, or the timeout elapses.
///
/// # Arguments
/// * `count`: The number of handles in the array pointed to by `handles_ptr`.
/// * `handles_ptr`: Pointer to an array of handles to wait on, which can be thread handles, process handles,
///   or synchronization object handles.
/// * `wait_all`: If `WinBool::TRUE`, the function returns when all handles are signalled;
///   if `WinBool::FALSE`, it returns when any one handle is signalled.
/// * `timeout_ms`: The timeout in milliseconds to wait, or `INFINITE` (0xFFFFFFFF) to wait indefinitely.
///
/// # Safety
/// The caller must ensure that `handles_ptr` points to a valid array of `count` handles, and that each handle
/// is valid and of a type that can be waited on.
/// The caller should also be aware that the actual wait time may be affected by system scheduling and load,
/// and that the function may return before the timeout elapses if the specified condition is met (e.g., a handle is signalled).
///
/// # Returns
/// If `wait_all` is `WinBool::FALSE`, returns `WAIT_OBJECT_0 + i` if the handle at index `i` is signalled,
/// `WAIT_TIMEOUT` if the timeout elapsed, or `WAIT_FAILED` on error.
/// If `wait_all` is `WinBool::TRUE`, returns `WAIT_OBJECT_0` if all handles are signalled,
/// `WAIT_TIMEOUT` if the timeout elapsed, or `WAIT_FAILED` on error.
#[rine_dlls::implemented]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn WaitForMultipleObjects(
    count: u32,
    handles_ptr: *const Handle,
    wait_all: WinBool,
    timeout_ms: u32,
) -> u32 {
    if handles_ptr.is_null() {
        return threading::WaitStatus::WAIT_FAILED.0;
    }

    let handles = unsafe { std::slice::from_raw_parts(handles_ptr, count as usize) };
    let duration = Duration::from_millis(timeout_ms as u64);
    common_thread::wait_for_multiple_objects(handles, wait_all.is_true(), duration)
}
