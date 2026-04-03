use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use thiserror::Error;

const IMAGE_FILE_MACHINE_I386: u16 = 0x014c;
const IMAGE_FILE_MACHINE_AMD64: u16 = 0x8664;
const DOS_MAGIC: [u8; 2] = *b"MZ";
const PE_SIGNATURE: [u8; 4] = *b"PE\0\0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeArchitecture {
    X86,
    X64,
    Unsupported(u16),
}

impl PeArchitecture {
    pub fn machine_name(self) -> &'static str {
        match self {
            Self::X86 => "x86 (PE32 / I386)",
            Self::X64 => "x64 (PE32+ / AMD64)",
            Self::Unsupported(_) => "unsupported",
        }
    }
}

#[derive(Debug, Error)]
pub enum ProbeError {
    #[error("failed to open file `{path}`: {source}")]
    Io {
        source: std::io::Error,
        path: PathBuf,
    },

    #[error("`{path}` is not a valid PE binary: missing DOS MZ signature")]
    MissingMz { path: PathBuf },

    #[error("`{path}` is not a valid PE binary: e_lfanew points outside file")]
    InvalidPeOffset { path: PathBuf },

    #[error("`{path}` is not a valid PE binary: missing PE signature")]
    MissingPeSignature { path: PathBuf },
}

/// Resolve a user-supplied executable path in the same way as parser::ParsedPe::load.
fn resolve_exe_path(path: &Path) -> PathBuf {
    if !path.exists() && path.extension().is_none() {
        path.with_extension("exe")
    } else {
        path.to_path_buf()
    }
}

/// Probe a PE file and return the executable machine architecture.
///
/// This reads only the fixed headers needed to identify PE machine type.
pub fn detect_architecture(path: &Path) -> Result<PeArchitecture, ProbeError> {
    let open_path = resolve_exe_path(path);
    let mut file = File::open(&open_path).map_err(|e| ProbeError::Io {
        source: e,
        path: open_path.clone(),
    })?;

    let mut mz = [0u8; 2];
    file.read_exact(&mut mz).map_err(|e| ProbeError::Io {
        source: e,
        path: open_path.clone(),
    })?;
    if mz != DOS_MAGIC {
        return Err(ProbeError::MissingMz { path: open_path });
    }

    file.seek(SeekFrom::Start(0x3c))
        .map_err(|e| ProbeError::Io {
            source: e,
            path: open_path.clone(),
        })?;

    let mut pe_offset_buf = [0u8; 4];
    file.read_exact(&mut pe_offset_buf)
        .map_err(|e| ProbeError::Io {
            source: e,
            path: open_path.clone(),
        })?;
    let pe_offset = u32::from_le_bytes(pe_offset_buf) as u64;

    if file.seek(SeekFrom::Start(pe_offset)).is_err() {
        return Err(ProbeError::InvalidPeOffset { path: open_path });
    }

    let mut signature = [0u8; 4];
    file.read_exact(&mut signature)
        .map_err(|_| ProbeError::MissingPeSignature {
            path: open_path.clone(),
        })?;
    if signature != PE_SIGNATURE {
        return Err(ProbeError::MissingPeSignature { path: open_path });
    }

    let mut machine_buf = [0u8; 2];
    file.read_exact(&mut machine_buf)
        .map_err(|e| ProbeError::Io {
            source: e,
            path: open_path.clone(),
        })?;
    let machine = u16::from_le_bytes(machine_buf);

    Ok(match machine {
        IMAGE_FILE_MACHINE_I386 => PeArchitecture::X86,
        IMAGE_FILE_MACHINE_AMD64 => PeArchitecture::X64,
        other => PeArchitecture::Unsupported(other),
    })
}
