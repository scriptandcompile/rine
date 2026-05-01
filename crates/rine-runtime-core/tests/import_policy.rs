use std::path::PathBuf;

use rine_dlls::DllRegistry;
use rine_runtime_core::loader::memory::LoadedImage;
use rine_runtime_core::loader::resolver::{self, ResolverError};
use rine_runtime_core::pe::parser::ParsedPe;

fn fixture_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/bin")
        .join(rel)
}

#[test]
fn resolve_imports_fails_when_unimplemented_imports_exist() {
    let exe = fixture_path("x64/hello_puts.exe");
    assert!(exe.exists(), "missing fixture: {}", exe.display());

    let parsed = ParsedPe::load(&exe).expect("failed to parse fixture PE");
    let image = LoadedImage::load(&parsed).expect("failed to load fixture PE image");
    let empty_registry = DllRegistry::from_plugins(&[]);

    let result = resolver::resolve_imports(&image, &parsed.pe, parsed.format, &empty_registry);

    match result {
        Err(ResolverError::UnimplementedImports { imports, report }) => {
            assert!(
                !imports.is_empty(),
                "expected at least one unimplemented import"
            );
            assert!(
                imports.iter().all(|name| name.contains('!')),
                "expected DLL!Function formatting, got: {:?}",
                imports
            );
            assert!(
                !report.dll_summaries.is_empty(),
                "expected import summary report for dev dashboard"
            );
            assert!(
                report.total_unimplemented > 0,
                "expected unimplemented imports in report when unimplemented imports exist"
            );
            assert_eq!(
                report.total_stubbed, 0,
                "expected pure unimplemented imports for empty registry"
            );
        }
        other => panic!("expected UnimplementedImports error, got {other:?}"),
    }
}
