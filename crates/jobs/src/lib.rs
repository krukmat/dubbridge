use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod subtitle_job;

pub use subtitle_job::{
    InMemorySubtitleJobQueue, InMemoryTranslationJobQueue, S150_INITIAL_TRANSLATION_NAMESPACE,
    SharedSubtitleJobQueue, SharedTranslationJobQueue, SubtitleJob, SubtitleJobQueue,
    TranslationJob, TranslationJobQueue, initial_translation_generation_request_id,
};

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

#[async_trait]
pub trait PreparationJobQueue: Send + Sync {
    async fn enqueue(&self, job: PreparationJob) -> Result<(), QueueError>;
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

#[async_trait]
impl PreparationJobQueue for InMemoryPreparationJobQueue {
    async fn enqueue(&self, job: PreparationJob) -> Result<(), QueueError> {
        self.jobs
            .lock()
            .map_err(|_| QueueError::Unavailable("queue lock poisoned".into()))?
            .push(job);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranscriptionJob {
    pub asset_id: Uuid,
    pub source_artifact_id: Uuid,
    pub source_language: String,
}

impl TranscriptionJob {
    pub const JOB_TYPE: &str = "asr_transcription";

    pub fn new(
        asset_id: Uuid,
        source_artifact_id: Uuid,
        source_language: impl Into<String>,
    ) -> Self {
        Self {
            asset_id,
            source_artifact_id,
            source_language: source_language.into(),
        }
    }
}

pub type SharedTranscriptionJobQueue = Arc<dyn TranscriptionJobQueue>;

#[async_trait]
pub trait TranscriptionJobQueue: Send + Sync {
    async fn enqueue(&self, job: TranscriptionJob) -> Result<(), QueueError>;
}

#[derive(Debug, Default)]
pub struct InMemoryTranscriptionJobQueue {
    jobs: Mutex<Vec<TranscriptionJob>>,
}

impl InMemoryTranscriptionJobQueue {
    pub fn queued_jobs(&self) -> Vec<TranscriptionJob> {
        self.jobs
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

#[async_trait]
impl TranscriptionJobQueue for InMemoryTranscriptionJobQueue {
    async fn enqueue(&self, job: TranscriptionJob) -> Result<(), QueueError> {
        self.jobs
            .lock()
            .map_err(|_| QueueError::Unavailable("transcription queue lock poisoned".into()))?
            .push(job);
        Ok(())
    }
}

// ---- Redis-backed queues ----

/// Upper bound on establishing a Redis connection.
///
/// redis-rs's `ConnectionManager` has no default connection timeout, so an
/// unreachable or blackholed Redis would otherwise hang the caller forever.
/// Bounding it here keeps the queue fail-closed: an unavailable backend
/// surfaces as `QueueError::Unavailable` in bounded time.
const REDIS_CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

macro_rules! define_redis_job_queue {
    ($queue_ty:ident, $job_ty:ty, $queue_trait:ident) => {
        pub struct $queue_ty {
            storage: apalis_redis::RedisStorage<$job_ty>,
        }

        impl $queue_ty {
            pub async fn connect(redis_url: &str) -> Result<Self, QueueError> {
                let conn =
                    tokio::time::timeout(REDIS_CONNECT_TIMEOUT, apalis_redis::connect(redis_url))
                        .await
                        .map_err(|_| {
                            QueueError::Unavailable("redis connect timed out".to_string())
                        })?
                        .map_err(|e| {
                            QueueError::Unavailable(format!("redis connect failed: {e}"))
                        })?;
                let config = apalis_redis::Config::default().set_namespace(<$job_ty>::JOB_TYPE);
                let storage = apalis_redis::RedisStorage::new_with_config(conn, config);
                Ok(Self { storage })
            }

            /// Enqueue and return the apalis task id.
            ///
            /// The queue trait deliberately returns `()`, so this inherent
            /// method exists to let tests prove the job is retrievable from the
            /// same Redis namespace a consumer would poll. The trait method
            /// delegates here, so both paths exercise the same push.
            pub async fn enqueue_with_id(
                &self,
                job: $job_ty,
            ) -> Result<apalis::prelude::TaskId, QueueError> {
                use apalis::prelude::Storage;
                let mut storage = self.storage.clone();
                storage
                    .push(job)
                    .await
                    .map(|parts| parts.task_id)
                    .map_err(|e| QueueError::Unavailable(format!("redis enqueue failed: {e}")))
            }

            /// Owned clone of the underlying apalis backend, for attaching
            /// this queue's namespace to a `WorkerBuilder` as a consumer.
            pub fn backend(&self) -> apalis_redis::RedisStorage<$job_ty> {
                self.storage.clone()
            }
        }

        #[async_trait]
        impl $queue_trait for $queue_ty {
            async fn enqueue(&self, job: $job_ty) -> Result<(), QueueError> {
                self.enqueue_with_id(job).await.map(|_| ())
            }
        }
    };
}

define_redis_job_queue!(
    RedisPreparationJobQueue,
    PreparationJob,
    PreparationJobQueue
);
define_redis_job_queue!(
    RedisTranscriptionJobQueue,
    TranscriptionJob,
    TranscriptionJobQueue
);
define_redis_job_queue!(RedisSubtitleJobQueue, SubtitleJob, SubtitleJobQueue);

pub fn default_queue() -> &'static str {
    "dubbridge.default"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn redis_url_for_test_from_env(redis_url: Option<String>) -> String {
        redis_url
            .expect("DUBBRIDGE_REDIS_URL must be set when running ignored Redis integration tests")
    }

    fn redis_url_for_test() -> String {
        redis_url_for_test_from_env(std::env::var("DUBBRIDGE_REDIS_URL").ok())
    }

    #[test]
    fn in_memory_queue_records_jobs() {
        let queue = InMemoryPreparationJobQueue::default();
        let job = PreparationJob::new(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());

        tokio_test::block_on(async { queue.enqueue(job.clone()).await.expect("enqueue") });

        assert_eq!(queue.queued_jobs(), vec![job]);
    }

    #[test]
    fn transcription_job_type_constant() {
        assert_eq!(TranscriptionJob::JOB_TYPE, "asr_transcription");
    }

    #[test]
    fn in_memory_transcription_queue_records_jobs() {
        let queue = InMemoryTranscriptionJobQueue::default();
        let job = TranscriptionJob::new(Uuid::new_v4(), Uuid::new_v4(), "en");

        tokio_test::block_on(async { queue.enqueue(job.clone()).await.expect("enqueue") });

        assert_eq!(queue.queued_jobs(), vec![job]);
    }

    #[test]
    fn subtitle_job_contract_remains_publicly_reexported() {
        let asset_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();
        let job = SubtitleJob::new(asset_id, project_id, "en");

        assert_eq!(job.asset_id, asset_id);
        assert_eq!(SubtitleJob::JOB_TYPE, "subtitle_generation");
        assert_eq!(
            initial_translation_generation_request_id(asset_id),
            Uuid::new_v5(
                &S150_INITIAL_TRANSLATION_NAMESPACE,
                format!(
                    "initial-translation-v1:{}",
                    asset_id.hyphenated().to_string().to_lowercase()
                )
                .as_bytes(),
            )
        );
    }

    #[test]
    fn job_envelope_wraps_payload_with_type() {
        let payload = PreparationJob::new(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        let envelope = JobEnvelope::new(PreparationJob::JOB_TYPE, payload.clone());

        assert_eq!(envelope.job_type, PreparationJob::JOB_TYPE);
        assert_eq!(envelope.payload, payload);
    }

    #[test]
    fn redis_url_for_test_accepts_explicit_value() {
        let url = "redis://127.0.0.1:6379/15".to_string();
        assert_eq!(redis_url_for_test_from_env(Some(url.clone())), url);
    }

    #[test]
    fn redis_url_for_test_requires_explicit_env_for_ignored_tests() {
        let panic = std::panic::catch_unwind(|| redis_url_for_test_from_env(None))
            .expect_err("missing redis url should panic");
        let message = if let Some(message) = panic.downcast_ref::<&str>() {
            *message
        } else if let Some(message) = panic.downcast_ref::<String>() {
            message.as_str()
        } else {
            panic!("unexpected panic payload type");
        };

        assert_eq!(
            message,
            "DUBBRIDGE_REDIS_URL must be set when running ignored Redis integration tests"
        );
    }

    #[tokio::test]
    #[ignore = "requires DUBBRIDGE_REDIS_URL; run explicitly via qa-test-redis"]
    async fn redis_preparation_queue_connects_and_enqueues() {
        let url = redis_url_for_test();
        let queue = RedisPreparationJobQueue::connect(&url)
            .await
            .expect("redis connect");
        let job = PreparationJob::new(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        assert!(queue.enqueue(job).await.is_ok());
    }

    #[tokio::test]
    #[ignore = "requires DUBBRIDGE_REDIS_URL; run explicitly via qa-test-redis"]
    async fn redis_transcription_queue_connects_and_enqueues() {
        let url = redis_url_for_test();
        let queue = RedisTranscriptionJobQueue::connect(&url)
            .await
            .expect("redis connect");
        let job = TranscriptionJob::new(Uuid::new_v4(), Uuid::new_v4(), "en");
        assert!(queue.enqueue(job).await.is_ok());
    }

    #[tokio::test]
    #[ignore = "requires DUBBRIDGE_REDIS_URL; run explicitly via qa-test-redis"]
    async fn redis_subtitle_queue_connects_and_enqueues() {
        let url = redis_url_for_test();
        let queue = RedisSubtitleJobQueue::connect(&url)
            .await
            .expect("redis connect");
        let job = SubtitleJob::new(Uuid::new_v4(), Uuid::new_v4(), "en");
        assert!(queue.enqueue(job).await.is_ok());
    }

    #[tokio::test]
    async fn redis_queue_fails_closed_on_malformed_url() {
        // Deterministic and infra-free: a URL with no redis:// scheme cannot
        // even be parsed into connection info, so connect must reject it.
        let result = RedisPreparationJobQueue::connect("not-a-redis-url").await;
        assert!(
            matches!(result, Err(QueueError::Unavailable(_))),
            "expected QueueError::Unavailable for a malformed url"
        );
    }

    #[tokio::test]
    async fn redis_queue_fails_closed_on_unreachable_server() {
        use std::time::Duration;

        // Port 1 is reserved and never accepts connections. connect must give
        // up on its own bound rather than hanging, so the outer ceiling here
        // only exists to fail the test loudly if the bound regresses.
        let outcome = tokio::time::timeout(
            REDIS_CONNECT_TIMEOUT + Duration::from_secs(10),
            RedisPreparationJobQueue::connect("redis://127.0.0.1:1"),
        )
        .await
        .expect("connect to an unreachable server hung past its own timeout");

        assert!(
            matches!(outcome, Err(QueueError::Unavailable(_))),
            "expected QueueError::Unavailable for an unreachable server"
        );
    }

    #[tokio::test]
    #[ignore = "requires DUBBRIDGE_REDIS_URL; run explicitly via qa-test-redis"]
    async fn redis_enqueued_job_is_retrievable_from_its_namespace() {
        use apalis::prelude::Storage;

        let url = redis_url_for_test();

        let queue = RedisPreparationJobQueue::connect(&url)
            .await
            .expect("prep connect");
        let job = PreparationJob::new(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        let task_id = queue.enqueue_with_id(job.clone()).await.expect("enqueue");

        // Independent connection into the same namespace: this is exactly what
        // an apalis worker polls, so retrieval here proves the round trip.
        let conn = apalis_redis::connect(url.as_str())
            .await
            .expect("probe connect");
        let config = apalis_redis::Config::default().set_namespace(PreparationJob::JOB_TYPE);
        let mut probe: apalis_redis::RedisStorage<PreparationJob> =
            apalis_redis::RedisStorage::new_with_config(conn, config);

        let fetched = probe.fetch_by_id(&task_id).await.expect("fetch");
        let fetched = fetched.expect("job present in its own namespace");
        assert_eq!(fetched.args, job);
    }

    #[tokio::test]
    #[ignore = "requires DUBBRIDGE_REDIS_URL; run explicitly via qa-test-redis"]
    async fn redis_queues_use_distinct_namespaces() {
        use apalis::prelude::Storage;

        assert_ne!(PreparationJob::JOB_TYPE, SubtitleJob::JOB_TYPE);
        assert_ne!(PreparationJob::JOB_TYPE, TranscriptionJob::JOB_TYPE);
        assert_ne!(TranscriptionJob::JOB_TYPE, SubtitleJob::JOB_TYPE);

        let url = redis_url_for_test();

        let prep_queue = RedisPreparationJobQueue::connect(&url)
            .await
            .expect("prep connect");
        let job = PreparationJob::new(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        let task_id = prep_queue.enqueue_with_id(job).await.expect("enqueue");

        // The subtitle namespace must not see the preparation job.
        let conn = apalis_redis::connect(url.as_str())
            .await
            .expect("probe connect");
        let config = apalis_redis::Config::default().set_namespace(SubtitleJob::JOB_TYPE);
        let mut other: apalis_redis::RedisStorage<SubtitleJob> =
            apalis_redis::RedisStorage::new_with_config(conn, config);

        let cross = other.fetch_by_id(&task_id).await;
        assert!(
            cross.is_err(),
            "preparation job leaked into the subtitle namespace"
        );
    }
}
