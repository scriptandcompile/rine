use std::fs::File;
use std::path::{Path, PathBuf};

use goblin::pe::PE;
use goblin::pe::characteristic;
use memmap2::Mmap;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeFormat {
    Pe32,
    Pe32Plus,
}

impl PeFormat {
    pub fn from_pe(pe: &PE) -> Self {
        if pe.is_64 { Self::Pe32Plus } else { Self::Pe32 }
    }
}

#[derive(Debug, Error)]
pub enum PeError {
    #[error("failed to open file `{path}`: {source}")]
    Io {
        source: std::io::Error,
        path: PathBuf,
    },

    #[error("failed to parse PE: {0}")]
    Parse(#[from] goblin::error::Error),

    #[error("binary is a DLL, not an executable")]
    IsDll,

    #[error("PE has no entry point")]
    NoEntryPoint,
}

/// A parsed PE file backed by a memory-mapped file.
///
/// The `PE` borrows from the `Mmap`, so both must live together.
pub struct ParsedPe {
    pub pe: PE<'static>,
    pub format: PeFormat,
    // The mmap must be kept alive for the lifetime of `pe`.
    // Safety: `pe` borrows from `_mmap`. We ensure `_mmap` is never moved or
    // dropped before `pe` by keeping them in the same struct, with `_mmap`
    // declared after `pe` (Rust drops fields in declaration order).
    _mmap: Mmap,
}

impl ParsedPe {
    /// Parse and validate a PE file from disk.
    ///
    /// Validates:
    /// - The file is a valid PE binary (goblin handles this)
    /// - It is PE32 or PE32+
    /// - It is not a DLL
    /// - It has a non-zero entry point
    pub fn load(path: &Path) -> Result<Self, PeError> {
        // Try the path as-is first; if it doesn't exist and has no extension,
        // retry with ".exe" appended (matching Windows behaviour).
        let resolved;
        let open_path = if !path.exists() && path.extension().is_none() {
            resolved = path.with_extension("exe");
            &resolved
        } else {
            path
        };

        let file = File::open(open_path).map_err(|e| PeError::Io {
            source: e,
            path: open_path.to_path_buf(),
        })?;
        // SAFETY: The file must not be modified while mapped. This is a
        // reasonable assumption for PE loading — the file is read-only input.
        let mmap = unsafe {
            Mmap::map(&file).map_err(|e| PeError::Io {
                source: e,
                path: open_path.to_path_buf(),
            })?
        };

        // Parse the mmap'd bytes. We need `pe` to borrow from `mmap` with a
        // 'static lifetime so they can coexist in the struct. We use unsafe to
        // extend the lifetime — this is sound because we keep both `pe` and
        // `_mmap` in `ParsedPe` and never expose the mmap to be dropped early.
        let bytes: &'static [u8] = unsafe { &*(mmap.as_ref() as *const [u8]) };
        let pe = PE::parse(bytes)?;
        let format = PeFormat::from_pe(&pe);

        validate(&pe)?;

        Ok(ParsedPe {
            pe,
            format,
            _mmap: mmap,
        })
    }

    /// Access the raw underlying file bytes (the mmap'd content).
    pub fn file_bytes(&self) -> &[u8] {
        &self._mmap
    }
}

/// Validate that the parsed PE meets rine's requirements.
fn validate(pe: &PE) -> Result<(), PeError> {
    if characteristic::is_dll(pe.header.coff_header.characteristics) {
        return Err(PeError::IsDll);
    }

    if pe.entry == 0 {
        return Err(PeError::NoEntryPoint);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn rejects_nonexistent_file() {
        let result = ParsedPe::load(Path::new("/nonexistent/fake.exe"));
        assert!(matches!(result, Err(PeError::Io { .. })));
    }

    #[test]
    fn rejects_non_pe_file() {
        // This source file is not a PE binary
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/pe/parser.rs");
        let result = ParsedPe::load(&path);
        assert!(matches!(result, Err(PeError::Parse(_))));
    }

    #[test]
    fn detects_pe32_plus_fixture_format() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/bin/hello_puts.exe");
        let parsed = ParsedPe::load(&path).expect("fixture should parse");
        assert_eq!(parsed.format, PeFormat::Pe32Plus);
    }
}
