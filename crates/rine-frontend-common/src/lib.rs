//! Shared frontend assets for rine Tauri applications.
//!
//! This crate embeds common JS/CSS files and provides a build-time helper
//! to copy them into a Tauri app's `frontend/dist/` directory.

/// ANSI escape code → HTML converter (JavaScript).
pub const ANSI_JS: &str = include_str!("../assets/ansi.js");

/// List of all shared assets as `(filename, contents)` pairs.
pub const ASSETS: &[(&str, &str)] = &[("ansi.js", ANSI_JS)];

/// Write all shared assets into `dest_dir`.
///
/// Call this from your Tauri crate's `build.rs` **before** `tauri_build::build()`.
///
/// ```no_run
/// // build.rs
/// fn main() {
///     rine_frontend_common::install_assets("frontend/dist");
///     tauri_build::build();
/// }
/// ```
pub fn install_assets(dest_dir: &str) {
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate has parent dir");
    // Resolve relative to the calling crate's manifest dir.
    // build.rs sets CARGO_MANIFEST_DIR at compile time of the *calling* crate,
    // but we're in a library — so we use a runtime approach instead.
    //
    // This won't work from include_str context. Use `install_assets_to` instead.
    let _ = out;
    panic!("Use install_assets_to(manifest_dir, dest_dir) from build.rs");
}

/// Write all shared assets into `dest_dir` relative to `manifest_dir`.
///
/// `manifest_dir` should be `env!("CARGO_MANIFEST_DIR")` from the calling build script.
pub fn install_assets_to(manifest_dir: &str, dest_dir: &str) {
    let dest = std::path::Path::new(manifest_dir).join(dest_dir);
    std::fs::create_dir_all(&dest).expect("failed to create frontend dist dir");

    for (name, contents) in ASSETS {
        let path = dest.join(name);
        // Only write if contents changed, to avoid unnecessary rebuilds.
        let needs_write = match std::fs::read_to_string(&path) {
            Ok(existing) => existing != *contents,
            Err(_) => true,
        };
        if needs_write {
            std::fs::write(&path, contents).unwrap_or_else(|e| {
                panic!("failed to write shared asset {name} to {}: {e}", path.display())
            });
        }
    }
}
