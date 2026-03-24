//! Filesystem path translation — maps Windows paths to Linux equivalents.
//!
//! Windows PE executables use paths like `C:\Users\foo\file.txt`. This module
//! translates them to Linux paths under a configurable root directory
//! (default: `~/.rine/drives/c/Users/foo/file.txt`).
//!
//! Features:
//! - Drive letter mapping (each letter → a Linux directory, configurable)
//! - Backslash → forward-slash conversion
//! - Optional case-insensitive lookup (find the real filename on disk)
//! - `\\?\` long path prefix stripping
//! - UNC path translation (`\\server\share` → configurable mount point)

use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

use thiserror::Error;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum PathError {
    #[error("empty path")]
    Empty,

    #[error("invalid drive letter: {0:?}")]
    InvalidDrive(String),

    #[error("no mapping configured for drive {0}:")]
    UnmappedDrive(char),

    #[error("relative Windows path without a current drive: {0:?}")]
    RelativeWithoutDrive(String),

    #[error("UNC paths are not yet supported: {0:?}")]
    UncNotSupported(String),
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configurable drive mappings and path-translation options.
///
/// Built from the per-app configuration (Phase 3) or sensible defaults.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PathTranslator {
    /// Map from uppercase drive letter → Linux directory.
    /// e.g. `'C'` → `/home/user/.rine/drives/c`
    drives: HashMap<char, PathBuf>,

    /// Fallback root used for drives that have no explicit mapping.
    /// The drive letter (lower-cased) is appended as a subdirectory.
    /// e.g. `~/.rine/drives` → drive `D:` maps to `~/.rine/drives/d`.
    default_root: PathBuf,

    /// When `true`, path lookups walk the filesystem to find the first
    /// entry whose name matches case-insensitively.  Expensive but
    /// necessary for some Windows programs.
    pub case_insensitive: bool,
}

#[allow(dead_code)]
impl PathTranslator {
    // -- constructors -------------------------------------------------------

    /// Create a translator with the default drive root (`~/.rine/drives`).
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        let default_root = PathBuf::from(home).join(".rine/drives");

        Self {
            drives: HashMap::new(),
            default_root,
            case_insensitive: false,
        }
    }

    /// Create a translator with a custom default root (useful for testing).
    pub fn with_root(root: PathBuf) -> Self {
        Self {
            drives: HashMap::new(),
            default_root: root,
            case_insensitive: false,
        }
    }

    /// Register an explicit mapping for a drive letter.
    pub fn map_drive(&mut self, letter: char, target: PathBuf) {
        self.drives.insert(letter.to_ascii_uppercase(), target);
    }

    // -- translation --------------------------------------------------------

    /// Translate a Windows path string to a Linux [`PathBuf`].
    ///
    /// Handles:
    ///  - `\\?\` and `\\.\` prefixes (stripped)
    ///  - `C:\...` absolute paths (drive mapped)
    ///  - UNC `\\server\share\...` paths (error for now)
    ///  - Relative paths like `subdir\file.txt` (resolved under `current_drive`)
    ///  - Backslash → forward slash conversion
    pub fn translate(
        &self,
        win_path: &str,
        current_drive: Option<char>,
    ) -> Result<PathBuf, PathError> {
        if win_path.is_empty() {
            return Err(PathError::Empty);
        }

        // Normalize separators: backslash → forward slash.
        let normalized = win_path.replace('\\', "/");

        // Strip Windows extended-length and device prefixes.
        let stripped = strip_path_prefix(&normalized);

        // Try to detect UNC.
        if stripped.starts_with("//") {
            return Err(PathError::UncNotSupported(win_path.to_owned()));
        }

        // Check for drive-letter absolute path: `X:/...`
        if let Some((drive, rest)) = parse_drive_prefix(stripped) {
            let linux_root = self.drive_root(drive)?;
            return Ok(join_and_clean(&linux_root, rest));
        }

        // Relative path — needs a current drive context.
        match current_drive {
            Some(d) => {
                let linux_root = self.drive_root(d)?;
                Ok(join_and_clean(&linux_root, stripped))
            }
            None => Err(PathError::RelativeWithoutDrive(win_path.to_owned())),
        }
    }

    /// Translate and then optionally resolve case-insensitively.
    ///
    /// If `self.case_insensitive` is `true`, each component is matched
    /// against the actual directory listing on disk.  Otherwise this is
    /// equivalent to [`translate`](Self::translate).
    pub fn translate_resolve(
        &self,
        win_path: &str,
        current_drive: Option<char>,
    ) -> Result<PathBuf, PathError> {
        let translated = self.translate(win_path, current_drive)?;

        if self.case_insensitive {
            Ok(resolve_case_insensitive(&translated))
        } else {
            Ok(translated)
        }
    }

    /// Return the Linux directory for a given drive letter.
    fn drive_root(&self, letter: char) -> Result<PathBuf, PathError> {
        let upper = letter.to_ascii_uppercase();

        if !upper.is_ascii_alphabetic() {
            return Err(PathError::InvalidDrive(letter.to_string()));
        }

        if let Some(mapped) = self.drives.get(&upper) {
            return Ok(mapped.clone());
        }

        // Fall back to default_root/<lowercase letter>
        Ok(self
            .default_root
            .join(upper.to_ascii_lowercase().to_string()))
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Strip `\\?\`, `\\.\`, `//?/`, `//./` prefixes.
#[allow(dead_code)]
fn strip_path_prefix(path: &str) -> &str {
    // After backslash normalization these appear as `//?/` or `//./`.
    for prefix in &["//?/", "//./"] {
        if let Some(rest) = path.strip_prefix(prefix) {
            return rest;
        }
    }
    path
}

/// Parse `X:/rest` or `X:rest` — returns `(drive_letter, remainder)`.
#[allow(dead_code)]
fn parse_drive_prefix(path: &str) -> Option<(char, &str)> {
    let bytes = path.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        let drive = (bytes[0] as char).to_ascii_uppercase();
        let rest = &path[2..];
        // Skip the leading `/` if present (e.g. `C:/foo` → `foo`).
        let rest = rest.strip_prefix('/').unwrap_or(rest);
        Some((drive, rest))
    } else {
        None
    }
}

