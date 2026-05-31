use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerRequest {
    pub job_id: String,
    pub payload: serde_json::Value,
}
