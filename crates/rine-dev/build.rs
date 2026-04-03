fn main() {
    rine_frontend_common::install_assets_to(env!("CARGO_MANIFEST_DIR"), "frontend/dist/js");

    // Ensure cargo rebuilds when any frontend file changes so Tauri
    // re-embeds the updated assets.
    println!("cargo:rerun-if-changed=frontend/dist");

    tauri_build::build();
}
