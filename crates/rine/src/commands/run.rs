use std::path::Path;
use std::process::Command;
#[cfg(feature = "dev")]
use std::{
    collections::HashMap,
    io::Write,
    sync::{LazyLock, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(feature = "dev")]
use rine_dev_bridge::{
    DevBridge, DevBridgeObserver, DevEventSink, imports_resolved_event, pe_loaded_event,
};
use rine_dlls::DllRegistry;
#[cfg(feature = "dev")]
use serde::Serialize;
use tracing::info;

use rine_runtime_core::loader::{entry, memory::LoadedImage, resolver};
use rine_runtime_core::loader::{entry::EntryError, memory::LoaderError, resolver::ResolverError};
use rine_runtime_core::pe::parser::{ParsedPe, PeError};
use rine64_advapi32::Advapi32Plugin;
use rine64_comdlg32::Comdlg32Plugin;
use rine64_gdi32::Gdi32Plugin;
use rine64_kernel32::Kernel32Plugin;
use rine64_msvcrt::{CrtForwarderPlugin, MsvcrtPlugin};
use rine64_user32::User32Plugin;
use rine64_ws2_32::Ws2_32Plugin;

use crate::cli::Cli;
use crate::commands::window_host::{WINDOW_HOST_SOCKET_ENV, WindowHostSession};
use crate::config::errors::ConfigError;
use crate::config::manager::ConfigManager;
use crate::pe::probe::{PeArchitecture, ProbeError, detect_architecture};
use crate::subsys;

fn emit_registry_metrics(registry: &DllRegistry) {
    let metrics = registry.metrics();
    rine_types::dev_notify!(on_dll_registry_metrics(
        rine_types::dev_hooks::DllRegistryMetricsTelemetry {
            registered_dlls: metrics.registered_dlls,
            loaded_dlls: metrics.loaded_dlls,
            name_lookups: metrics.name_lookups,
            ordinal_lookups: metrics.ordinal_lookups,
            lazy_loads: metrics.lazy_loads,
            cache_hits: metrics.cache_hits,
        }
    ));
}

const DYNAMIC_PROVIDER_DIR_ENV: &str = "RINE_PLUGIN_DIR";
const NTDLL_PROVIDER_LIBRARY_NAME: &str = "librine64_ntdll.so";

fn set_var_if_absent(key: &str, value: &str) {
    if std::env::var_os(key).is_none() {
        // SAFETY: invoked before PE entry while runtime is still single-threaded.
        unsafe { std::env::set_var(key, value) };
    }
}

fn current_thread_id() -> u32 {
    // Linux thread ID is the closest runtime identifier to Win32 thread ID.
    unsafe { libc::syscall(libc::SYS_gettid) as u32 }
}

fn resolve_ntdll_provider_path() -> Result<std::path::PathBuf, RunError> {
    if let Some(dir) = std::env::var_os(DYNAMIC_PROVIDER_DIR_ENV) {
        let path = std::path::PathBuf::from(dir).join(NTDLL_PROVIDER_LIBRARY_NAME);
        if path.is_file() {
            return Ok(path);
        }

        return Err(RunError::DynamicProviderNotFound {
            path,
            lookup: DYNAMIC_PROVIDER_DIR_ENV,
        });
    }

    let exe = std::env::current_exe().map_err(RunError::CurrentExe)?;
    let path = exe
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(NTDLL_PROVIDER_LIBRARY_NAME);
    if path.is_file() {
        return Ok(path);
    }

    Err(RunError::DynamicProviderNotFound {
        path,
        lookup: "runtime directory",
    })
}

/// Conditionally emits a dev event. Compiles to nothing without the `dev` feature.
macro_rules! dev_emit {
    ($bridge:expr, $event:expr) => {
        #[cfg(feature = "dev")]
        if let Some(bridge) = $bridge.as_ref() {
            let _ = bridge.send(&$event);
        }
    };
}

#[cfg(feature = "dev")]
#[derive(Debug, Clone)]
struct TrackedMemoryRegion {
    address: u64,
    size: u64,
    source: String,
}

#[cfg(feature = "dev")]
static MEMORY_REGIONS: LazyLock<Mutex<HashMap<u64, TrackedMemoryRegion>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[cfg(feature = "dev")]
#[derive(Debug, Serialize)]
struct SnapshotRegion {
    address: u64,
    size: u64,
    source: String,
    file_offset: u64,
}

#[cfg(feature = "dev")]
#[derive(Debug, Serialize)]
struct SnapshotManifest {
    format: String,
    pid: u32,
    created_unix_ms: u128,
    region_count: usize,
    total_bytes: u64,
    regions: Vec<SnapshotRegion>,
}

#[cfg(feature = "dev")]
fn track_memory_alloc(address: u64, size: u64, source: &str) {
    if address == 0 || size == 0 {
        return;
    }
    MEMORY_REGIONS.lock().unwrap().insert(
        address,
        TrackedMemoryRegion {
            address,
            size,
            source: source.to_owned(),
        },
    );
}

#[cfg(feature = "dev")]
fn track_memory_free(address: u64) {
    MEMORY_REGIONS.lock().unwrap().remove(&address);
}

#[cfg(feature = "dev")]
fn write_memory_snapshot_files() -> Option<(String, String, usize, u64)> {
    let regions: Vec<TrackedMemoryRegion> = {
        let guard = MEMORY_REGIONS.lock().ok()?;
        let mut items = guard.values().cloned().collect::<Vec<_>>();
        items.sort_by_key(|r| r.address);
        items
    };

    if regions.is_empty() {
        return None;
    }

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_millis();
    let pid = std::process::id();
    let out_dir = std::env::temp_dir().join(format!("rine-memory-snapshot-{pid}-{ts}"));
    std::fs::create_dir_all(&out_dir).ok()?;

    let bin_path = out_dir.join("snapshot.bin");
    let json_path = out_dir.join("snapshot.json");

    let mut bin_file = std::fs::File::create(&bin_path).ok()?;
    let mut manifest_regions = Vec::with_capacity(regions.len());
    let mut offset = 0u64;

    for region in regions {
        if region.size == 0 {
            continue;
        }

        let size = region.size as usize;
        // SAFETY: Regions are tracked from successful alloc events and are expected
        // to remain valid while marked active.
        let bytes = unsafe { std::slice::from_raw_parts(region.address as *const u8, size) };
        if bin_file.write_all(bytes).is_err() {
            continue;
        }

        manifest_regions.push(SnapshotRegion {
            address: region.address,
            size: region.size,
            source: region.source,
            file_offset: offset,
        });
        offset = offset.saturating_add(region.size);
    }

    if manifest_regions.is_empty() {
        return None;
    }

    let manifest = SnapshotManifest {
        format: "rine-memory-snapshot-v1".to_owned(),
        pid,
        created_unix_ms: ts,
        region_count: manifest_regions.len(),
        total_bytes: offset,
        regions: manifest_regions,
    };

    let json = serde_json::to_vec_pretty(&manifest).ok()?;
    std::fs::write(&json_path, json).ok()?;

    Some((
        json_path.to_string_lossy().into_owned(),
        bin_path.to_string_lossy().into_owned(),
        manifest.region_count,
        manifest.total_bytes,
    ))
}

#[cfg(feature = "dev")]
fn emit_memory_snapshot_ready(sink: &dyn DevEventSink) {
    if let Some((json_path, bin_path, region_count, total_bytes)) = write_memory_snapshot_files() {
        sink.send_event(rine_channel::DevEvent::MemorySnapshotReady {
            json_path,
            bin_path,
            region_count,
            total_bytes,
        });
    }
}

#[cfg(feature = "dev")]
struct SnapshotObserver;

#[cfg(feature = "dev")]
impl DevBridgeObserver for SnapshotObserver {
    fn on_memory_allocated(&self, _sink: &dyn DevEventSink, address: u64, size: u64, source: &str) {
        track_memory_alloc(address, size, source);
    }

    fn on_memory_freed(&self, _sink: &dyn DevEventSink, address: u64, size: u64, source: &str) {
        let _ = (size, source);
        track_memory_free(address);
    }

    fn on_process_exiting(&self, sink: &dyn DevEventSink, exit_code: i32) {
        let _ = exit_code;
        emit_memory_snapshot_ready(sink);
    }
}

pub fn run(
    exe_path: &Path,
    #[allow(unused)] cli: &Cli,
) -> Result<std::convert::Infallible, RunError> {
    info!(exe = %exe_path.display(), "loading PE");

    // ── Dev channel setup ──────────────────────────────────────────
    #[cfg(not(feature = "dev"))]
    let _dev_channel: Option<()> = None;
    #[cfg(feature = "dev")]
    let _dev_channel = DevBridge::init_from_env_with_observer("rine", SnapshotObserver);

    // Route by PE machine type so users can keep using `rine <exe>`.
    let arch = detect_architecture(exe_path).map_err(RunError::Probe)?;
    info!(
        architecture = arch.machine_name(),
        "detected PE architecture"
    );
    if let PeArchitecture::X86 = arch {
        // x86 executables are executed by the helper runtime (`rine32`). Emit
        // a synthetic PE lifecycle event so the dashboard can show progress
        // instead of remaining in the initial waiting state.
        dev_emit!(
            _dev_channel,
            rine_channel::DevEvent::PeLoaded {
                exe_path: exe_path.display().to_string(),
                architecture: "32-bit (PE32 / x86) via rine32 dispatch".to_owned(),
                image_base: 0,
                image_size: 0,
                entry_rva: 0,
                relocation_delta: 0,
                sections: Vec::new(),
            }
        );
        return dispatch_to_rine32(
            exe_path,
            cli,
            #[cfg(feature = "dev")]
            _dev_channel.as_ref(),
        )
        .map_err(RunError::Dispatch);
    }
    if let PeArchitecture::Unsupported(machine) = arch {
        return Err(RunError::UnsupportedMachine(machine));
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
        _dev_channel,
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
        format = ?parsed.format,
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

    dev_emit!(_dev_channel, pe_loaded_event(exe_path, &parsed, &image));

    dev_emit!(
        _dev_channel,
        rine_channel::DevEvent::MemoryAllocated {
            address: image.base().as_usize() as u64,
            size: image.size() as u64,
            source: "PE Image".to_owned(),
        }
    );
    #[cfg(feature = "dev")]
    track_memory_alloc(
        image.base().as_usize() as u64,
        image.size() as u64,
        "PE Image",
    );

    // 3. Resolve imports (write function pointers into the IAT).
    let ntdll_provider_path = resolve_ntdll_provider_path()?;

    let mut registry = DllRegistry::new_lazy();
    registry.register_plugin_factory(|| Box::new(Kernel32Plugin));
    registry.register_plugin_factory(|| Box::new(MsvcrtPlugin));
    registry.register_plugin_factory(|| Box::new(CrtForwarderPlugin));
    registry.register_dynamic_provider_library(&["ntdll.dll"], &ntdll_provider_path);
    registry.register_plugin_factory(|| Box::new(Advapi32Plugin));
    registry.register_plugin_factory(|| Box::new(Gdi32Plugin));
    registry.register_plugin_factory(|| Box::new(Comdlg32Plugin));
    registry.register_plugin_factory(|| Box::new(User32Plugin));
    registry.register_plugin_factory(|| Box::new(Ws2_32Plugin));
    emit_registry_metrics(&registry);
    let report = match resolver::resolve_imports(&image, &parsed.pe, parsed.format, &registry) {
        Ok(report) => report,
        Err(resolver::ResolverError::UnimplementedImports { imports, report }) => {
            emit_registry_metrics(&registry);
            dev_emit!(_dev_channel, imports_resolved_event(&report));
            return Err(RunError::Resolver(
                resolver::ResolverError::UnimplementedImports { imports, report },
            ));
        }
        Err(e) => return Err(RunError::Resolver(e)),
    };
    info!(
        resolved = report.total_resolved,
        stubbed = report.total_stubbed,
        "imports resolved"
    );
    for dll in &report.dll_summaries {
        let non_implemented = dll
            .imports
            .iter()
            .filter(|entry| !matches!(entry.kind, resolver::ImportResolutionKind::Implemented))
            .map(|entry| format!("{}:{:?}", entry.name, entry.kind))
            .collect::<Vec<_>>();
        if !non_implemented.is_empty() {
            tracing::warn!(
                dll = dll.dll_name,
                imports = ?non_implemented,
                "non-implemented imports"
            );
        }
    }

    dev_emit!(_dev_channel, imports_resolved_event(&report));
    emit_registry_metrics(&registry);

    // Also attempt delay-load imports (currently just warns if present).
    let _ = resolver::resolve_delay_imports(&image, &parsed.pe, &registry);

    // 4. Set final memory protections on PE sections.
    image.protect(&parsed.pe)?;

    // 5a. Set the spoofed Windows version from config.
    subsys::version::init_version(app_config.windows_version);

    // 5aa. Initialize dialog policy from config.
    subsys::dialogs::init_policy(app_config.dialogs.clone(), app_config.windows_version);
    if let Some(policy) = subsys::dialogs::policy() {
        info!(
            theme = ?policy.theme,
            native_backend = ?policy.native_backend,
            windows_theme = ?policy.windows_theme,
            desktop = ?policy.desktop,
            "dialog policy initialized"
        );

        // SAFETY: still single-threaded before PE entry.
        set_var_if_absent(
            "RINE_DIALOG_THEME",
            subsys::dialogs::dialog_theme_env(policy.theme),
        );
        // Backward compatibility for in-flight integrations.
        set_var_if_absent(
            "RINE_DIALOG_MODE",
            match policy.theme {
                rine_types::config::DialogTheme::Native => "native",
                rine_types::config::DialogTheme::Windows => "emulated",
            },
        );
        set_var_if_absent(
            "RINE_DIALOG_NATIVE_BACKEND",
            subsys::dialogs::native_backend_env(policy.native_backend),
        );
        set_var_if_absent(
            "RINE_DIALOG_EMULATED_THEME",
            subsys::dialogs::windows_theme_env(policy.windows_theme),
        );
    }

    // 5b. Set up fake Windows Thread Environment Block (TEB) so CRT code
    //     that reads segment-based TEB fields doesn't fault.
    unsafe { subsys::threading::init_teb_for_format(parsed.format)? };

    // Emit a synthetic primary thread lifecycle for dashboards: even apps
    // that never call CreateThread execute on an implicit main thread.
    let main_tid = current_thread_id();
    let main_entry = image.base().as_usize() as u64 + parsed.pe.entry as u64;
    rine_types::dev_notify!(on_thread_created(-2, main_tid, main_entry));

    // 6. Execute the PE entry point.
    let exit_code = rine_common_kernel32::process::run_with_unhandled_exception_filter(|| {
        entry::execute(&image, &parsed)
    })?;
    rine_types::dev_notify!(on_thread_exited(main_tid, exit_code as u32));

    // ProcessExited + shutdown are normally handled by ExitProcess
    // (via the DevHook).  This is a fallback for PEs that return from
    // their entry point instead of calling ExitProcess.
    #[cfg(feature = "dev")]
    if let Some(bridge) = _dev_channel.as_ref() {
        emit_memory_snapshot_ready(bridge);
        let _ = bridge.send(&rine_channel::DevEvent::ProcessExited { exit_code });
        bridge.shutdown();
    }

    std::process::exit(exit_code);
}

/// Top-level error type wrapping all stages of PE loading and execution.
#[derive(Debug, thiserror::Error)]
pub enum RunError {
    #[error("{0}")]
    Config(#[source] ConfigError),

    #[error("{0}")]
    Probe(#[from] ProbeError),

    #[error("unsupported PE machine type: 0x{0:04x}")]
    UnsupportedMachine(u16),

    #[error("{0}")]
    Dispatch(#[from] DispatchError),

    #[error("failed to determine runtime executable path: {0}")]
    CurrentExe(#[source] std::io::Error),

    #[error("dynamic provider not found at {path} ({lookup})")]
    DynamicProviderNotFound {
        path: std::path::PathBuf,
        lookup: &'static str,
    },

    #[error("{0}")]
    Pe(#[from] PeError),

    #[error("{0}")]
    Loader(#[from] LoaderError),

    #[error("{0}")]
    Resolver(#[from] ResolverError),

    #[error("{0}")]
    Threading(#[from] crate::subsys::threading::ThreadingError),

    #[error("{0}")]
    Entry(#[from] EntryError),
}

#[derive(Debug, thiserror::Error)]
pub enum DispatchError {
    #[error(
        "this executable is 32-bit, but the 32-bit support component (rine32) could not be started: {source}\n\
         The rine installation may be incomplete or damaged. Try reinstalling rine."
    )]
    Spawn {
        helper: String,
        source: std::io::Error,
    },

    #[error(
        "this executable is 32-bit, but the 32-bit support component (rine32) was not found.\n\
         The rine installation may be incomplete or damaged. Try reinstalling rine."
    )]
    HelperNotFound,
}

fn dispatch_to_rine32(
    exe_path: &Path,
    #[allow(unused)] cli: &Cli,
    #[cfg(feature = "dev")] dev_bridge: Option<&DevBridge>,
) -> Result<std::convert::Infallible, DispatchError> {
    let window_host = match WindowHostSession::start() {
        Ok(session) => Some(session),
        Err(error) => {
            tracing::warn!(
                "failed to start x86 window host; continuing without native host: {error}"
            );
            None
        }
    };

    let helper_path = match resolve_rine32_helper_path() {
        Some(path) => path,
        None => return Err(DispatchError::HelperNotFound),
    };
    let helper = helper_path.display().to_string();

    let mut child = match spawn_rine32(
        &helper_path,
        exe_path,
        &cli.exe_args,
        window_host
            .as_ref()
            .map(|session| (WINDOW_HOST_SOCKET_ENV, session.socket_path())),
    ) {
        Ok(child) => child,
        Err(source) => {
            if let Some(session) = window_host {
                session.wait();
            }
            return Err(DispatchError::Spawn {
                helper: helper.clone(),
                source,
            });
        }
    };

    #[cfg(feature = "dev")]
    if let Some(bridge) = dev_bridge {
        // The x86 helper reuses the same dashboard socket. Release the parent
        // connection immediately so rine-dev can accept the follow-up stream
        // and surface ConfigLoaded/ImportsResolved before the PE starts running.
        bridge.shutdown();
    }

    let status = child.wait().map_err(|source| DispatchError::Spawn {
        helper: helper.clone(),
        source,
    })?;

    if let Some(session) = window_host {
        session.wait();
    }

    if status.success() {
        std::process::exit(0);
    }

    // Preserve the helper's exit semantics for x86 fixtures and real apps.
    // If the helper terminated via signal, fall back to a generic failure code.
    std::process::exit(status.code().unwrap_or(1))
}

fn resolve_rine32_helper_path() -> Option<std::path::PathBuf> {
    if let Some(explicit) = std::env::var_os("RINE_RINE32_HELPER") {
        return Some(explicit.into());
    }

    if let Ok(exe) = std::env::current_exe() {
        let sibling = exe.with_file_name("rine32");
        if sibling.is_file() {
            return Some(sibling);
        }
    }

    None
}

fn spawn_rine32(
    helper_path: &Path,
    exe_path: &Path,
    exe_args: &[String],
    window_host_socket: Option<(&str, &Path)>,
) -> Result<std::process::Child, std::io::Error> {
    let mut command = Command::new(helper_path);
    command.arg(exe_path).args(exe_args);
    if let Some(socket_path) = std::env::var_os("RINE_DEV_SOCKET") {
        command.env("RINE_DEV_SOCKET", socket_path);
    }
    if let Some((key, value)) = window_host_socket {
        command.env(key, value);
    }
    command.spawn()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_rine32_returns_error_for_missing_helper() {
        let helper = Path::new("/definitely/missing/rine32-helper-binary");
        let exe = Path::new("/tmp/fake-x86.exe");
        let args: Vec<String> = vec!["arg1".to_string()];

        let result = spawn_rine32(helper, exe, &args, None);
        assert!(result.is_err(), "missing helper should fail to spawn");
    }
}
