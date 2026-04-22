use std::cell::Cell;
use std::collections::HashMap;
use std::ffi::{CString, OsStr};
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};

use rine_types::errors::{ERROR_SUCCESS, WinBool};
use rine_types::handles::{Handle, HandleEntry, handle_table};
use rine_types::os::ProcessInformation;
use rine_types::threading::{ProcessWaitable, STILL_ACTIVE};

use tracing::{debug, warn};

/// Cached command-line strings, built once from `std::env::args`.
pub struct CmdLineCache {
    pub ansi: CString,
    pub wide: Vec<u16>,
}

static CMD_LINE: OnceLock<CmdLineCache> = OnceLock::new();

thread_local! {
    static LAST_ERROR_CODE: Cell<u32> = const { Cell::new(ERROR_SUCCESS) };
}

pub fn cached_cmd_line() -> &'static CmdLineCache {
    CMD_LINE.get_or_init(|| {
        // Reconstruct a single command-line string from argv, quoting args
        // that contain spaces (matches Windows convention loosely).
        let args: Vec<String> = std::env::args().collect();
        let joined = args
            .iter()
            .map(|a| {
                if a.contains(' ') {
                    format!("\"{a}\"")
                } else {
                    a.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        let ansi = CString::new(joined.clone()).unwrap_or_default();
        let mut wide: Vec<u16> = joined.encode_utf16().collect();
        wide.push(0); // null-terminate

        CmdLineCache { ansi, wide }
    })
}

/// Create a child process.
///
/// # Arguments
/// - `exe_path`: the executable path to run (first arg).
/// - `args`: additional command-line arguments (excluding the executable).
/// - `env`: optional environment variables (None to inherit from parent).
/// - `proc_info`: output parameter for process information (can be null).
///
/// # Safety
/// `proc_info` must be null or point to valid memory for a ProcessInformation struct.
/// The caller must ensure that the returned process and thread handles are eventually closed.
///
/// # Returns
/// WinBool::TRUE on success, WinBool::FALSE on failure (e.g. if the executable is not found or fails to launch).
///
/// # Notes
/// This implementation is intentionally incomplete in a few areas:
/// - It does not model Windows `SECURITY_ATTRIBUTES`, handle inheritance,
///   startup-info flags, or creation flags semantics.
/// - It launches through the host `rine` runtime path rather than executing a
///   native Windows image directly.
/// - It does not currently map all failure modes to precise Win32
///   `GetLastError` values.
pub unsafe fn create_process(
    exe_path: &str,
    args: &[String],
    env: Option<HashMap<String, String>>,
    proc_info: *mut ProcessInformation,
) -> WinBool {
    if exe_path.is_empty() {
        warn!("CreateProcess: empty executable path");
        return WinBool::FALSE;
    }

    let rine = rine_exe();
    debug!(rine = %rine.display(), exe = exe_path, ?args, "CreateProcess → spawning child");

    let mut cmd = Command::new(&rine);
    cmd.arg(exe_path);
    if !args.is_empty() {
        cmd.args(args);
    }

    if let Some(ref env) = env {
        cmd.env_clear();
        for (k, v) in env {
            cmd.env(OsStr::new(k), OsStr::new(v));
        }
        // Pass through essential Linux env vars if not already set.
        for key in &["PATH", "HOME", "USER", "LANG", "TERM", "DISPLAY"] {
            if !env.contains_key(*key)
                && let Ok(val) = std::env::var(key)
            {
                cmd.env(key, val);
            }
        }
    }

    let child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            warn!(error = %e, "CreateProcess: spawn failed");
            return WinBool::FALSE;
        }
    };

    let pid = child.id();
    let exit_code = Arc::new(AtomicU32::new(STILL_ACTIVE));
    let completed = Arc::new((Mutex::new(false), Condvar::new()));

    let waitable = ProcessWaitable {
        exit_code: exit_code.clone(),
        completed: completed.clone(),
        pid,
    };

    // Waiter thread to reap the child.
    {
        let exit_code = exit_code.clone();
        let completed = completed.clone();
        std::thread::spawn(move || {
            reap_child(child, exit_code, completed);
        });
    }

    let proc_handle = handle_table().insert(HandleEntry::Process(waitable.clone()));
    let thread_handle = handle_table().insert(HandleEntry::Process(waitable));

    rine_types::dev_notify!(on_handle_created(
        proc_handle.as_raw() as i64,
        "Process",
        &format!("pid={pid}, exe={exe_path}")
    ));
    rine_types::dev_notify!(on_handle_created(
        thread_handle.as_raw() as i64,
        "Process",
        &format!("pid={pid}, primary thread handle")
    ));

    if !proc_info.is_null() {
        unsafe {
            (*proc_info).process = proc_handle.as_raw();
            (*proc_info).thread = thread_handle.as_raw();
            (*proc_info).process_id = pid;
            (*proc_info).thread_id = pid; // no separate thread id
        }
    }

    debug!(pid, proc_handle = ?proc_handle, "child process created");
    WinBool::TRUE
}

