mod receiver;
mod sender;
mod window_host;

pub use receiver::DevReceiver;
pub use sender::DevSender;
pub use window_host::{
    HostWindowCommand, HostWindowEvent, HostWindowReceiver, HostWindowRect, HostWindowSender,
};

use serde::{Deserialize, Serialize};

/// Events sent from rine → rine-dev over the Unix domain socket.
///
/// Protocol: 4-byte little-endian length prefix + UTF-8 JSON payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DevEvent {
    PeLoaded {
        exe_path: String,
        architecture: String,
        image_base: u64,
        image_size: u64,
        entry_rva: u64,
        relocation_delta: i64,
        sections: Vec<SectionInfo>,
    },
    ConfigLoaded {
        config_path: String,
        windows_version: String,
        environment_overrides: Vec<(String, String)>,
    },
    ImportsResolved {
        summaries: Vec<DllSummary>,
        total_resolved: usize,
        total_stubbed: usize,
    },
    // ── Handle & thread tracking (phase 2) ───────────────────────
    HandleCreated {
        handle: i64,
        kind: String,
        detail: String,
    },
    HandleClosed {
        handle: i64,
    },
    ThreadCreated {
        handle: i64,
        thread_id: u32,
        entry_point: u64,
    },
    ThreadExited {
        thread_id: u32,
        exit_code: u32,
    },
    TlsAllocated {
        index: u32,
    },
    TlsFreed {
        index: u32,
    },
    MemoryAllocated {
        address: u64,
        size: u64,
        source: String,
    },
    MemoryFreed {
        address: u64,
        size: u64,
        source: String,
    },
    MemorySnapshotReady {
        json_path: String,
        bin_path: String,
        region_count: usize,
        total_bytes: u64,
    },
    DialogOpened {
        api: String,
        theme: String,
        native_backend: String,
        windows_theme: String,
    },
    DialogResult {
        api: String,
        theme: String,
        native_backend: String,
        windows_theme: String,
        success: bool,
        error_code: u32,
        selected_path: Option<String>,
    },
    // ── Lifecycle ────────────────────────────────────────────────
    ProcessExited {
        exit_code: i32,
    },
    OutputData {
        stream: OutputStream,
        data: String,
    },
}

/// Which output stream a piece of data came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputStream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionInfo {
    pub name: String,
    pub virtual_address: u64,
    pub virtual_size: u64,
    pub characteristics: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DllSummary {
    pub dll_name: String,
    pub resolved: usize,
    pub stubbed: usize,
    pub stubbed_names: Vec<String>,
    pub resolved_names: Vec<String>,
}
