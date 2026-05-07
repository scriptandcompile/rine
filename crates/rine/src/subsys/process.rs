//! Process management — `CreateProcess` support.
//!
//! `CreateProcessA/W` is implemented by spawning a child `rine` process
//! (i.e. re-invoking the loader) with the target .exe as its argument.
//! The child's exit code is tracked via a waiter thread so the parent
//! can call `WaitForSingleObject` / `GetExitCodeProcess` on the
//! returned process handle.

use std::collections::HashMap;
use std::ffi::OsStr;
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Condvar, Mutex};

use rine_types::handles::{HANDLE, HandleEntry, handle_table};
use rine_types::threading::{ProcessWaitable, STILL_ACTIVE};
use tracing::{debug, warn};

/// Find the path to the current `rine` binary so child processes can be
/// launched through it.
#[allow(dead_code)]
fn rine_exe() -> std::path::PathBuf {
    std::env::current_exe().unwrap_or_else(|_| "rine".into())
}

/// Parse a Windows-style command line into (application, arguments).
///
/// If `app_name` is non-empty it is the executable path; otherwise the
/// first token of `cmd_line` is used.  Returns `(exe_path, args)`.
#[allow(dead_code)]
pub fn parse_command_line(app_name: &str, cmd_line: &str) -> (String, Vec<String>) {
    if !app_name.is_empty() {
        // app_name is the exe; cmd_line is arguments only.
        let args = split_cmd_line(cmd_line);
        return (app_name.to_string(), args);
    }

    // No app_name — first token of cmd_line is the exe.
    let tokens = split_cmd_line(cmd_line);
    if tokens.is_empty() {
        return (String::new(), Vec::new());
    }
    let exe = tokens[0].clone();
    let args = tokens[1..].to_vec();
    (exe, args)
}

/// Split a command line respecting double-quote grouping (simplified
/// Windows `CommandLineToArgvW` rules).
#[allow(dead_code)]
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

/// Parse a Windows environment block (null-separated, double-null terminated)
/// into a `HashMap`.  The block is a sequence of `VAR=value\0` pairs.
#[allow(dead_code)]
pub fn parse_env_block(ptr: *const u8) -> HashMap<String, String> {
    let mut env = HashMap::new();
    if ptr.is_null() {
        return env;
    }

    let mut offset = 0usize;
    loop {
        // Read until we hit a NUL byte.
        let start = offset;
        while unsafe { *ptr.add(offset) } != 0 {
            offset += 1;
        }
        if offset == start {
            // Double-NUL → end of block.
            break;
        }
        let bytes = unsafe { std::slice::from_raw_parts(ptr.add(start), offset - start) };
        if let Ok(s) = std::str::from_utf8(bytes)
            && let Some(eq) = s.find('=')
        {
            let (k, v) = s.split_at(eq);
            env.insert(k.to_string(), v[1..].to_string());
        }
        offset += 1; // skip NUL
    }
    env
}

/// Parse a wide (UTF-16LE) Windows environment block.
#[allow(dead_code)]
pub fn parse_env_block_wide(ptr: *const u16) -> HashMap<String, String> {
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

/// Spawn a child process running `rine <exe_path> [args...]`.
///
/// Returns `(process_handle, thread_handle, pid, tid)`.
/// The thread handle currently mirrors the process handle.
#[allow(dead_code)]
pub fn spawn_child(
    exe_path: &str,
    args: &[String],
    env: Option<&HashMap<String, String>>,
) -> Option<(HANDLE, HANDLE, u32)> {
    let rine = rine_exe();
    debug!(rine = %rine.display(), exe = exe_path, "CreateProcess → spawning child rine");

    let mut cmd = Command::new(&rine);
    cmd.arg(exe_path);
    // Separator: everything after `--` is forwarded to the PE.
    if !args.is_empty() {
        cmd.args(args);
    }

    // If the PE supplied an environment block, use it.
    if let Some(env) = env {
        cmd.env_clear();
        for (k, v) in env {
            cmd.env(OsStr::new(k), OsStr::new(v));
        }
        // Always pass through essential Linux env vars.
        for pass_through in &["PATH", "HOME", "USER", "LANG", "TERM", "DISPLAY"] {
            if !env.contains_key(*pass_through)
                && let Ok(val) = std::env::var(pass_through)
            {
                cmd.env(pass_through, val);
            }
        }
    }

    let child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            warn!(error = %e, "CreateProcess: failed to spawn child");
            return None;
        }
    };

    let pid = child.id();
    let exit_code = Arc::new(AtomicU32::new(STILL_ACTIVE));
    let completed = Arc::new((Mutex::new(false), Condvar::new()));

    let proc_waitable = ProcessWaitable {
        exit_code: exit_code.clone(),
        completed: completed.clone(),
        pid,
    };

    // Spawn a waiter thread that reaps the child and signals completion.
    {
        let exit_code = exit_code.clone();
        let completed = completed.clone();
        std::thread::spawn(move || {
            wait_for_child(child, exit_code, completed);
        });
    }

    let proc_handle = handle_table().insert(HandleEntry::Process(proc_waitable.clone()));
    // Windows returns a separate thread handle; we re-use the same waitable.
    let thread_handle = handle_table().insert(HandleEntry::Process(proc_waitable));

    debug!(pid, proc_handle = ?proc_handle, "child process spawned");
    Some((proc_handle, thread_handle, pid))
}

