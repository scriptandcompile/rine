//! Win32 `WIN32_FIND_DATA` structures (ANSI and wide) and glob-matching
//! helpers for `FindFirstFile`/`FindNextFile`.

use super::handle_tables::FindEntry;
use crate::handles::{FILE_ATTRIBUTE_ARCHIVE, FILE_ATTRIBUTE_DIRECTORY};

// ---------------------------------------------------------------------------
// Win32 find-data structure (ANSI, as returned by FindFirstFileA)
// ---------------------------------------------------------------------------

/// `WIN32_FIND_DATAA` — the struct Windows fills in during FindFirstFileA /
/// FindNextFileA.  We only populate the fields that programs commonly read.
#[repr(C)]
pub struct Win32FindDataA {
    pub file_attributes: u32,
    pub creation_time_lo: u32,
    pub creation_time_hi: u32,
    pub last_access_time_lo: u32,
    pub last_access_time_hi: u32,
    pub last_write_time_lo: u32,
    pub last_write_time_hi: u32,
    pub file_size_high: u32,
    pub file_size_low: u32,
    pub reserved0: u32,
    pub reserved1: u32,
    pub file_name: [u8; 260],
    pub alternate_file_name: [u8; 14],
}

impl Win32FindDataA {
    pub fn from_entry(entry: &FindEntry) -> Self {
        let mut data = Self {
            file_attributes: entry.attributes,
            creation_time_lo: 0,
            creation_time_hi: 0,
            last_access_time_lo: 0,
            last_access_time_hi: 0,
            last_write_time_lo: 0,
            last_write_time_hi: 0,
            file_size_high: (entry.file_size >> 32) as u32,
            file_size_low: entry.file_size as u32,
            reserved0: 0,
            reserved1: 0,
            file_name: [0u8; 260],
            alternate_file_name: [0u8; 14],
        };
        let name_bytes = entry.file_name.as_bytes();
        let copy_len = name_bytes.len().min(259);
        data.file_name[..copy_len].copy_from_slice(&name_bytes[..copy_len]);
        data
    }
}

/// Wide variant — `WIN32_FIND_DATAW`.
#[repr(C)]
pub struct Win32FindDataW {
    pub file_attributes: u32,
    pub creation_time_lo: u32,
    pub creation_time_hi: u32,
    pub last_access_time_lo: u32,
    pub last_access_time_hi: u32,
    pub last_write_time_lo: u32,
    pub last_write_time_hi: u32,
    pub file_size_high: u32,
    pub file_size_low: u32,
    pub reserved0: u32,
    pub reserved1: u32,
    pub file_name: [u16; 260],
    pub alternate_file_name: [u16; 14],
}

impl Win32FindDataW {
    pub fn from_entry(entry: &FindEntry) -> Self {
        let mut data = Self {
            file_attributes: entry.attributes,
            creation_time_lo: 0,
            creation_time_hi: 0,
            last_access_time_lo: 0,
            last_access_time_hi: 0,
            last_write_time_lo: 0,
            last_write_time_hi: 0,
            file_size_high: (entry.file_size >> 32) as u32,
            file_size_low: entry.file_size as u32,
            reserved0: 0,
            reserved1: 0,
            file_name: [0u16; 260],
            alternate_file_name: [0u16; 14],
        };
        let wide: Vec<u16> = entry.file_name.encode_utf16().collect();
        let copy_len = wide.len().min(259);
        data.file_name[..copy_len].copy_from_slice(&wide[..copy_len]);
        data
    }
}

// ---------------------------------------------------------------------------
// Glob matching for FindFirstFile patterns
// ---------------------------------------------------------------------------

/// Simple glob match supporting `*` and `?` (Windows FindFirstFile semantics).
/// Case-insensitive.
pub fn glob_matches(pattern: &str, name: &str) -> bool {
    glob_match_bytes(
        pattern.to_ascii_lowercase().as_bytes(),
        name.to_ascii_lowercase().as_bytes(),
    )
}

fn glob_match_bytes(pattern: &[u8], name: &[u8]) -> bool {
    let (mut pi, mut ni) = (0, 0);
    let (mut star_p, mut star_n) = (usize::MAX, 0);

    while ni < name.len() {
        if pi < pattern.len() && (pattern[pi] == b'?' || pattern[pi] == name[ni]) {
            pi += 1;
            ni += 1;
        } else if pi < pattern.len() && pattern[pi] == b'*' {
            star_p = pi;
            star_n = ni;
            pi += 1;
        } else if star_p != usize::MAX {
            pi = star_p + 1;
            star_n += 1;
            ni = star_n;
        } else {
            return false;
        }
    }

    while pi < pattern.len() && pattern[pi] == b'*' {
        pi += 1;
    }
    pi == pattern.len()
}

// ---------------------------------------------------------------------------
// Helper: collect directory entries matching a glob
// ---------------------------------------------------------------------------

/// Read a directory and collect entries matching `pattern`.
pub fn collect_find_entries(dir: &std::path::Path, pattern: &str) -> Vec<FindEntry> {
    let mut entries = Vec::new();
    let Ok(read_dir) = std::fs::read_dir(dir) else {
        return entries;
    };

    for result in read_dir {
        let Ok(de) = result else { continue };
        let fname = de.file_name().to_string_lossy().into_owned();
        if !glob_matches(pattern, &fname) {
            continue;
        }
        let meta = de.metadata().ok();
        let file_size = meta.as_ref().map_or(0, |m| m.len());
        let is_dir = meta.as_ref().is_some_and(|m| m.is_dir());
        let attributes = if is_dir {
            FILE_ATTRIBUTE_DIRECTORY
        } else {
            FILE_ATTRIBUTE_ARCHIVE
        };
        entries.push(FindEntry {
            file_name: fname,
            file_size,
            attributes,
        });
    }
    entries
}

// ---------------------------------------------------------------------------
// Path utilities for splitting "dir\pattern"
// ---------------------------------------------------------------------------

/// Split a Windows find-file path (e.g. `C:\Dir\*.txt`) into the directory
/// portion and the glob pattern.
pub fn split_find_path(win_path: &str) -> (&str, &str) {
    let sep = win_path.rfind(['\\', '/']).map(|i| i + 1);
    match sep {
        Some(pos) => (&win_path[..pos], &win_path[pos..]),
        None => ("", win_path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_basic() {
        assert!(glob_matches("*", "anything"));
        assert!(glob_matches("*.txt", "readme.txt"));
        assert!(!glob_matches("*.txt", "readme.md"));
        assert!(glob_matches("test?", "test1"));
        assert!(!glob_matches("test?", "test12"));
        assert!(glob_matches("*.*", "file.txt"));
        assert!(glob_matches("FILE.*", "file.TXT"));
    }

    #[test]
    fn split_find_path_cases() {
        assert_eq!(split_find_path(r"C:\Dir\*.txt"), (r"C:\Dir\", "*.txt"));
        assert_eq!(split_find_path("*.exe"), ("", "*.exe"));
        assert_eq!(split_find_path(r"foo\bar"), (r"foo\", "bar"));
    }
}
