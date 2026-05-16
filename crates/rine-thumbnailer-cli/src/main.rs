//! `rine-thumbnailer` – Freedesktop thumbnailer for Windows PE executables.
//!
//! Invoked by GNOME Files (Nautilus), Nemo, and other Freedesktop-compliant
//! file managers via the `.thumbnailer` descriptor:
//!
//! ```text
//! Exec=/usr/bin/rine-thumbnailer %u %o %s
//! ```
//!
//! - `%u`  Source URI (e.g. `file:///path/to/program.exe`) or a plain path.
//! - `%o`  Absolute path where the output PNG must be written.
//! - `%s`  Requested thumbnail size in pixels (square).
//!
//! The binary writes the output atomically (temp file + rename) and exits with
//! a non-zero status on any hard failure so the file manager can fall back to a
//! generic icon.

use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;
use rine_thumbnailer_core::{ThumbnailRequest, generate_png_thumbnail};

#[derive(Parser)]
#[command(name = "rine-thumbnailer", about = "Generate PNG thumbnails for Windows PE executables")]
struct Args {
    /// Source file: a `file://` URI or an absolute/relative path.
    input: String,

    /// Destination path where the PNG thumbnail should be written.
    output: PathBuf,

    /// Requested thumbnail size in pixels (used as both width and height).
    #[arg(default_value = "128")]
    size: u32,
}

fn main() {
    let args = Args::parse();

    let input_path = match resolve_input(&args.input) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("rine-thumbnailer: failed to resolve input {:?}: {e}", args.input);
            std::process::exit(1);
        }
    };

    let size = args.size.clamp(16, 1024);

    let req = ThumbnailRequest { input_path: &input_path, size_px: size };

    let png = match generate_png_thumbnail(&req) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("rine-thumbnailer: thumbnail generation failed for {:?}: {e}", input_path);
            std::process::exit(1);
        }
    };

    if let Err(e) = write_atomic(&args.output, &png) {
        eprintln!("rine-thumbnailer: failed to write {:?}: {e}", args.output);
        std::process::exit(1);
    }
}

/// Resolve the `input` argument to a filesystem path.
///
/// Handles:
/// - `file://` URIs (percent-decoded, host component ignored for `localhost`).
/// - Plain paths passed through as-is.
fn resolve_input(input: &str) -> Result<PathBuf, String> {
    if input.starts_with("file://") || input.starts_with("FILE://") {
        return parse_file_uri(input);
    }
    Ok(PathBuf::from(input))
}

fn parse_file_uri(uri: &str) -> Result<PathBuf, String> {
    let parsed = url::Url::parse(uri).map_err(|e| format!("invalid URI: {e}"))?;

    if parsed.scheme() != "file" {
        return Err(format!("unsupported URI scheme: {}", parsed.scheme()));
    }

    // url::Url::to_file_path handles percent-decoding and strips the host.
    parsed
        .to_file_path()
        .map_err(|_| format!("URI does not map to a local file path: {uri}"))
}

/// Write `data` to `dest` atomically using a sibling temporary file + rename.
///
/// This prevents file managers from reading a partially-written PNG if the
/// process is interrupted mid-write.
fn write_atomic(dest: &Path, data: &[u8]) -> std::io::Result<()> {
    let parent = dest.parent().unwrap_or(Path::new("."));
    // Use a fixed suffix so the temp file is on the same filesystem as dest.
    let tmp = parent.join(format!(
        ".rine-thumb-{}.tmp",
        dest.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("output")
    ));

    fs::write(&tmp, data)?;
    fs::rename(&tmp, dest)?;
    Ok(())
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_path_passthrough() {
        let p = resolve_input("/home/user/game.exe").unwrap();
        assert_eq!(p, PathBuf::from("/home/user/game.exe"));
    }

    #[test]
    fn file_uri_decoded() {
        let p = resolve_input("file:///home/user/my%20game.exe").unwrap();
        assert_eq!(p, PathBuf::from("/home/user/my game.exe"));
    }

    #[test]
    fn file_uri_simple() {
        let p = resolve_input("file:///usr/bin/notepad.exe").unwrap();
        assert_eq!(p, PathBuf::from("/usr/bin/notepad.exe"));
    }

    #[test]
    fn invalid_scheme_rejected() {
        assert!(resolve_input("https://example.com/app.exe").is_ok()); // treated as path
        assert!(parse_file_uri("https://example.com/app.exe").is_err());
    }

    #[test]
    fn write_atomic_roundtrip() {
        use std::env;
        let dir = env::temp_dir().join("rine_thumb_test");
        std::fs::create_dir_all(&dir).unwrap();
        let dest = dir.join("thumb_test.png");
        write_atomic(&dest, b"\x89PNG test").unwrap();
        let read = fs::read(&dest).unwrap();
        assert_eq!(read, b"\x89PNG test");
        // Temp file should be cleaned up
        assert!(!dir.join(".rine-thumb-thumb_test.png.tmp").exists());
        fs::remove_file(&dest).ok();
    }
}
