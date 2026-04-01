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

/// Conditionally emits a dev event. Compiles to nothing without the `dev` feature.
macro_rules! dev_emit {
    ($channel:expr, $event:expr) => {
        #[cfg(feature = "dev")]
        if let Some(ch) = $channel.as_mut() {
            let _ = ch.send(&$event);
        }
    };
}

pub fn run(
    exe_path: &Path,
    #[allow(unused)] cli: &Cli,
) -> Result<std::convert::Infallible, RunError> {
    info!(exe = %exe_path.display(), "loading PE");

    // ── Dev channel setup ──────────────────────────────────────────
    #[cfg(feature = "dev")]
    let mut dev_channel: Option<rine_channel::DevSender> = if cli.dev {
        None // Will be connected after rine-dev is spawned (handled in main.rs)
    } else {
        None
    };
    #[cfg(not(feature = "dev"))]
    let _dev_channel: Option<()> = None;

    // If --dev, spawn rine-dev and connect the channel.
    #[cfg(feature = "dev")]
    if cli.dev {
        let socket_path =
            std::env::temp_dir().join(format!("rine-dev-{}.sock", std::process::id()));

        // Spawn rine-dev binary.
        let dev_bin = std::env::current_exe()
            .ok()
            .and_then(|p| {
                let sibling = p.with_file_name("rine-dev");
                sibling.is_file().then_some(sibling)
            })
            .unwrap_or_else(|| "rine-dev".into());

        info!(bin = %dev_bin.display(), socket = %socket_path.display(), "launching rine-dev");

        match std::process::Command::new(&dev_bin)
            .arg("--socket")
            .arg(&socket_path)
            .spawn()
        {
            Ok(_child) => match rine_channel::DevSender::connect(&socket_path) {
                Ok(sender) => {
                    info!("connected to rine-dev dashboard");
                    dev_channel = Some(sender);
                }
                Err(e) => {
                    tracing::warn!("failed to connect to rine-dev: {e}");
                }
            },
            Err(e) => {
                tracing::warn!(
                    "failed to launch rine-dev ({}): {e}\n\
                     hint: make sure rine-dev is built (`cargo build -p rine-dev`)",
                    dev_bin.display()
                );
            }
        }
    }

    // 0. Load per-app configuration.
    let mgr = ConfigManager::new();
    let app_config = mgr.load(exe_path).map_err(RunError::Config)?;
    info!(
        version = %app_config.windows_version,
        config = %mgr.config_path(exe_path).display(),
        "app config loaded"
    );

    dev_emit!(
        dev_channel,
        rine_channel::DevEvent::ConfigLoaded {
            config_path: mgr.config_path(exe_path).display().to_string(),
            windows_version: app_config.windows_version.to_string(),
            environment_overrides: app_config
                .environment
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        }
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

    dev_emit!(
        dev_channel,
        rine_channel::DevEvent::PeLoaded {
            exe_path: exe_path.display().to_string(),
            image_base: image.base().as_usize() as u64,
            image_size: image.size() as u64,
            entry_rva: parsed.pe.entry as u64,
            relocation_delta: image.base().as_usize() as i64 - parsed.pe.image_base as i64,
            sections: parsed
                .pe
                .sections
                .iter()
                .map(|s| {
                    rine_channel::SectionInfo {
                        name: String::from_utf8_lossy(&s.name)
                            .trim_end_matches('\0')
                            .to_string(),
                        virtual_address: s.virtual_address as u64,
                        virtual_size: s.virtual_size as u64,
                        characteristics: s.characteristics,
                    }
                })
                .collect(),
        }
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

    dev_emit!(
        dev_channel,
        rine_channel::DevEvent::ImportsResolved {
            summaries: report
                .dll_summaries
                .iter()
                .map(|d| {
                    rine_channel::DllSummary {
                        dll_name: d.dll_name.clone(),
                        resolved: d.resolved,
                        stubbed: d.stubbed,
                        resolved_names: d.resolved_names.clone(),
                        stubbed_names: d.stubbed_names.clone(),
                    }
                })
                .collect(),
            total_resolved: report.total_resolved,
            total_stubbed: report.total_stubbed,
        }
    );

    // Also attempt delay-load imports (currently just warns if present).
    let _ = resolver::resolve_delay_imports(&image, &parsed.pe, &registry);

    // 4. Set final memory protections on PE sections.
    image.protect(&parsed.pe)?;

    // 5a. Set the spoofed Windows version from config.
    subsys::version::init_version(app_config.windows_version);

    // 5b. Set up fake Windows Thread Environment Block (TEB) so CRT code
    //     that reads gs:0x30 doesn't segfault.
    unsafe { subsys::threading::init_teb() };

    // 6. Execute the PE entry point.
    let exit_code = crate::loader::entry::execute(&image, &parsed)?;

    dev_emit!(
        dev_channel,
        rine_channel::DevEvent::ProcessExited {
            exit_code: exit_code,
        }
    );

    std::process::exit(exit_code);
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
