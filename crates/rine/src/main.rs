mod cli;
mod compat;
mod config;
mod integration;
mod loader;
mod pe;
mod subsys;

use std::process::ExitCode;

use clap::Parser;
use rine_dlls::registry::DllRegistry;
use tracing::{error, info};

use crate::cli::Cli;
use crate::loader::memory::LoadedImage;
use crate::loader::resolver;
use crate::pe::parser::ParsedPe;

fn main() -> ExitCode {
    tracing_subscriber::fmt().with_target(false).init();

    let cli = Cli::parse();

    match run(&cli) {
        Ok(infallible) => match infallible {},
        Err(e) => {
            error!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: &Cli) -> Result<std::convert::Infallible, RunError> {
    info!(exe = %cli.exe_path.display(), "loading PE");

    // 1. Parse the PE file.
    let parsed = ParsedPe::load(&cli.exe_path)?;
    info!(
        entry_rva = format_args!("{:#x}", parsed.pe.entry),
        image_base = format_args!("{:#x}", parsed.pe.image_base),
        sections = parsed.pe.sections.len(),
        "PE parsed"
    );

    // 2. Load PE into memory (mmap sections, apply relocations).
    let image = LoadedImage::load(&parsed)?;
    info!(
        base = format_args!("{}", image.base()),
        size = format_args!("{:#x}", image.size()),
        "image loaded"
    );

    // 3. Resolve imports (write function pointers into the IAT).
    let registry = DllRegistry::new();
    let report = resolver::resolve_imports(&image, &parsed.pe, &registry)?;
    info!(
        resolved = report.total_resolved,
        stubbed = report.total_stubbed,
        "imports resolved"
    );
    for dll in &report.dll_summaries {
        if !dll.stubbed_names.is_empty() {
            tracing::warn!(
                dll = dll.dll_name,
                stubs = ?dll.stubbed_names,
                "stubbed imports"
            );
        }
    }

    // Also attempt delay-load imports (currently just warns if present).
    let _ = resolver::resolve_delay_imports(&image, &parsed.pe, &registry);

    // 4. Set final memory protections on PE sections.
    image.protect(&parsed.pe)?;

    // 5. Set up fake Windows Thread Environment Block (TEB) so CRT code
    //    that reads gs:0x30 doesn't segfault.
    unsafe { subsys::threading::init_teb() };

    // 6. Execute the PE entry point (does not return).
    match loader::entry::execute(&image, &parsed)? {}
}

/// Top-level error type wrapping all stages of PE loading and execution.
#[derive(Debug, thiserror::Error)]
enum RunError {
    #[error("{0}")]
    Pe(#[from] pe::parser::PeError),

    #[error("{0}")]
    Loader(#[from] loader::memory::LoaderError),

    #[error("{0}")]
    Resolver(#[from] loader::resolver::ResolverError),

    #[error("{0}")]
    Entry(#[from] loader::entry::EntryError),
}
