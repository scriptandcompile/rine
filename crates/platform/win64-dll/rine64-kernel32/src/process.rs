//! kernel32 process functions: ExitProcess, CreateProcessA/W,
//! GetCommandLineA/W, GetModuleHandleA/W, GetCurrentProcessId,
//! GetExitCodeProcess.

use std::collections::HashMap;
use std::ffi::{CString, OsStr};
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};

use rine_types::errors::WinBool;
use rine_types::handles::{Handle, HandleEntry, handle_table};
use rine_types::os::{ProcessInformation, StartupInfoA, StartupInfoW};
use rine_types::strings::{read_cstr, read_wstr};
use rine_types::threading::{ProcessWaitable, STILL_ACTIVE};
use tracing::{debug, warn};

/// Cached command-line strings, built once from `std::env::args`.
struct CmdLineCache {
    ansi: CString,
    wide: Vec<u16>,
}

static CMD_LINE: OnceLock<CmdLineCache> = OnceLock::new();

fn cached_cmd_line() -> &'static CmdLineCache {
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

/// ExitProcess — terminate the current process.
///
/// # Safety
/// Does not return.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn ExitProcess(exit_code: u32) -> ! {
    std::process::exit(exit_code as i32);
}

/// GetCommandLineA — return a pointer to the ANSI command-line string.
///
/// # Safety
/// The returned pointer is valid for the lifetime of the process.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetCommandLineA() -> *const u8 {
    cached_cmd_line().ansi.as_ptr().cast()
}

/// GetCommandLineW — return a pointer to the wide command-line string.
///
/// # Safety
/// The returned pointer is valid for the lifetime of the process.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetCommandLineW() -> *const u16 {
    cached_cmd_line().wide.as_ptr()
}

/// GetModuleHandleA — retrieve the base address of a loaded module.
///
/// When `module_name` is NULL, returns the base address of the main
/// executable. For now we return NULL (0) as a placeholder — the loader
/// will need to provide the real image base once entry-point execution
/// is wired up.
///
/// # Safety
/// `module_name` must be null or a valid null-terminated ANSI string.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetModuleHandleA(module_name: *const u8) -> usize {
    if module_name.is_null() {
        // TODO: return the actual image base once the loader exposes it.
        tracing::debug!("GetModuleHandleA(NULL) — returning 0 (placeholder)");
        return 0;
    }

    let name = unsafe { std::ffi::CStr::from_ptr(module_name.cast()) };
    tracing::warn!(
        ?name,
        "GetModuleHandleA: non-NULL module_name not yet supported"
    );
    0
}

/// GetModuleHandleW — wide variant of `GetModuleHandleA`.
///
/// # Safety
/// `module_name` must be null or a valid null-terminated UTF-16LE string.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetModuleHandleW(module_name: *const u16) -> usize {
    if module_name.is_null() {
        tracing::debug!("GetModuleHandleW(NULL) — returning 0 (placeholder)");
        return 0;
    }

    // Decode for logging only.
    let mut len = 0;
    unsafe {
        while *module_name.add(len) != 0 {
            len += 1;
        }
    }
    let wide_slice = unsafe { core::slice::from_raw_parts(module_name, len) };
    let name = String::from_utf16_lossy(wide_slice);
    tracing::warn!(
        name,
        "GetModuleHandleW: non-NULL module_name not yet supported"
    );
    0
}

/// GetLastError — return the last-error code for the calling thread.
///
/// Stub: always returns 0 (ERROR_SUCCESS). A real per-thread last-error
/// store will be added with the threading subsystem.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn GetLastError() -> u32 {
    0
}

/// SetUnhandledExceptionFilter — install a top-level exception filter.
///
/// Stub: returns NULL (no previous handler). Exception handling is not
/// yet implemented.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn SetUnhandledExceptionFilter(
    _filter: usize, // LPTOP_LEVEL_EXCEPTION_FILTER
) -> usize {
    0 // NULL — no previous handler
}

// ---------------------------------------------------------------------------
// CreateProcess helpers
// ---------------------------------------------------------------------------

