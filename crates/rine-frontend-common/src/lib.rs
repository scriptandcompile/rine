//! Shared frontend assets for rine Tauri applications.
//!
//! This crate embeds common JS/CSS files and provides a build-time helper
//! to copy them into a Tauri app's `frontend/dist/` directory.

use std::path::{Path, PathBuf};

use resvg::{
    tiny_skia,
    usvg::{self, fontdb},
};

/// ANSI escape code → HTML converter (JavaScript).
pub const ANSI_JS: &str = include_str!("../assets/ansi.js");
const BRAND_ICON_ASSET: &str = "assets/rine-mark.svg";

/// List of all shared assets as `(filename, contents)` pairs.
pub const ASSETS: &[(&str, &str)] = &[("ansi.js", ANSI_JS)];

/// Write all shared assets into `dest_dir`.
///
/// Call this from your Tauri crate's `build.rs` **before** `tauri_build::build()`.
///
/// ```ignore
/// rine_frontend_common::install_assets("frontend/dist");
/// tauri_build::build();
/// ```
pub fn install_assets(_dest_dir: &str) {
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
        write_if_changed(&path, contents.as_bytes(), &format!("shared asset {name}"));
    }
}

/// Generate a PNG icon for Tauri bundle configuration from the shared SVG.
pub fn generate_icon_png_to(manifest_dir: &str, dest_path: &str, size: u32) {
    let svg_path = brand_icon_path();
    println!("cargo:rerun-if-changed={}", svg_path.display());

    let svg_data = std::fs::read(&svg_path).unwrap_or_else(|e| {
        panic!(
            "failed to read shared brand icon {}: {e}",
            svg_path.display()
        )
    });
    let png_data = rasterize_icon_png(&svg_data, size).unwrap_or_else(|e| {
        panic!(
            "failed to rasterize shared brand icon {}: {e}",
            svg_path.display()
        )
    });

    let output_path = Path::new(manifest_dir).join(dest_path);
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).unwrap_or_else(|e| {
            panic!(
                "failed to create icon output directory {}: {e}",
                parent.display()
            )
        });
    }

    write_if_changed(&output_path, &png_data, "generated Tauri icon");
}

fn brand_icon_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(BRAND_ICON_ASSET)
}

fn rasterize_icon_png(svg_data: &[u8], size: u32) -> Result<Vec<u8>, String> {
    let mut font_database = fontdb::Database::new();
    font_database.load_system_fonts();

    let options = usvg::Options {
        fontdb: std::sync::Arc::new(font_database),
        ..Default::default()
    };

    let tree = usvg::Tree::from_data(svg_data, &options)
        .map_err(|e| format!("failed to parse SVG: {e}"))?;
    let svg_size = tree.size();
    let scale_x = size as f32 / svg_size.width();
    let scale_y = size as f32 / svg_size.height();
    let transform = tiny_skia::Transform::from_scale(scale_x, scale_y);

    let mut pixmap = tiny_skia::Pixmap::new(size, size)
        .ok_or_else(|| format!("failed to allocate {size}x{size} pixmap"))?;
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    pixmap
        .encode_png()
        .map_err(|e| format!("failed to encode PNG: {e}"))
}

fn write_if_changed(path: &Path, bytes: &[u8], label: &str) {
    let needs_write = match std::fs::read(path) {
        Ok(existing) => existing != bytes,
        Err(_) => true,
    };

    if needs_write {
        std::fs::write(path, bytes)
            .unwrap_or_else(|e| panic!("failed to write {label} to {}: {e}", path.display()));
    }
}
