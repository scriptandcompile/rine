use std::path::Path;

use rine_dlls::DllRegistry;
use tracing::info;

use rine64_advapi32::Advapi32Plugin;
use rine64_gdi32::Gdi32Plugin;
use rine64_kernel32::Kernel32Plugin;
use rine64_msvcrt::{CrtForwarderPlugin, MsvcrtPlugin};
use rine64_ntdll::NtdllPlugin;
use rine64_user32::User32Plugin;
use rine64_ws2_32::Ws2_32Plugin;

use crate::cli::Cli;
use crate::config::errors::ConfigError;
use crate::config::manager::ConfigManager;
use crate::loader::memory::LoadedImage;
use crate::loader::resolver;
use crate::pe::parser::ParsedPe;
use crate::subsys;

pub fn run(exe_path: &Path, _cli: &Cli) -> Result<std::convert::Infallible, RunError> {
    info!(exe = %exe_path.display(), "loading PE");

    // 0. Load per-app configuration.
    let mgr = ConfigManager::new();
    let app_config = mgr.load(exe_path).map_err(RunError::Config)?;
    info!(
        version = %app_config.windows_version,
        config = %mgr.config_path(exe_path).display(),
        "app config loaded"
    );

    // Inject environment overrides from the config.
    for (key, value) in &app_config.environment {
        // SAFETY: rine is single-threaded at this point (before PE entry).
        unsafe { std::env::set_var(key, value) };
    }

    // 1. Parse the PE file.
    let parsed = ParsedPe::load(exe_path)?;
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
    let registry = DllRegistry::from_plugins(&[
        &Kernel32Plugin,
        &MsvcrtPlugin,
        &CrtForwarderPlugin,
        &NtdllPlugin,
        &Advapi32Plugin,
        &Gdi32Plugin,
        &User32Plugin,
        &Ws2_32Plugin,
    ]);
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

    // 5a. Set the spoofed Windows version from config.
    subsys::version::init_version(app_config.windows_version);

    // 5b. Set up fake Windows Thread Environment Block (TEB) so CRT code
    //     that reads gs:0x30 doesn't segfault.
    unsafe { subsys::threading::init_teb() };

    // 6. Execute the PE entry point (does not return).
    match crate::loader::entry::execute(&image, &parsed)? {}
}

/// Top-level error type wrapping all stages of PE loading and execution.
#[derive(Debug, thiserror::Error)]
pub enum RunError {
    #[error("{0}")]
    Config(#[source] ConfigError),

    #[error("{0}")]
    Pe(#[from] crate::pe::parser::PeError),

    #[error("{0}")]
    Loader(#[from] crate::loader::memory::LoaderError),

    #[error("{0}")]
    Resolver(#[from] crate::loader::resolver::ResolverError),

    #[error("{0}")]
    Entry(#[from] crate::loader::entry::EntryError),
}