/// Split a command line respecting double-quote grouping (simplified
/// Windows `CommandLineToArgvW` rules).
fn split_cmd_line(s: &str) -> Vec<String> {
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
fn parse_env_block(ptr: *const u8) -> HashMap<String, String> {
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
fn parse_env_block_wide(ptr: *const u16) -> HashMap<String, String> {
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

/// Find the path to the running `rine` binary.
fn rine_exe() -> std::path::PathBuf {
    std::env::current_exe().unwrap_or_else(|_| "rine".into())
}

/// Core spawn logic shared by CreateProcessA/W.
fn do_create_process(
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

/// Wait for a child to exit and store the result.
fn reap_child(
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

// ---------------------------------------------------------------------------
// CreateProcessA / CreateProcessW
// ---------------------------------------------------------------------------

/// CreateProcessA — create a child process (ANSI).
///
/// # Safety
/// All pointer parameters must be null or point to valid memory of the
/// expected layout.
#[allow(non_snake_case, clippy::missing_safety_doc, clippy::too_many_arguments)]
pub unsafe extern "win64" fn CreateProcessA(
    application_name: *const u8,           // rcx
    command_line: *mut u8,                 // rdx
    _process_attrs: usize,                 // r8
    _thread_attrs: usize,                  // r9
    _inherit_handles: i32,                 // [rsp+0x28]
    _creation_flags: u32,                  // [rsp+0x30]
    environment: *const u8,                // [rsp+0x38]
    _current_directory: *const u8,         // [rsp+0x40]
    _startup_info: *const StartupInfoA,    // [rsp+0x48]
    process_info: *mut ProcessInformation, // [rsp+0x50]
) -> WinBool {
    let app = unsafe { read_cstr(application_name) }.unwrap_or_default();
    let cmd = unsafe { read_cstr(command_line.cast_const()) }.unwrap_or_default();

    let (exe, args) = if !app.is_empty() {
        (app, split_cmd_line(&cmd))
    } else {
        let tokens = split_cmd_line(&cmd);
        if tokens.is_empty() {
            warn!("CreateProcessA: no executable specified");
            return WinBool::FALSE;
        }
        (tokens[0].clone(), tokens[1..].to_vec())
    };

    let env = if environment.is_null() {
        None
    } else {
        Some(parse_env_block(environment))
    };

    do_create_process(&exe, &args, env, process_info)
}

/// CreateProcessW — create a child process (wide).
///
/// # Safety
/// All pointer parameters must be null or point to valid memory of the
/// expected layout.
#[allow(non_snake_case, clippy::missing_safety_doc, clippy::too_many_arguments)]
pub unsafe extern "win64" fn CreateProcessW(
    application_name: *const u16,          // rcx
    command_line: *mut u16,                // rdx
    _process_attrs: usize,                 // r8
    _thread_attrs: usize,                  // r9
    _inherit_handles: i32,                 // [rsp+0x28]
    _creation_flags: u32,                  // [rsp+0x30]
    environment: *const u16,               // [rsp+0x38]
    _current_directory: *const u16,        // [rsp+0x40]
    _startup_info: *const StartupInfoW,    // [rsp+0x48]
    process_info: *mut ProcessInformation, // [rsp+0x50]
) -> WinBool {
    let app = unsafe { read_wstr(application_name) }.unwrap_or_default();
    let cmd = unsafe { read_wstr(command_line.cast_const()) }.unwrap_or_default();

    let (exe, args) = if !app.is_empty() {
        (app, split_cmd_line(&cmd))
    } else {
        let tokens = split_cmd_line(&cmd);
        if tokens.is_empty() {
            warn!("CreateProcessW: no executable specified");
            return WinBool::FALSE;
        }
        (tokens[0].clone(), tokens[1..].to_vec())
    };

    let env = if environment.is_null() {
        None
    } else {
        Some(parse_env_block_wide(environment))
    };

    do_create_process(&exe, &args, env, process_info)
}

// ---------------------------------------------------------------------------
// Process info queries
// ---------------------------------------------------------------------------

/// GetCurrentProcessId — return the process ID of the calling process.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn GetCurrentProcessId() -> u32 {
    std::process::id()
}

/// GetCurrentProcess — return a pseudo-handle for the current process.
///
/// Windows defines this as `(HANDLE)-1`; it is not a real handle but a
/// sentinel that APIs accept to mean "this process".
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn GetCurrentProcess() -> isize {
    -1 // pseudo-handle
}

/// GetExitCodeProcess — read the exit code of a process handle.
///
/// Returns `STILL_ACTIVE` (259) if the process has not yet terminated.
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "win64" fn GetExitCodeProcess(process: isize, exit_code: *mut u32) -> WinBool {
    if exit_code.is_null() {
        return WinBool::FALSE;
    }

    let h = Handle::from_raw(process);
    if let Some(rine_types::threading::Waitable::Process(p)) = handle_table().get_waitable(h) {
        unsafe { *exit_code = p.exit_code.load(Ordering::Acquire) };
        return WinBool::TRUE;
    }

    warn!(handle = ?h, "GetExitCodeProcess: invalid handle");
    WinBool::FALSE
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
        let env = parse_env_block(std::ptr::null());
        assert!(env.is_empty());
    }

    #[test]
    fn env_block_single() {
        let block = b"FOO=bar\0\0";
        let env = parse_env_block(block.as_ptr());
        assert_eq!(env.get("FOO").unwrap(), "bar");
        assert_eq!(env.len(), 1);
    }

    #[test]
    fn env_block_multiple() {
        let block = b"A=1\0B=2\0C=hello\0\0";
        let env = parse_env_block(block.as_ptr());
        assert_eq!(env.len(), 3);
        assert_eq!(env["A"], "1");
        assert_eq!(env["B"], "2");
        assert_eq!(env["C"], "hello");
    }

    // ── parse_env_block_wide ────────────────────────────────────

    #[test]
    fn env_block_wide_null() {
        let env = parse_env_block_wide(std::ptr::null());
        assert!(env.is_empty());
    }

    #[test]
    fn env_block_wide_single() {
        let block: Vec<u16> = "KEY=val\0\0".encode_utf16().collect();
        let env = parse_env_block_wide(block.as_ptr());
        assert_eq!(env.get("KEY").unwrap(), "val");
    }

    // ── GetCurrentProcessId / GetCurrentProcess ─────────────────

    #[test]
    fn current_process_id_nonzero() {
        let pid = unsafe { GetCurrentProcessId() };
        assert!(pid > 0);
    }

    #[test]
    fn current_process_pseudo_handle() {
        let h = unsafe { GetCurrentProcess() };
        assert_eq!(h, -1);
    }

    // ── GetExitCodeProcess with null pointer ─────────────────────

    #[test]
    fn exit_code_null_ptr_returns_false() {
        let result = unsafe { GetExitCodeProcess(0x9999, std::ptr::null_mut()) };
        assert_eq!(result, WinBool::FALSE);
    }

    #[test]
    fn exit_code_invalid_handle_returns_false() {
        let mut code: u32 = 0;
        let result = unsafe { GetExitCodeProcess(0x9999, &mut code) };
        assert_eq!(result, WinBool::FALSE);
    }
}
