use std::io;
use std::path::Path;
use std::sync::{Arc, Mutex};

use rine_channel::{DevEvent, DevSender, DllSummary, SectionInfo};
use rine_runtime_core::loader::{memory::LoadedImage, resolver::ResolutionReport};
use rine_runtime_core::pe::parser::{ParsedPe, PeFormat};
use rine_types::dev_hooks::{self, DevHook, DialogOpenTelemetry, DialogResultTelemetry};
use tracing::{info, warn};

pub trait DevEventSink: Send + Sync {
    fn send_event(&self, event: DevEvent);
}

pub trait DevBridgeObserver: Send + Sync + 'static {
    fn on_memory_allocated(
        &self,
        _sink: &dyn DevEventSink,
        _address: u64,
        _size: u64,
        _source: &str,
    ) {
    }

    fn on_memory_freed(&self, _sink: &dyn DevEventSink, _address: u64, _size: u64, _source: &str) {}

    fn on_process_exiting(&self, _sink: &dyn DevEventSink, _exit_code: i32) {}
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NoopObserver;

impl DevBridgeObserver for NoopObserver {}

#[derive(Clone)]
pub struct DevBridge {
    sender: Arc<Mutex<DevSender>>,
}

impl DevEventSink for DevBridge {
    fn send_event(&self, event: DevEvent) {
        let _ = self.send(&event);
    }
}

impl DevBridge {
    pub fn init_from_env(runtime_name: &str) -> Option<Self> {
        Self::init_from_env_with_observer(runtime_name, NoopObserver)
    }

    pub fn init_from_env_with_observer<O>(runtime_name: &str, observer: O) -> Option<Self>
    where
        O: DevBridgeObserver,
    {
        let socket_path = std::env::var("RINE_DEV_SOCKET").ok()?;
        match Self::connect(std::path::Path::new(&socket_path)) {
            Ok(bridge) => {
                info!(runtime = runtime_name, "connected to rine-dev dashboard");
                bridge.install_global_hook(observer);
                Some(bridge)
            }
            Err(e) => {
                warn!(runtime = runtime_name, "failed to connect to rine-dev: {e}");
                None
            }
        }
    }

    pub fn connect(socket_path: &Path) -> io::Result<Self> {
        let sender = DevSender::connect(socket_path)?;
        Ok(Self {
            sender: Arc::new(Mutex::new(sender)),
        })
    }

    pub fn install_global_hook<O>(&self, observer: O)
    where
        O: DevBridgeObserver,
    {
        let hook = ChannelDevHook {
            bridge: self.clone(),
            observer: Arc::new(observer),
        };
        if dev_hooks::set_dev_hook(Box::new(hook)).is_err() {
            warn!("dev hook already installed; skipping duplicate bridge hook registration");
        }
    }

    pub fn send(&self, event: &DevEvent) -> io::Result<()> {
        if let Ok(mut sender) = self.sender.lock() {
            sender.send(event)
        } else {
            Err(io::Error::other("failed to lock dev sender"))
        }
    }

    pub fn shutdown(&self) {
        if let Ok(mut sender) = self.sender.lock() {
            sender.shutdown();
        }
    }
}

struct ChannelDevHook {
    bridge: DevBridge,
    observer: Arc<dyn DevBridgeObserver>,
}

impl DevHook for ChannelDevHook {
    fn on_handle_created(&self, handle: i64, kind: &str, detail: &str) {
        self.bridge.send_event(DevEvent::HandleCreated {
            handle,
            kind: kind.to_owned(),
            detail: detail.to_owned(),
        });
    }

    fn on_handle_closed(&self, handle: i64) {
        self.bridge.send_event(DevEvent::HandleClosed { handle });
    }

    fn on_thread_created(&self, handle: i64, thread_id: u32, entry_point: u64) {
        self.bridge.send_event(DevEvent::ThreadCreated {
            handle,
            thread_id,
            entry_point,
        });
    }

    fn on_thread_exited(&self, thread_id: u32, exit_code: u32) {
        self.bridge.send_event(DevEvent::ThreadExited {
            thread_id,
            exit_code,
        });
    }

    fn on_tls_allocated(&self, index: u32) {
        self.bridge.send_event(DevEvent::TlsAllocated { index });
    }

    fn on_tls_freed(&self, index: u32) {
        self.bridge.send_event(DevEvent::TlsFreed { index });
    }

    fn on_memory_allocated(&self, address: u64, size: u64, source: &str) {
        self.observer
            .on_memory_allocated(&self.bridge, address, size, source);
        self.bridge.send_event(DevEvent::MemoryAllocated {
            address,
            size,
            source: source.to_owned(),
        });
    }

    fn on_memory_freed(&self, address: u64, size: u64, source: &str) {
        self.observer
            .on_memory_freed(&self.bridge, address, size, source);
        self.bridge.send_event(DevEvent::MemoryFreed {
            address,
            size,
            source: source.to_owned(),
        });
    }

    fn on_dialog_opened(&self, opened: DialogOpenTelemetry<'_>) {
        self.bridge.send_event(DevEvent::DialogOpened {
            api: opened.api.to_owned(),
            theme: opened.theme.to_owned(),
            native_backend: opened.native_backend.to_owned(),
            windows_theme: opened.windows_theme.to_owned(),
        });
    }

    fn on_dialog_result(&self, result: DialogResultTelemetry<'_>) {
        self.bridge.send_event(DevEvent::DialogResult {
            api: result.api.to_owned(),
            theme: result.theme.to_owned(),
            native_backend: result.native_backend.to_owned(),
            windows_theme: result.windows_theme.to_owned(),
            success: result.success,
            error_code: result.error_code,
            selected_path: result.selected_path.map(str::to_owned),
        });
    }

    fn on_process_exiting(&self, exit_code: i32) {
        self.observer.on_process_exiting(&self.bridge, exit_code);
        self.bridge
            .send_event(DevEvent::ProcessExited { exit_code });
        self.bridge.shutdown();
    }
}

pub fn pe_loaded_event(exe_path: &Path, parsed: &ParsedPe, image: &LoadedImage) -> DevEvent {
    DevEvent::PeLoaded {
        exe_path: exe_path.display().to_string(),
        architecture: architecture_name(parsed.format).to_owned(),
        image_base: image.base().as_usize() as u64,
        image_size: image.size() as u64,
        entry_rva: parsed.pe.entry as u64,
        relocation_delta: image.base().as_usize() as i64 - parsed.pe.image_base as i64,
        sections: parsed
            .pe
            .sections
            .iter()
            .map(|section| SectionInfo {
                name: String::from_utf8_lossy(&section.name)
                    .trim_end_matches('\0')
                    .to_string(),
                virtual_address: section.virtual_address as u64,
                virtual_size: section.virtual_size as u64,
                characteristics: section.characteristics,
            })
            .collect(),
    }
}

pub fn imports_resolved_event(report: &ResolutionReport) -> DevEvent {
    DevEvent::ImportsResolved {
        summaries: report
            .dll_summaries
            .iter()
            .map(|dll| DllSummary {
                dll_name: dll.dll_name.clone(),
                resolved: dll.resolved,
                stubbed: dll.stubbed,
                resolved_names: dll.resolved_names.clone(),
                stubbed_names: dll.stubbed_names.clone(),
            })
            .collect(),
        total_resolved: report.total_resolved,
        total_stubbed: report.total_stubbed,
    }
}

fn architecture_name(format: PeFormat) -> &'static str {
    match format {
        PeFormat::Pe32 => "32-bit (PE32 / x86)",
        PeFormat::Pe32Plus => "64-bit (PE32+ / x64)",
    }
}
