use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[cfg(not(target_pointer_width = "32"))]
compile_error!(
    "crate `rine32` must be built for a 32-bit target (for example: --target i686-unknown-linux-gnu)"
);

use clap::{CommandFactory, Parser};
use goblin::Object;
use rine_channel::DevEvent;
use rine_dev_bridge::{DevBridge, imports_resolved_event, pe_loaded_event};
use rine_dlls::DllRegistry;
use rine_runtime_core::loader::{entry, memory, resolver};
use rine_runtime_core::loader::{entry::EntryError, memory::LoaderError, resolver::ResolverError};
use rine_runtime_core::pe::parser::{ParsedPe, PeError, PeFormat};
use rine_types::config::{
    AppConfig, ConfigError, DialogTheme, EmulatedDialogTheme, NativeDialogBackend, WindowsVersion,
};
use rine_types::os::{VersionInfo, set_version};
use rine32_advapi32::Advapi32Plugin32;
use rine32_comdlg32::Comdlg32Plugin32;
use rine32_gdi32::Gdi32Plugin32;
use rine32_kernel32::Kernel32Plugin32;
use rine32_msvcrt::{CrtForwarderPlugin32, MsvcrtPlugin32};
use rine32_ntdll::NtdllPlugin32;
use rine32_user32::User32Plugin32;
use thiserror::Error;
use tracing::{error, info, warn};

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

const IMAGE_FILE_MACHINE_I386: u16 = 0x014c;

#[cfg(target_arch = "x86")]
#[repr(C)]
struct UserDesc {
    entry_number: u32,
    base_addr: u32,
    limit: u32,
    flags: u32,
}

#[cfg(target_arch = "x86")]
const USER_DESC_SEG_32BIT: u32 = 1 << 0;
#[cfg(target_arch = "x86")]
const USER_DESC_LIMIT_IN_PAGES: u32 = 1 << 4;
#[cfg(target_arch = "x86")]
const USER_DESC_USEABLE: u32 = 1 << 6;

macro_rules! dev_emit {
    ($bridge:expr, $event:expr) => {
        if let Some(bridge) = $bridge.as_ref() {
            let _ = bridge.send(&$event);
        }
    };
}

fn current_thread_id() -> u32 {
    // Linux thread ID is the closest runtime identifier to Win32 thread ID.
    unsafe { libc::syscall(libc::SYS_gettid) as u32 }
}

#[derive(Parser, Debug)]
#[command(
    name = "rine32",
    version,
    about = "rine32 - 32-bit Windows PE runtime launcher",
    override_usage = "rine32 <EXE_PATH> [EXE_ARGS]..."
)]
struct Cli {
    /// Path to the Windows .exe to run.
    exe_path: Option<PathBuf>,

    /// Show or create the per-app config file instead of running the exe.
    #[arg(long = "config")]
    show_config: bool,

    /// Arguments to pass to the Windows executable.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    exe_args: Vec<String>,
}

#[derive(Debug, Error)]
enum Run32Error {
    #[error("failed to read executable `{path}`: {source}")]
    Io {
        source: std::io::Error,
        path: PathBuf,
    },

    #[error("failed to parse executable `{path}`: {source}")]
    Parse {
        source: goblin::error::Error,
        path: PathBuf,
    },

    #[error("`{path}` is not a PE executable")]
    NotPe { path: PathBuf },

    #[error("`{path}` is not a 32-bit PE (machine=0x{machine:04x})")]
    NotPe32 { path: PathBuf, machine: u16 },

