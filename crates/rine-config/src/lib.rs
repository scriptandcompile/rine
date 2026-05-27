use serde::Serialize;

#[derive(Serialize)]
pub struct VersionOption {
    pub value: serde_json::Value,
    pub label: String,
}