/// Join a linux root with the remaining Windows-style path, cleaning up
/// `.` and `..` components (purely lexical — no I/O).
#[allow(dead_code)]
fn join_and_clean(root: &Path, relative: &str) -> PathBuf {
    let mut result = root.to_path_buf();

    for component in Path::new(relative).components() {
        match component {
            Component::CurDir => {} // skip `.`
            Component::ParentDir if result.starts_with(root) && result != root => {
                result.pop();
            }
            Component::Normal(seg) => result.push(seg),
            // RootDir, Prefix — shouldn't appear in the relative remainder.
            _ => {}
        }
    }

    result
}

/// Walk `path` from the root, resolving each component case-insensitively
/// by scanning the directory listing. If a component can't be found, the
/// remaining tail is appended in its original casing.
#[allow(dead_code)]
fn resolve_case_insensitive(path: &Path) -> PathBuf {
    let mut resolved = PathBuf::new();
    let mut components = path.components().peekable();

    // Preserve the root / prefix as-is.
    while let Some(comp) = components.peek() {
        match comp {
            Component::Prefix(p) => {
                resolved.push(p.as_os_str());
                components.next();
            }
            Component::RootDir => {
                resolved.push(Component::RootDir);
                components.next();
            }
            _ => break,
        }
    }

    for component in components {
        match component {
            Component::Normal(seg) => {
                let target = seg.to_string_lossy();
                match find_case_match(&resolved, &target) {
                    Some(found) => resolved.push(found),
                    None => resolved.push(seg), // keep original casing
                }
            }
            other => resolved.push(other),
        }
    }

    resolved
}

