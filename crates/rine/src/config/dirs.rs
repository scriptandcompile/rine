//! Filesystem helpers for locating rine's data directories.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

/// `~/.rine`
pub fn rine_root() -> PathBuf {
    home().join(".rine")
}

fn home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
}

/// Produce a short, filesystem-safe identifier from an exe path.
///
/// Format: `<stem>-<hex hash>` (e.g. `hello-a1b2c3d4`).
pub fn app_hash(exe_path: &Path) -> String {
    let canonical = exe_path
        .canonicalize()
        .unwrap_or_else(|_| exe_path.to_path_buf());
    let stem = exe_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    // Sanitise the stem to be filesystem-safe.
    let stem: String = stem
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    let mut hasher = DefaultHasher::new();
    canonical.hash(&mut hasher);
    let hash = hasher.finish();
    format!("{}-{:016x}", stem, hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_hash_deterministic() {
        let h1 = app_hash(Path::new("/some/fake/path.exe"));
        let h2 = app_hash(Path::new("/some/fake/path.exe"));
        assert_eq!(h1, h2);
    }

    #[test]
    fn app_hash_includes_stem() {
        let h = app_hash(Path::new("/foo/bar/game.exe"));
        assert!(h.starts_with("game-"), "got: {h}");
    }
}
