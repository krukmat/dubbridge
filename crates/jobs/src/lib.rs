use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparationJob {
    pub asset_id: Uuid,
    pub source_artifact_id: Uuid,
    pub ingest_token: Uuid,
}

impl PreparationJob {
    pub const JOB_TYPE: &str = "media_preparation";

    pub fn new(asset_id: Uuid, source_artifact_id: Uuid, ingest_token: Uuid) -> Self {
        Self {
            asset_id,
            source_artifact_id,
            ingest_token,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobEnvelope<T> {
    pub job_type: String,
    pub payload: T,
}

impl<T> JobEnvelope<T> {
    pub fn new(job_type: impl Into<String>, payload: T) -> Self {
        Self {
            job_type: job_type.into(),
            payload,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueueError {
    Unavailable(String),
}

impl std::fmt::Display for QueueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for QueueError {}

pub trait PreparationJobQueue: Send + Sync {
    fn enqueue(&self, job: PreparationJob) -> Result<(), QueueError>;
}

pub type SharedPreparationJobQueue = Arc<dyn PreparationJobQueue>;

#[derive(Debug, Default)]
pub struct InMemoryPreparationJobQueue {
    jobs: Mutex<Vec<PreparationJob>>,
}

impl InMemoryPreparationJobQueue {
    pub fn queued_jobs(&self) -> Vec<PreparationJob> {
        self.jobs
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl PreparationJobQueue for InMemoryPreparationJobQueue {
    fn enqueue(&self, job: PreparationJob) -> Result<(), QueueError> {
        self.jobs
            .lock()
            .map_err(|_| QueueError::Unavailable("queue lock poisoned".into()))?
            .push(job);
        Ok(())
    }
}

pub fn default_queue() -> &'static str {
    "dubbridge.default"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_memory_queue_records_jobs() {
        let queue = InMemoryPreparationJobQueue::default();
        let job = PreparationJob::new(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());

        queue.enqueue(job.clone()).expect("enqueue");

        assert_eq!(queue.queued_jobs(), vec![job]);
    }

    #[test]
    fn job_envelope_wraps_payload_with_type() {
        let payload = PreparationJob::new(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        let envelope = JobEnvelope::new(PreparationJob::JOB_TYPE, payload.clone());

        assert_eq!(envelope.job_type, PreparationJob::JOB_TYPE);
        assert_eq!(envelope.payload, payload);
    }
}
