use std::collections::HashMap;
use std::ffi::{CString, OsStr};
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};

use rine_types::errors::WinBool;
use rine_types::handles::{HandleEntry, handle_table};
use rine_types::os::ProcessInformation;
use rine_types::threading::{ProcessWaitable, STILL_ACTIVE};

use tracing::{debug, warn};

/// Cached command-line strings, built once from `std::env::args`.
pub struct CmdLineCache {
    pub ansi: CString,
    pub wide: Vec<u16>,
}

static CMD_LINE: OnceLock<CmdLineCache> = OnceLock::new();

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

/// Core spawn logic shared by CreateProcessA/W.
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

/// Find the path to the running `rine` binary.
pub fn rine_exe() -> std::path::PathBuf {
    std::env::current_exe().unwrap_or_else(|_| "rine".into())
}

/// Wait for a child to exit and store the result.
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
