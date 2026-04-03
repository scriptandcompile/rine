use serde::Serialize;

/// Accumulated snapshot of the runtime state, served to the frontend on demand.
#[derive(Debug, Clone, Default, Serialize)]
pub struct StateSnapshot {
    pub pe: Option<PeInfo>,
    pub config: Option<ConfigInfo>,
    pub imports: Option<ImportsInfo>,
    pub handles: Vec<HandleInfo>,
    pub threads: Vec<ThreadInfo>,
    pub tls_slots: Vec<u32>,
    pub memory_regions: Vec<MemoryRegionInfo>,
    pub memory_current_usage: u64,
    pub memory_peak_usage: u64,
    pub memory_total_allocated: u64,
    pub memory_total_freed: u64,
    pub memory_snapshot: Option<MemorySnapshotInfo>,
    pub dialog_calls: Vec<DialogCallInfo>,
    pub exited: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PeInfo {
    pub exe_path: String,
    pub image_base: u64,
    pub image_size: u64,
    pub entry_rva: u64,
    pub relocation_delta: i64,
    pub sections: Vec<rine_channel::SectionInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigInfo {
    pub config_path: String,
    pub windows_version: String,
    pub environment_overrides: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportsInfo {
    pub summaries: Vec<rine_channel::DllSummary>,
    pub total_resolved: usize,
    pub total_stubbed: usize,
}

/// A tracked handle (open or closed).
#[derive(Debug, Clone, Serialize)]
pub struct HandleInfo {
    pub handle: i64,
    pub kind: String,
    pub detail: String,
    pub closed: bool,
}

/// A tracked thread (running or exited).
#[derive(Debug, Clone, Serialize)]
pub struct ThreadInfo {
    pub handle: i64,
    pub thread_id: u32,
    pub entry_point: u64,
    pub exit_code: Option<u32>,
}

/// A tracked memory region.
#[derive(Debug, Clone, Serialize)]
pub struct MemoryRegionInfo {
    pub address: u64,
    pub size: u64,
    pub source: String,
    pub freed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemorySnapshotInfo {
    pub json_path: String,
    pub bin_path: String,
    pub region_count: usize,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DialogCallInfo {
    pub phase: String,
    pub api: String,
    pub theme: String,
    pub native_backend: String,
    pub windows_theme: String,
    pub success: Option<bool>,
    pub error_code: Option<u32>,
    pub selected_path: Option<String>,
}
