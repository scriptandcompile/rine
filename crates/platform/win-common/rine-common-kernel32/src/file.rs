use rine_types::handles::{
    CREATE_ALWAYS, CREATE_NEW, GENERIC_READ, GENERIC_WRITE, HandleEntry, INVALID_HANDLE_VALUE,
    OPEN_ALWAYS, OPEN_EXISTING, TRUNCATE_EXISTING, handle_table,
};

/// Shared implementation for CreateFileA/W.
pub fn create_file(win_path: &str, desired_access: u32, creation_disposition: u32) -> isize {
    tracing::debug!(
        path = win_path,
        access = desired_access,
        disp = creation_disposition,
        "CreateFile"
    );

    // Build Linux open flags from Windows parameters.
    let mut flags: i32 = 0;

    let read = (desired_access & GENERIC_READ) != 0;
    let write = (desired_access & GENERIC_WRITE) != 0;
    if read && write {
        flags |= libc::O_RDWR;
    } else if write {
        flags |= libc::O_WRONLY;
    } else {
        flags |= libc::O_RDONLY;
    }

    match creation_disposition {
        CREATE_NEW => flags |= libc::O_CREAT | libc::O_EXCL,
        CREATE_ALWAYS => flags |= libc::O_CREAT | libc::O_TRUNC,
        OPEN_EXISTING => {} // no extra flags
        OPEN_ALWAYS => flags |= libc::O_CREAT,
        TRUNCATE_EXISTING => flags |= libc::O_TRUNC,
        _ => {
            tracing::warn!(
                disp = creation_disposition,
                "CreateFile: unknown creation disposition"
            );
            return INVALID_HANDLE_VALUE.as_raw();
        }
    }

    // Translate Windows path → Linux path.
    let linux_path = translate_win_path(win_path);

    let c_path = match std::ffi::CString::new(linux_path.to_string_lossy().as_bytes()) {
        Ok(s) => s,
        Err(_) => return INVALID_HANDLE_VALUE.as_raw(),
    };

    let mode: libc::mode_t = 0o644;
    let fd = unsafe { libc::open(c_path.as_ptr(), flags, mode as libc::c_uint) };
    if fd < 0 {
        tracing::debug!(path = %linux_path.display(), errno = std::io::Error::last_os_error().raw_os_error(), "CreateFile: open failed");
        return INVALID_HANDLE_VALUE.as_raw();
    }

    let h = handle_table().insert(HandleEntry::File(fd));
    tracing::debug!(handle = ?h, fd, path = %linux_path.display(), "CreateFile: opened");
    rine_types::dev_notify!(on_handle_created(
        h.as_raw() as i64,
        "File",
        &linux_path.display().to_string()
    ));
    h.as_raw()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Translate the directory portion of a FindFirstFile path to a Linux path.
pub fn translate_find_dir(dir_part: &str) -> std::path::PathBuf {
    if dir_part.is_empty() {
        return std::path::PathBuf::from(".");
    }
    translate_win_path(dir_part)
}

/// Translate a Windows path to a Linux path.
///
/// If the path already looks like a Linux path (`/…`), it's returned as-is.
/// Otherwise we apply a simple drive-letter mapping:
///   `X:\rest` → `~/.rine/drives/x/rest`
/// Backslashes are converted to forward slashes.
fn translate_win_path(win_path: &str) -> std::path::PathBuf {
    // Already a Linux absolute path — pass through.
    if win_path.starts_with('/') {
        return std::path::PathBuf::from(win_path);
    }

    let normalized = win_path.replace('\\', "/");

    // Strip \\?\ and \\.\ prefixes (now //?/ and //./).
    let stripped = normalized
        .strip_prefix("//?/")
        .or_else(|| normalized.strip_prefix("//./"))
        .unwrap_or(&normalized);

    // Check for drive letter: X:/…
    let bytes = stripped.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        let drive = (bytes[0] as char).to_ascii_lowercase();
        let rest = &stripped[2..];
        let rest = rest.strip_prefix('/').unwrap_or(rest);
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        let mut path = std::path::PathBuf::from(home);
        path.push(".rine/drives");
        path.push(drive.to_string());
        if !rest.is_empty() {
            path.push(rest);
        }
        return path;
    }

    // Relative or unrecognized — return as-is with normalized slashes.
    std::path::PathBuf::from(stripped)
}
