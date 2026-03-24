//! kernel32 process functions: ExitProcess, GetCommandLineA/W,
//! GetModuleHandleA/W.

use std::ffi::CString;
use std::sync::OnceLock;

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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ExitProcess(exit_code: u32) {
    std::process::exit(exit_code as i32);
}

/// GetCommandLineA — return a pointer to the ANSI command-line string.
///
/// # Safety
/// The returned pointer is valid for the lifetime of the process.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn GetCommandLineA() -> *const u8 {
    cached_cmd_line().ansi.as_ptr().cast()
}

/// GetCommandLineW — return a pointer to the wide command-line string.
///
/// # Safety
/// The returned pointer is valid for the lifetime of the process.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn GetCommandLineW() -> *const u16 {
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn GetModuleHandleA(module_name: *const u8) -> usize {
    if module_name.is_null() {
        // TODO: return the actual image base once the loader exposes it.
        tracing::debug!("GetModuleHandleA(NULL) — returning 0 (placeholder)");
        return 0;
    }

    let name = unsafe { std::ffi::CStr::from_ptr(module_name.cast()) };
    tracing::warn!(?name, "GetModuleHandleA: non-NULL module_name not yet supported");
    0
}

/// GetModuleHandleW — wide variant of `GetModuleHandleA`.
///
/// # Safety
/// `module_name` must be null or a valid null-terminated UTF-16LE string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn GetModuleHandleW(module_name: *const u16) -> usize {
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
    tracing::warn!(name, "GetModuleHandleW: non-NULL module_name not yet supported");
    0
}
