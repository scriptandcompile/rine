use serde::Serialize;

/// Accumulated snapshot of the runtime state, served to the frontend on demand.
#[derive(Debug, Clone, Default, Serialize)]
pub struct StateSnapshot {
    pub pe: Option<PeInfo>,
    pub config: Option<ConfigInfo>,
    pub imports: Option<ImportsInfo>,
    pub exited: Option<i32>,
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
