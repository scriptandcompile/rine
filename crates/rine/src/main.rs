mod cli;
mod compat;
mod config;
mod integration;
mod loader;
mod pe;
mod subsys;

use std::process::ExitCode;

use clap::Parser;
use rine_dlls::DllRegistry;
use tracing::{error, info};

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

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

    // Handle `--config`: print/create the per-app config and exit.
    if cli.show_config {
        return show_config(&cli);
    }

    match run(&cli) {
        Ok(infallible) => match infallible {},
        Err(e) => {
            error!("{e}");
            ExitCode::FAILURE
        }
    }
}

/// Print the path and contents of the per-app config. Creates a default
/// config file if one does not yet exist.
fn show_config(cli: &Cli) -> ExitCode {
    let mgr = ConfigManager::new();
    let cfg = match mgr.load(&cli.exe_path) {
        Ok(c) => c,
        Err(e) => {
            error!("{e}");
            return ExitCode::FAILURE;
        }
    };

    let path = mgr.config_path(&cli.exe_path);
    if !path.exists() {
        match mgr.save(&cli.exe_path, &cfg) {
            Ok(p) => eprintln!("created default config: {}", p.display()),
            Err(e) => {
                error!("{e}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        eprintln!("config: {}", path.display());
    }

    match toml::to_string_pretty(&cfg) {
        Ok(s) => print!("{s}"),
        Err(e) => {
            error!("failed to serialise config: {e}");
            return ExitCode::FAILURE;
        }
    }

    ExitCode::SUCCESS
}

fn run(cli: &Cli) -> Result<std::convert::Infallible, RunError> {
    info!(exe = %cli.exe_path.display(), "loading PE");

    // 0. Load per-app configuration.
    let mgr = ConfigManager::new();
    let app_config = mgr.load(&cli.exe_path).map_err(RunError::Config)?;
    info!(
        version = %app_config.windows_version,
        config = %mgr.config_path(&cli.exe_path).display(),
        "app config loaded"
    );

    // Inject environment overrides from the config.
    for (key, value) in &app_config.environment {
        // SAFETY: rine is single-threaded at this point (before PE entry).
        unsafe { std::env::set_var(key, value) };
    }

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
    Config(#[source] ConfigError),

    #[error("{0}")]
    Pe(#[from] pe::parser::PeError),

    #[error("{0}")]
    Loader(#[from] loader::memory::LoaderError),

    #[error("{0}")]
    Resolver(#[from] loader::resolver::ResolverError),

    #[error("{0}")]
    Entry(#[from] loader::entry::EntryError),
}
