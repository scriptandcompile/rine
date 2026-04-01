mod receiver;
mod sender;

pub use receiver::DevReceiver;
pub use sender::DevSender;

use serde::{Deserialize, Serialize};

/// Events sent from rine → rine-dev over the Unix domain socket.
///
/// Protocol: 4-byte little-endian length prefix + UTF-8 JSON payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DevEvent {
    PeLoaded {
        exe_path: String,
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