/// Gets the process ID of the calling process.
///
/// # Returns
/// The process ID of the calling process.
/// This value is a non-negative integer that uniquely identifies the process within the system.
/// The process ID can be used in various API calls that require a process identifier, such as
/// `OpenProcess` or `WaitForSingleObject`.
///
/// # Note
/// Process IDs can be reused by the system after a process terminates, so they should not
/// be assumed to be unique over time.
pub fn get_current_process_id() -> u32 {
    std::process::id()
}

/// Find the path to the running `rine` binary.
pub fn rine_exe() -> std::path::PathBuf {
    std::env::current_exe().unwrap_or_else(|_| "rine".into())
}

/// Wait for a child to exit and store the result.
///
/// # Arguments
/// * `child` - The child process to wait on. This should be a `std::process::Child` representing the spawned process.
/// * `exit_code` - An `Arc<AtomicU32>` where the exit code of the child process will be stored once it exits.
///   The exit code will be set to the actual exit code of the child process if it terminates normally,
///   or 1 if there is an error while waiting for the child process.
/// * `completed` - An `Arc<(Mutex<bool>, Condvar)>` used to signal when the child process has exited and the exit code has been stored.
///   The mutex protects a boolean flag indicating whether the child process has completed, and the condition variable is used to
///   notify any waiting threads that the child process has exited and the exit code is available.
///
/// # Notes
/// This function will block until the child process exits.
/// It should be run in a separate thread to avoid blocking the main thread of execution.
pub fn reap_child(
    mut child: std::process::Child,
    exit_code: Arc<AtomicU32>,
    completed: Arc<(Mutex<bool>, Condvar)>,
) {
    let code = match child.wait() {
        Ok(s) => s.code().unwrap_or(1) as u32,
        Err(e) => {
            warn!(error = %e, "failed to reap child process");
            1
        }
    };
    exit_code.store(code, Ordering::Release);
    let (lock, cvar) = &*completed;
    let mut done = lock.lock().unwrap();
    *done = true;
    cvar.notify_all();
}

/// Load a DLL into the process's address space.
///
/// # Arguments
/// * `_file_name` - A string slice specifying the name of the DLL to load.
///   If the string does not specify an absolute path, the system searches for the DLL in a specific order of directories.
///   If the function fails to find the DLL, it returns NULL (0).
///
/// # Safety
/// This function is unsafe because it involves raw pointer parameters that must be used correctly by the caller.
/// The caller must ensure that the `library_name` parameter is either null or points to a valid null-terminated
/// ANSI string representing the name of the library to load.
/// Additionally, the caller must handle the returned handle correctly, as it is a raw pointer that may need to
/// be closed with `FreeLibrary` when it is no longer needed. Misuse of the returned handle can lead to resource
/// leaks or other unintended consequences.
///
/// # Returns
/// A handle to the loaded DLL module, or NULL (0) if the function fails to find the DLL.
/// The returned handle can be used in subsequent calls to `GetProcAddress` and `FreeLibrary`.
///
/// # Notes
/// Currently, our implementation always returns NULL (0) as a placeholder since we do not actually load any
/// modules in this stub implementation.
pub unsafe fn load_library(_file_name: &str) -> u32 {
    0
}

