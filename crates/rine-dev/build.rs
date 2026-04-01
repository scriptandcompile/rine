fn main() {
    rine_frontend_common::install_assets_to(env!("CARGO_MANIFEST_DIR"), "frontend/dist");
    tauri_build::build();
}