/// Scan `dir` for an entry whose name matches `name` case-insensitively.
/// Returns the real filename on match, or `None`.
#[allow(dead_code)]
fn find_case_match(dir: &Path, name: &str) -> Option<String> {
    let lower = name.to_ascii_lowercase();
    let entries = std::fs::read_dir(dir).ok()?;

    for entry in entries.flatten() {
        let fname = entry.file_name();
        let s = fname.to_string_lossy();
        if s.to_ascii_lowercase() == lower {
            return Some(s.into_owned());
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn translator(root: &str) -> PathTranslator {
        PathTranslator::with_root(PathBuf::from(root))
    }

    // -- basic drive mapping ------------------------------------------------

    #[test]
    fn absolute_c_drive_path() {
        let t = translator("/mnt/rine");
        let result = t
            .translate(r"C:\Windows\System32\notepad.exe", None)
            .unwrap();
        assert_eq!(
            result,
            PathBuf::from("/mnt/rine/c/Windows/System32/notepad.exe")
        );
    }

    #[test]
    fn drive_letter_is_case_insensitive() {
        let t = translator("/mnt/rine");
        let lower = t.translate(r"c:\file.txt", None).unwrap();
        let upper = t.translate(r"C:\file.txt", None).unwrap();
        assert_eq!(lower, upper);
    }

    #[test]
    fn explicit_drive_mapping() {
        let mut t = translator("/mnt/rine");
        t.map_drive('D', PathBuf::from("/media/data"));
        let result = t.translate(r"D:\Games\app.exe", None).unwrap();
        assert_eq!(result, PathBuf::from("/media/data/Games/app.exe"));
    }

    #[test]
    fn unmapped_drive_falls_back_to_default() {
        let t = translator("/mnt/rine");
        let result = t.translate(r"E:\something", None).unwrap();
        assert_eq!(result, PathBuf::from("/mnt/rine/e/something"));
    }

    // -- prefix stripping ---------------------------------------------------

    #[test]
    fn long_path_prefix_stripped() {
        let t = translator("/mnt/rine");
        let result = t.translate(r"\\?\C:\very\long\path.txt", None).unwrap();
        assert_eq!(result, PathBuf::from("/mnt/rine/c/very/long/path.txt"));
    }

    #[test]
    fn device_prefix_stripped() {
        let t = translator("/mnt/rine");
        let result = t.translate(r"\\.\C:\dev\file", None).unwrap();
        assert_eq!(result, PathBuf::from("/mnt/rine/c/dev/file"));
    }

    // -- relative paths -----------------------------------------------------

    #[test]
    fn relative_path_with_current_drive() {
        let t = translator("/mnt/rine");
        let result = t.translate(r"subdir\file.txt", Some('C')).unwrap();
        assert_eq!(result, PathBuf::from("/mnt/rine/c/subdir/file.txt"));
    }

    #[test]
    fn relative_path_without_drive_is_error() {
        let t = translator("/mnt/rine");
        assert!(t.translate(r"subdir\file.txt", None).is_err());
    }

    // -- dot components -----------------------------------------------------

    #[test]
    fn dot_components_resolved() {
        let t = translator("/mnt/rine");
        let result = t.translate(r"C:\foo\.\bar\..\baz.txt", None).unwrap();
        assert_eq!(result, PathBuf::from("/mnt/rine/c/foo/baz.txt"));
    }

    #[test]
    fn parent_does_not_escape_drive_root() {
        let t = translator("/mnt/rine");
        let result = t.translate(r"C:\..\..\..\etc\passwd", None).unwrap();
        assert_eq!(result, PathBuf::from("/mnt/rine/c/etc/passwd"));
    }

    // -- edge cases ---------------------------------------------------------

    #[test]
    fn empty_path_is_error() {
        let t = translator("/mnt/rine");
        assert!(matches!(t.translate("", None), Err(PathError::Empty)));
    }

    #[test]
    fn unc_path_returns_error() {
        let t = translator("/mnt/rine");
        assert!(matches!(
            t.translate(r"\\server\share\file", None),
            Err(PathError::UncNotSupported(_))
        ));
    }

    #[test]
    fn forward_slashes_work() {
        let t = translator("/mnt/rine");
        let result = t.translate("C:/Users/test/file.txt", None).unwrap();
        assert_eq!(result, PathBuf::from("/mnt/rine/c/Users/test/file.txt"));
    }

    #[test]
    fn drive_root_only() {
        let t = translator("/mnt/rine");
        let result = t.translate(r"C:\", None).unwrap();
        assert_eq!(result, PathBuf::from("/mnt/rine/c"));
    }

    #[test]
    fn drive_colon_no_slash() {
        let t = translator("/mnt/rine");
        let result = t.translate("C:file.txt", Some('C')).unwrap();
        assert_eq!(result, PathBuf::from("/mnt/rine/c/file.txt"));
    }

    // -- case-insensitive resolution ----------------------------------------

    #[test]
    fn case_insensitive_resolves_existing_dir() {
        // This test operates on real filesystem, so use /tmp.
        let root = std::env::temp_dir().join("rine_test_ci");
        let drive_dir = root.join("c");
        let sub = drive_dir.join("MyFolder");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(sub.join("Hello.TXT"), b"hi").unwrap();

        let mut t = PathTranslator::with_root(root.clone());
        t.case_insensitive = true;

        let result = t.translate_resolve(r"C:\myfolder\hello.txt", None).unwrap();
        // Should find the real casing.
        assert_eq!(result, drive_dir.join("MyFolder").join("Hello.TXT"));

        // Cleanup.
        let _ = std::fs::remove_dir_all(&root);
    }
}