/// Retrieve the address of an exported function or variable from a loaded DLL module.
///
/// # Arguments
/// * `_module` - A handle to the loaded DLL module that contains the function or variable.
///   This handle must have been returned by a previous call to `LoadLibraryA` or `LoadLibraryW`.
/// * `_proc_name` - A pointer to a null-terminated ANSI string specifying the name of the function or variable to retrieve.
///   If the string specifies an ordinal value, it must be in the form of `#123` where `123` is the ordinal number of the
///   function or variable.
///
/// # Safety
/// This function is unsafe because it involves raw pointer parameters that must be used correctly by the caller.
/// The caller must ensure that the `module` parameter is a valid handle returned by a previous call to
/// `LoadLibraryA` or `LoadLibraryW`, and that it has not already been freed.
/// The caller must also ensure that the `proc_name` parameter is a valid null-terminated ANSI string representing the
/// name of the function or variable to retrieve, or a valid ordinal string in the form of `#123`.
///
/// # Returns
/// If the function succeeds, the return value is the address of the specified function or variable.
/// If the function fails, the return value is NULL (0). To get extended error information, call `GetLastError`.
///
/// # Notes
/// Currently, our implementation always returns NULL (0) as a placeholder since we do not actually load any
/// modules or export any functions in this stub implementation.
pub unsafe fn get_proc_address() -> u32 {
    0
}

/// Free a loaded DLL module.
///
/// # Arguments
/// * `_module` - A handle to the loaded DLL module to be freed.
///   This handle must have been returned by a previous call to `LoadLibraryA` or `LoadLibraryW`.
///
/// # Safety
/// This function is unsafe because it involves raw pointer parameters that must be used correctly by the caller.
/// The caller must ensure that the `module` parameter is a valid handle returned by a previous call
/// to `LoadLibraryA` or `LoadLibraryW`, and that it has not already been freed.
/// Additionally, the caller must handle the return value correctly, as it indicates whether the operation succeeded or failed.
/// Misuse of the returned value can lead to incorrect assumptions about the state of the loaded module and
/// potential resource leaks if the module is not properly freed when it is no longer needed.
///
/// # Returns
/// If the function succeeds, the return value is `WinBool::TRUE`.
/// If the function fails, the return value is `WinBool::FALSE`.
/// To get extended error information, call `GetLastError`.
///
/// # Notes
/// Currently, our implementation always returns `WinBool::FALSE` as a placeholder,
/// since we do not actually load any modules in this stub implementation.
pub fn free_library(_module: u32) -> WinBool {
    tracing::warn!(
        api = "FreeLibrary",
        dll = "kernel32",
        "FreeLibrary stub called"
    );
    WinBool::FALSE
}

/// Gets the pseudo-handle for the current process, which is currently always -1 in our implementation.
///
/// # Safety
/// This function is unsafe because it returns a raw handle value that must be used correctly by the caller.
/// The caller must ensure that the returned handle is not misused, as it is a sentinel value representing
/// the current process and not a real handle that can be manipulated or closed.
///
/// # Returns
/// The pseudo-handle for the current process, which is currently always -1.
pub fn get_current_process() -> isize {
    -1
}

/// Gets the exit code of a process handle.
///
/// # Arguments
/// * `process_handle` - A handle to the process.
///   This handle must have the `PROCESS_QUERY_INFORMATION` or `PROCESS_QUERY_LIMITED_INFORMATION` access right.
/// * `exit_code` - A pointer to a variable that receives the process's exit code.
///   If the function succeeds, the exit code is stored in the variable pointed to by `exit_code`.
///   If the function fails, the contents of the variable pointed to by `exit_code` are undefined.
///   A process that is still active returns the `STILL_ACTIVE` (259) exit code.
///
/// # Safety
/// The caller must ensure that the `process` handle is valid and has the appropriate access rights to query
/// information about the process.
/// The caller must also ensure that the `exit_code` pointer is valid and points to a writable memory location.
///
/// # Returns
/// If the function succeeds, the return value is nonzero `WinBool::TRUE`.
/// If the function fails, the return value is zero `WinBool::FALSE`.
///
/// # Notes
/// We do not currently handle the error case where the handle does not have the
/// PROCESS_QUERY_INFORMATION or PROCESS_QUERY_LIMITED_INFORMATION access right, and instead just
/// return `WinBool::FALSE` with ERROR_INVALID_HANDLE.
///
/// We also do not currently distinguish all invalid-handle sub-cases with
/// finer-grained Win32 error codes.
pub fn get_exit_code_process(process_handle: Handle) -> Option<u32> {
    if let Some(rine_types::threading::Waitable::Process(p)) =
        handle_table().get_waitable(process_handle)
    {
        return Some(p.exit_code.load(Ordering::Acquire));
    }

    warn!(handle = ?process_handle, "GetExitCodeProcess: invalid handle");
    None
}