    #[error("{0}")]
    Config(#[from] ConfigError),

    #[error("{0}")]
    Pe(#[from] PeError),

    #[error("{0}")]
    Loader(#[from] LoaderError),

    #[error("{0}")]
    Resolver(#[from] ResolverError),

    #[error("{0}")]
    Entry(#[from] EntryError),

    #[error("failed to initialize 32-bit thread environment block")]
    TebInit,
}

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_writer(std::io::stderr)
        .init();

    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => e.exit(),
    };

    let Some(exe_path) = cli.exe_path else {
        let _ = Cli::command().print_help();
        return ExitCode::FAILURE;
    };

    if cli.show_config {
        return show_config(&exe_path);
    }

    match run(&exe_path, &cli.exe_args) {
        Ok(code) => ExitCode::from(code as u8),
        Err(e) => {
            error!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn run(exe_path: &Path, exe_args: &[String]) -> Result<i32, Run32Error> {
    let dev_bridge = DevBridge::init_from_env("rine32");

    let resolved = resolve_exe_path(exe_path);
    ensure_pe32(&resolved)?;
    let app_config = rine_types::config::load_config(exe_path)?;

    info!(
        version = %app_config.windows_version,
        config = %rine_types::config::config_path(exe_path).display(),
        "app config loaded"
    );

    dev_emit!(
        dev_bridge,
        DevEvent::ConfigLoaded {
            config_path: rine_types::config::config_path(exe_path)
                .display()
                .to_string(),
            windows_version: app_config.windows_version.to_string(),
            environment_overrides: app_config
                .environment
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        }
    );

    for (key, value) in &app_config.environment {
        // SAFETY: invoked before PE execution while runtime is single-threaded.
        unsafe { std::env::set_var(key, value) };
    }

    apply_dialog_policy_env(&app_config);

    let mut registry = DllRegistry::new_lazy();
    registry.register_plugin_factory(|| Box::new(Kernel32Plugin32));
    registry.register_plugin_factory(|| Box::new(Advapi32Plugin32));
    registry.register_plugin_factory(|| Box::new(Comdlg32Plugin32));
    registry.register_plugin_factory(|| Box::new(Gdi32Plugin32));
    registry.register_plugin_factory(|| Box::new(User32Plugin32));
    registry.register_plugin_factory(|| Box::new(MsvcrtPlugin32));
    registry.register_plugin_factory(|| Box::new(CrtForwarderPlugin32));
    registry.register_plugin_factory(|| Box::new(NtdllPlugin32));
    emit_registry_metrics(&registry);

    info!(
        exe = %resolved.display(),
        arg_count = exe_args.len(),
        known_dlls = registry.known_dlls().len(),
        "validated 32-bit executable"
    );

    let parsed = ParsedPe::load(&resolved)?;
    if parsed.format != PeFormat::Pe32 {
        return Err(Run32Error::NotPe32 {
            path: resolved,
            machine: parsed.pe.header.coff_header.machine,
        });
    }

    init_version_from_config(app_config.windows_version);

    let image = memory::LoadedImage::load(&parsed)?;
    dev_emit!(dev_bridge, pe_loaded_event(&resolved, &parsed, &image));

    let report = match resolver::resolve_imports(&image, &parsed.pe, parsed.format, &registry) {
        Ok(report) => report,
        Err(ResolverError::UnimplementedImports { imports, report }) => {
            emit_registry_metrics(&registry);
            dev_emit!(dev_bridge, imports_resolved_event(&report));
            return Err(Run32Error::Resolver(ResolverError::UnimplementedImports {
                imports,
                report,
            }));
        }
        Err(e) => return Err(Run32Error::Resolver(e)),
    };
    info!(
        resolved = report.total_resolved,
        stubbed = report.total_stubbed,
        "imports resolved"
    );
    for dll in &report.dll_summaries {
        if !dll.stubbed_names.is_empty() {
            warn!(
                dll = dll.dll_name,
                stubs = ?dll.stubbed_names,
                "stubbed imports"
            );
        }
    }

    dev_emit!(dev_bridge, imports_resolved_event(&report));
    emit_registry_metrics(&registry);

    let _ = resolver::resolve_delay_imports(&image, &parsed.pe, &registry);
    image.protect(&parsed.pe)?;

    unsafe { init_teb_for_pe32().map_err(|_| Run32Error::TebInit)? };

    // Emit synthetic primary-thread lifecycle telemetry for apps that never
    // call CreateThread but still run on an implicit main thread.
    let main_tid = current_thread_id();
    let main_entry = image.base().as_usize() as u64 + parsed.pe.entry as u64;
    rine_types::dev_notify!(on_thread_created(-2, main_tid, main_entry));

    let exit_code = rine_common_kernel32::process::run_with_unhandled_exception_filter(|| {
        entry::execute(&image, &parsed)
    })?;
    rine_types::dev_notify!(on_thread_exited(main_tid, exit_code as u32));
    dev_emit!(dev_bridge, DevEvent::ProcessExited { exit_code });
    Ok(exit_code)
}

fn init_version_from_config(ver: WindowsVersion) {
    let (major, minor, build) = ver.version_triple();

    let (sp_major, sp_minor, csd) = match ver {
        WindowsVersion::WinXP => (3, 0, "Service Pack 3"),
        WindowsVersion::Win7 => (1, 0, "Service Pack 1"),
        WindowsVersion::Win10 | WindowsVersion::Win11 => (0, 0, ""),
    };

    set_version(VersionInfo {
        major,
        minor,
        build,
        service_pack_major: sp_major,
        service_pack_minor: sp_minor,
        csd_version: csd.into(),
    });
}

unsafe fn init_teb_for_pe32() -> Result<(), ()> {
    // Allocate a fake TEB and PEB for CRT reads during startup.
    const TEB_SIZE: usize = 0x1000;
    const PEB_SIZE: usize = 0x1000;
    const TEB_STACK_BASE: usize = 0x04;
    const TEB_STACK_LIMIT: usize = 0x08;
    const TEB_SELF: usize = 0x18;
    const TEB_PEB: usize = 0x30;

    let teb_layout = std::alloc::Layout::from_size_align(TEB_SIZE, 16).map_err(|_| ())?;
    let peb_layout = std::alloc::Layout::from_size_align(PEB_SIZE, 16).map_err(|_| ())?;

    let teb = unsafe { std::alloc::alloc_zeroed(teb_layout) };
    if teb.is_null() {
        return Err(());
    }

    let peb = unsafe { std::alloc::alloc_zeroed(peb_layout) };
    if peb.is_null() {
        return Err(());
    }

    #[cfg(target_arch = "x86")]
    {
        let stack_base: u32;
        unsafe {
            core::arch::asm!("mov {}, esp", out(reg) stack_base);
        }
        let stack_base = stack_base.saturating_add(0x100000) & !0xFFF;
        let stack_limit = stack_base.saturating_sub(0x200000);

        unsafe {
            std::ptr::write(teb.add(TEB_STACK_BASE) as *mut u32, stack_base);
            std::ptr::write(teb.add(TEB_STACK_LIMIT) as *mut u32, stack_limit);
            std::ptr::write(teb.add(TEB_SELF) as *mut u32, teb as u32);
            std::ptr::write(teb.add(TEB_PEB) as *mut u32, peb as u32);
        }

        let mut user_desc = UserDesc {
            entry_number: u32::MAX,
            base_addr: teb as u32,
            limit: 0xFFFFF,
            flags: USER_DESC_SEG_32BIT | USER_DESC_LIMIT_IN_PAGES | USER_DESC_USEABLE,
        };

        let ret = unsafe {
            libc::syscall(
                libc::SYS_set_thread_area,
                &mut user_desc as *mut UserDesc as *mut libc::c_void,
            )
        };
        if ret != 0 {
            return Err(());
        }

        let selector = ((user_desc.entry_number << 3) | 0x3) as u16;
        unsafe {
            core::arch::asm!("mov fs, ax", in("ax") selector, options(nostack, preserves_flags));
        }

        return Ok(());
    }

    #[allow(unreachable_code)]
    Err(())
}

fn resolve_exe_path(path: &Path) -> PathBuf {
    if !path.exists() && path.extension().is_none() {
        path.with_extension("exe")
    } else {
        path.to_path_buf()
    }
}

fn ensure_pe32(path: &Path) -> Result<(), Run32Error> {
    let bytes = std::fs::read(path).map_err(|source| Run32Error::Io {
        source,
        path: path.to_path_buf(),
    })?;

    let pe = match Object::parse(&bytes).map_err(|source| Run32Error::Parse {
        source,
        path: path.to_path_buf(),
    })? {
        Object::PE(pe) => pe,
        _ => {
            return Err(Run32Error::NotPe {
                path: path.to_path_buf(),
            });
        }
    };

    let machine = pe.header.coff_header.machine;
    if machine != IMAGE_FILE_MACHINE_I386 {
        return Err(Run32Error::NotPe32 {
            path: path.to_path_buf(),
            machine,
        });
    }

    Ok(())
}

fn show_config(exe_path: &Path) -> ExitCode {
    let cfg = match rine_types::config::load_config(exe_path) {
        Ok(c) => c,
        Err(e) => {
            error!("{e}");
            return ExitCode::FAILURE;
        }
    };

    let path = rine_types::config::config_path(exe_path);
    if !path.exists()
        && let Err(e) = rine_types::config::save_config(exe_path, &cfg)
    {
        error!("{e}");
        return ExitCode::FAILURE;
    }

    let abs_exe = exe_path
        .canonicalize()
        .unwrap_or_else(|_| exe_path.to_path_buf());

    let config_bin = std::env::current_exe()
        .ok()
        .and_then(|p| {
            let sibling = p.with_file_name("rine-config");
            sibling.is_file().then_some(sibling)
        })
        .unwrap_or_else(|| PathBuf::from("rine-config"));

    info!(
        "launching {} for {}",
        config_bin.display(),
        abs_exe.display()
    );

    match std::process::Command::new(&config_bin)
        .arg(abs_exe.as_os_str())
        .spawn()
    {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            error!(
                "failed to launch rine-config ({}): {e}\n\
                 hint: make sure rine-config is built (`cargo build -p rine-config`)",
                config_bin.display()
            );
            ExitCode::FAILURE
        }
    }
}

fn apply_dialog_policy_env(cfg: &AppConfig) {
    let desktop = detect_desktop();
    let native_backend = resolve_native_backend(cfg.dialogs.native_backend, desktop);
    let windows_theme = resolve_emulated_theme(cfg.windows_version);

    set_var_if_absent("RINE_DIALOG_THEME", dialog_theme_env(cfg.dialogs.theme));
    set_var_if_absent(
        "RINE_DIALOG_MODE",
        match cfg.dialogs.theme {
            DialogTheme::Native => "native",
            DialogTheme::Windows => "emulated",
        },
    );
    set_var_if_absent(
        "RINE_DIALOG_NATIVE_BACKEND",
        native_backend_env(native_backend),
    );
    set_var_if_absent(
        "RINE_DIALOG_EMULATED_THEME",
        windows_theme_env(windows_theme),
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DesktopEnvironment {
    Gnome,
    Kde,
    Other,
}

fn detect_desktop() -> DesktopEnvironment {
    let desktop = std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .unwrap_or_default()
        .to_ascii_lowercase();

    if desktop.contains("gnome") {
        return DesktopEnvironment::Gnome;
    }
    if desktop.contains("kde") || std::env::var_os("KDE_FULL_SESSION").is_some() {
        return DesktopEnvironment::Kde;
    }
    DesktopEnvironment::Other
}

fn resolve_native_backend(
    backend: NativeDialogBackend,
    desktop: DesktopEnvironment,
) -> NativeDialogBackend {
    match backend {
        NativeDialogBackend::Auto => match desktop {
            DesktopEnvironment::Gnome | DesktopEnvironment::Kde | DesktopEnvironment::Other => {
                NativeDialogBackend::Portal
            }
        },
        explicit => explicit,
    }
}

fn resolve_emulated_theme(windows_version: WindowsVersion) -> EmulatedDialogTheme {
    match windows_version {
        WindowsVersion::WinXP => EmulatedDialogTheme::Xp,
        WindowsVersion::Win7 => EmulatedDialogTheme::Win7,
        WindowsVersion::Win10 => EmulatedDialogTheme::Win10,
        WindowsVersion::Win11 => EmulatedDialogTheme::Win11,
    }
}

fn dialog_theme_env(theme: DialogTheme) -> &'static str {
    match theme {
        DialogTheme::Native => "native",
        DialogTheme::Windows => "windows",
    }
}

fn native_backend_env(backend: NativeDialogBackend) -> &'static str {
    match backend {
        NativeDialogBackend::Auto => "auto",
        NativeDialogBackend::Portal => "portal",
        NativeDialogBackend::Gtk => "gtk",
        NativeDialogBackend::Kde => "kde",
    }
}

fn windows_theme_env(theme: EmulatedDialogTheme) -> &'static str {
    match theme {
        EmulatedDialogTheme::Auto => "auto",
        EmulatedDialogTheme::Xp => "xp",
        EmulatedDialogTheme::Win7 => "win7",
        EmulatedDialogTheme::Win10 => "win10",
        EmulatedDialogTheme::WindowsVersion => "windows_version",
        EmulatedDialogTheme::Win11 => "win11",
    }
}

fn set_var_if_absent(key: &str, value: &str) {
    if std::env::var_os(key).is_none() {
        // SAFETY: invoked before PE execution while runtime is single-threaded.
        unsafe { std::env::set_var(key, value) };
    }
}
