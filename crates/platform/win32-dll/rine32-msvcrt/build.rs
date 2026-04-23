fn main() {
    println!("cargo:rustc-check-cfg=cfg(rust_analyzer)");
    rine_dll_build::generate_metadata_code();
}