/// Get the last error code for the current thread.
///
/// # Returns
/// The thread-local last-error value.
pub fn get_last_error() -> u32 {
    LAST_ERROR_CODE.with(Cell::get)
}

/// Set the last error code for the current thread.
///
/// # Arguments
/// * `error_code` - The Win32 error code to store as the current thread's
///   last-error value.
pub fn set_last_error(error_code: u32) {
    LAST_ERROR_CODE.with(|last_error_code| last_error_code.set(error_code));
}

/// Install a top-level exception filter.
///
/// # Arguments
/// * `_filter` - A pointer to a function that will be called when an unhandled exception occurs in the process.
///   The function should match the `LPTOP_LEVEL_EXCEPTION_FILTER` type, which takes a pointer to an `EXCEPTION_POINTERS`
///   structure and returns a `LONG` value indicating how the exception should be handled.
///
/// # Safety
/// This function is unsafe because it involves raw pointer parameters that must be used correctly by the caller.
///
/// # Returns
/// The SetUnhandledExceptionFilter function returns the address of the previous exception filter established with the function.
/// A NULL return value means that there is no current top-level exception handler.
///
/// # Notes
/// Stub: returns NULL (no previous handler). Exception handling is not
/// yet implemented.
pub fn set_unhandled_exception_filter(_filter: usize, // LPTOP_LEVEL_EXCEPTION_FILTER
) -> usize {
    tracing::warn!(
        api = "SetUnhandledExceptionFilter",
        dll = "kernel32",
        "stub called — exceptions are not yet implemented"
    );

    0 // NULL — no previous handler
}

/// Get a module handle by name. Currently only supports NULL (main executable) and returns 0 as a placeholder.
///
/// When `module_name` is NULL, returns the base address of the main
/// executable. For now we return NULL (0) as a placeholder — the loader
/// will need to provide the real image base once entry-point execution
/// is wired up.
///
/// # Arguments
/// * `module_name` - A pointer to a null-terminated ANSI string specifying the module name.
///
/// # Safety
/// `module_name` must be null or a valid null-terminated ANSI string.
///
/// # Returns
/// If `module_name` is NULL, returns 0 as a placeholder for the main executable.
/// For non-NULL `module_name`, also returns 0 as a placeholder since module lookup is not yet implemented.
pub unsafe fn get_module_handle(_module_name: &str) -> usize {
    0
}

// ---------------------------------------------------------------------------
// CreateProcess helpers
// ---------------------------------------------------------------------------

/// Split a command line respecting double-quote grouping (simplified
/// Windows `CommandLineToArgvW` rules).
pub fn split_cmd_line(s: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for c in s.chars() {
        match c {
            '"' => in_quotes = !in_quotes,
            ' ' | '\t' if !in_quotes => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(c),
        }
    }
    if !current.is_empty() {
        args.push(current);
    }
    args
}