/// Reap a child process and store its exit code.
#[allow(dead_code)]
fn wait_for_child(
    mut child: std::process::Child,
    exit_code: Arc<AtomicU32>,
    completed: Arc<(Mutex<bool>, Condvar)>,
) {
    let status = child.wait();
    let code = match status {
        Ok(s) => s.code().unwrap_or(1) as u32,
        Err(e) => {
            warn!(error = %e, "failed to wait for child process");
            1
        }
    };
    exit_code.store(code, Ordering::Release);
    let (lock, cvar) = &*completed;
    let mut done = lock.lock().unwrap();
    *done = true;
    cvar.notify_all();
}

/// Get the exit code of a process handle.
#[allow(dead_code)]
pub fn get_process_exit_code(h: HANDLE) -> Option<u32> {
    // Delegate to the handle table (reads the atomic exit_code).
    let inner = handle_table();
    // We need to peek at the entry without removing it.
    let table_inner = inner.get_waitable(h)?;
    match table_inner {
        rine_types::threading::Waitable::Process(p) => Some(p.exit_code.load(Ordering::Acquire)),
        _ => None,
    }
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

    #[test]
    fn split_trailing_space() {
        assert_eq!(split_cmd_line("a b "), vec!["a", "b"]);
    }

    // ── parse_command_line ───────────────────────────────────────

    #[test]
    fn parse_with_app_name() {
        let (exe, args) = parse_command_line("app.exe", "arg1 arg2");
        assert_eq!(exe, "app.exe");
        assert_eq!(args, vec!["arg1", "arg2"]);
    }

    #[test]
    fn parse_without_app_name() {
        let (exe, args) = parse_command_line("", "app.exe arg1 arg2");
        assert_eq!(exe, "app.exe");
        assert_eq!(args, vec!["arg1", "arg2"]);
    }

    #[test]
    fn parse_empty_both() {
        let (exe, args) = parse_command_line("", "");
        assert!(exe.is_empty());
        assert!(args.is_empty());
    }

    #[test]
    fn parse_app_name_no_args() {
        let (exe, args) = parse_command_line("app.exe", "");
        assert_eq!(exe, "app.exe");
        assert!(args.is_empty());
    }

    // ── parse_env_block ─────────────────────────────────────────

    #[test]
    fn env_block_null() {
        let env = parse_env_block(std::ptr::null());
        assert!(env.is_empty());
    }

    #[test]
    fn env_block_single_var() {
        let block = b"FOO=bar\0\0";
        let env = parse_env_block(block.as_ptr());
        assert_eq!(env.get("FOO").unwrap(), "bar");
        assert_eq!(env.len(), 1);
    }

    #[test]
    fn env_block_multiple_vars() {
        let block = b"A=1\0B=2\0C=hello world\0\0";
        let env = parse_env_block(block.as_ptr());
        assert_eq!(env.get("A").unwrap(), "1");
        assert_eq!(env.get("B").unwrap(), "2");
        assert_eq!(env.get("C").unwrap(), "hello world");
        assert_eq!(env.len(), 3);
    }

    #[test]
    fn env_block_empty_value() {
        let block = b"KEY=\0\0";
        let env = parse_env_block(block.as_ptr());
        assert_eq!(env.get("KEY").unwrap(), "");
    }

    // ── parse_env_block_wide ────────────────────────────────────

    #[test]
    fn env_block_wide_null() {
        let env = parse_env_block_wide(std::ptr::null());
        assert!(env.is_empty());
    }

    #[test]
    fn env_block_wide_single_var() {
        // "FOO=bar\0\0" in UTF-16LE
        let block: Vec<u16> = "FOO=bar\0\0".encode_utf16().collect();
        let env = parse_env_block_wide(block.as_ptr());
        assert_eq!(env.get("FOO").unwrap(), "bar");
        assert_eq!(env.len(), 1);
    }

    #[test]
    fn env_block_wide_multiple_vars() {
        let block: Vec<u16> = "A=1\0B=two\0\0".encode_utf16().collect();
        let env = parse_env_block_wide(block.as_ptr());
        assert_eq!(env.get("A").unwrap(), "1");
        assert_eq!(env.get("B").unwrap(), "two");
        assert_eq!(env.len(), 2);
    }
}
