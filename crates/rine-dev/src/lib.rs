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