/// Parse a Windows ANSI environment block (null-separated, double-null
/// terminated) into key→value pairs.
///
/// # Arguments
/// * `ptr` - Pointer to the start of the environment block.
///   Must be null or point to a valid block of memory containing
///   null-separated "KEY=VALUE" strings, terminated by an additional null
///   character (i.e. two consecutive nulls at the end).
///
/// # Safety
/// * `ptr` must be null or point to a valid environment block as described above.
/// * The caller must ensure that the memory pointed to by `ptr` remains valid for
///   the duration of the call and that it is properly null-terminated to avoid reading out of bounds.
pub unsafe fn parse_env_block(ptr: *const u8) -> HashMap<String, String> {
    let mut env = HashMap::new();
    if ptr.is_null() {
        return env;
    }
    let mut offset = 0usize;
    loop {
        let start = offset;

        while unsafe { *ptr.add(offset) } != 0 {
            offset += 1;
        }

        if offset == start {
            break;
        }

        let bytes = unsafe { std::slice::from_raw_parts(ptr.add(start), offset - start) };
        if let Ok(s) = std::str::from_utf8(bytes)
            && let Some(eq) = s.find('=')
        {
            let (k, v) = s.split_at(eq);
            env.insert(k.to_string(), v[1..].to_string());
        }
        offset += 1;
    }
    env
}

/// Parse a wide (UTF-16LE) Windows environment block.
///
/// # Arguments
/// * `ptr` - Pointer to the start of the environment block.
///   Must be null or point to a valid block of memory containing
///   null-separated "KEY=VALUE" strings, terminated by an additional null
///   character (i.e. two consecutive nulls at the end).
///
/// # Safety
/// * `ptr` must be null or point to a valid environment block as described above.
/// * The caller must ensure that the memory pointed to by `ptr` remains valid for
///   the duration of the call and that it is properly null-terminated to avoid reading out of bounds.
pub unsafe fn parse_env_block_wide(ptr: *const u16) -> HashMap<String, String> {
    let mut env = HashMap::new();
    if ptr.is_null() {
        return env;
    }
    let mut offset = 0usize;
    loop {
        let start = offset;
        while unsafe { *ptr.add(offset) } != 0 {
            offset += 1;
        }
        if offset == start {
            break;
        }
        let slice = unsafe { std::slice::from_raw_parts(ptr.add(start), offset - start) };
        let s = String::from_utf16_lossy(slice);
        if let Some(eq) = s.find('=') {
            let (k, v) = s.split_at(eq);
            env.insert(k.to_string(), v[1..].to_string());
        }
        offset += 1;
    }
    env
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── split_cmd_line ───────────────────────────────────────────

    #[test]
    fn split_simple() {
        assert_eq!(split_cmd_line("foo bar baz"), vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn split_empty() {
        assert!(split_cmd_line("").is_empty());
    }

    #[test]
    fn split_quoted_spaces() {
        assert_eq!(
            split_cmd_line(r#""C:\Program Files\app.exe" --flag"#),
            vec![r"C:\Program Files\app.exe", "--flag"]
        );
    }

    #[test]
    fn split_multiple_spaces() {
        assert_eq!(split_cmd_line("a   b\tc"), vec!["a", "b", "c"]);
    }

    // ── parse_env_block ─────────────────────────────────────────

    #[test]
    fn env_block_null() {
        let env = unsafe { parse_env_block(std::ptr::null()) };
        assert!(env.is_empty());
    }

    #[test]
    fn env_block_single() {
        let block = b"FOO=bar\0\0";
        let env = unsafe { parse_env_block(block.as_ptr()) };
        assert_eq!(env.get("FOO").unwrap(), "bar");
        assert_eq!(env.len(), 1);
    }

    #[test]
    fn env_block_multiple() {
        let block = b"A=1\0B=2\0C=hello\0\0";
        let env = unsafe { parse_env_block(block.as_ptr()) };
        assert_eq!(env.len(), 3);
        assert_eq!(env["A"], "1");
        assert_eq!(env["B"], "2");
        assert_eq!(env["C"], "hello");
    }

    // ── parse_env_block_wide ────────────────────────────────────

    #[test]
    fn env_block_wide_null() {
        let env = unsafe { parse_env_block_wide(std::ptr::null()) };
        assert!(env.is_empty());
    }

    #[test]
    fn env_block_wide_single() {
        let block: Vec<u16> = "KEY=val\0\0".encode_utf16().collect();
        let env = unsafe { parse_env_block_wide(block.as_ptr()) };
        assert_eq!(env.get("KEY").unwrap(), "val");
    }
}
