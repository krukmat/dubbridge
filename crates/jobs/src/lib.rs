use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobEnvelope {
    pub job_type: String,
}

pub fn default_queue() -> &'static str {
    "dubbridge.default"
}
