use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::QueueError;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubtitlePostReadyRoute {
    #[serde(rename = "legacy_subtitle_review_v1")]
    #[default]
    LegacySubtitleReviewV1,
    #[serde(rename = "s150_localization_v1")]
    S150LocalizationV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubtitleJob {
    pub asset_id: Uuid,
    pub project_id: Uuid,
    pub target_language: String,
    #[serde(default)]
    pub post_ready_route: SubtitlePostReadyRoute,
}

impl SubtitleJob {
    pub const JOB_TYPE: &str = "subtitle_generation";

    pub fn new(asset_id: Uuid, project_id: Uuid, target_language: impl Into<String>) -> Self {
        Self {
            asset_id,
            project_id,
            target_language: target_language.into(),
            post_ready_route: SubtitlePostReadyRoute::LegacySubtitleReviewV1,
        }
    }

    pub fn new_s150_localization(
        asset_id: Uuid,
        project_id: Uuid,
        target_language: impl Into<String>,
    ) -> Self {
        Self {
            asset_id,
            project_id,
            target_language: target_language.into(),
            post_ready_route: SubtitlePostReadyRoute::S150LocalizationV1,
        }
    }
}

/// Stable namespace for initial S-150 translation request identities.
pub const S150_INITIAL_TRANSLATION_NAMESPACE: Uuid =
    Uuid::from_u128(0x794aa6e0_28e8_4b50_9dc0_48a7cfafaa2a);

/// Derive the replay-stable initial translation request ID for one subtitle artifact.
pub fn initial_translation_generation_request_id(subtitle_artifact_id: Uuid) -> Uuid {
    let name = format!(
        "initial-translation-v1:{}",
        subtitle_artifact_id.hyphenated().to_string().to_lowercase()
    );
    Uuid::new_v5(&S150_INITIAL_TRANSLATION_NAMESPACE, name.as_bytes())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranslationJob {
    pub project_id: Uuid,
    pub asset_id: Uuid,
    pub target_language_id: Uuid,
    pub source_subtitle_artifact_id: Uuid,
    pub generation_request_id: Uuid,
}

impl TranslationJob {
    pub const JOB_TYPE: &str = "translation_generation";

    pub fn new(
        project_id: Uuid,
        asset_id: Uuid,
        target_language_id: Uuid,
        source_subtitle_artifact_id: Uuid,
        generation_request_id: Uuid,
    ) -> Self {
        Self {
            project_id,
            asset_id,
            target_language_id,
            source_subtitle_artifact_id,
            generation_request_id,
        }
    }
}

pub type SharedTranslationJobQueue = Arc<dyn TranslationJobQueue>;

#[async_trait]
pub trait TranslationJobQueue: Send + Sync {
    async fn enqueue(&self, job: TranslationJob) -> Result<(), QueueError>;
}

#[derive(Debug, Default)]
pub struct InMemoryTranslationJobQueue {
    jobs: Mutex<Vec<TranslationJob>>,
}

impl InMemoryTranslationJobQueue {
    pub fn queued_jobs(&self) -> Vec<TranslationJob> {
        self.jobs
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

#[async_trait]
impl TranslationJobQueue for InMemoryTranslationJobQueue {
    async fn enqueue(&self, job: TranslationJob) -> Result<(), QueueError> {
        self.jobs
            .lock()
            .map_err(|_| QueueError::Unavailable("translation queue lock poisoned".into()))?
            .push(job);
        Ok(())
    }
}

pub type SharedSubtitleJobQueue = Arc<dyn SubtitleJobQueue>;

#[async_trait]
pub trait SubtitleJobQueue: Send + Sync {
    async fn enqueue(&self, job: SubtitleJob) -> Result<(), QueueError>;
}

#[derive(Debug, Default)]
pub struct InMemorySubtitleJobQueue {
    jobs: Mutex<Vec<SubtitleJob>>,
}

impl InMemorySubtitleJobQueue {
    pub fn queued_jobs(&self) -> Vec<SubtitleJob> {
        self.jobs
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

#[async_trait]
impl SubtitleJobQueue for InMemorySubtitleJobQueue {
    async fn enqueue(&self, job: SubtitleJob) -> Result<(), QueueError> {
        self.jobs
            .lock()
            .map_err(|_| QueueError::Unavailable("subtitle queue lock poisoned".into()))?
            .push(job);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subtitle_job_type_constant() {
        assert_eq!(SubtitleJob::JOB_TYPE, "subtitle_generation");
    }

    #[test]
    fn in_memory_subtitle_queue_records_jobs() {
        let queue = InMemorySubtitleJobQueue::default();
        let job = SubtitleJob::new(Uuid::new_v4(), Uuid::new_v4(), "en");

        tokio_test::block_on(async { queue.enqueue(job.clone()).await.expect("enqueue") });

        assert_eq!(queue.queued_jobs(), vec![job]);
    }

    #[test]
    fn in_memory_subtitle_queue_empty_by_default() {
        let queue = InMemorySubtitleJobQueue::default();
        assert!(queue.queued_jobs().is_empty());
    }

    #[test]
    fn legacy_subtitle_job_json_defaults_to_legacy_route() {
        let asset_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();
        let job: SubtitleJob = serde_json::from_value(serde_json::json!({
            "asset_id": asset_id,
            "project_id": project_id,
            "target_language": "en"
        }))
        .expect("legacy job JSON should decode");

        assert_eq!(
            job.post_ready_route,
            SubtitlePostReadyRoute::LegacySubtitleReviewV1
        );
        assert_eq!(
            SubtitleJob::new(asset_id, project_id, "en").post_ready_route,
            SubtitlePostReadyRoute::LegacySubtitleReviewV1
        );
    }

    #[test]
    fn unknown_subtitle_post_ready_route_fails_deserialization() {
        let result: Result<SubtitleJob, _> = serde_json::from_value(serde_json::json!({
            "asset_id": Uuid::new_v4(),
            "project_id": Uuid::new_v4(),
            "target_language": "en",
            "post_ready_route": "unexpected_route"
        }));

        assert!(result.is_err(), "unknown routes must fail closed");
    }

    #[test]
    fn localization_subtitle_job_serializes_its_versioned_route() {
        let job = SubtitleJob::new_s150_localization(Uuid::new_v4(), Uuid::new_v4(), "es");
        let value = serde_json::to_value(job).expect("serialize localization job");

        assert_eq!(value["post_ready_route"], "s150_localization_v1");
    }

    #[test]
    fn translation_job_serializes_the_full_durable_identity() {
        let job = TranslationJob::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        );
        let value = serde_json::to_value(&job).expect("serialize translation job");

        assert_eq!(value["project_id"], job.project_id.to_string());
        assert_eq!(value["asset_id"], job.asset_id.to_string());
        assert_eq!(
            value["target_language_id"],
            job.target_language_id.to_string()
        );
        assert_eq!(
            value["source_subtitle_artifact_id"],
            job.source_subtitle_artifact_id.to_string()
        );
        assert_eq!(
            value["generation_request_id"],
            job.generation_request_id.to_string()
        );
    }

    #[test]
    fn in_memory_translation_queue_records_jobs() {
        let queue = InMemoryTranslationJobQueue::default();
        let job = TranslationJob::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        );

        tokio_test::block_on(async { queue.enqueue(job.clone()).await.expect("enqueue") });

        assert_eq!(queue.queued_jobs(), vec![job]);
    }

    #[test]
    fn initial_translation_request_id_is_deterministic_and_canonical() {
        let subtitle_artifact_id =
            Uuid::parse_str("A0C51F57-08D6-4D8B-88F1-FB244C503059").expect("valid UUID");
        let expected = Uuid::new_v5(
            &S150_INITIAL_TRANSLATION_NAMESPACE,
            b"initial-translation-v1:a0c51f57-08d6-4d8b-88f1-fb244c503059",
        );

        assert_eq!(
            initial_translation_generation_request_id(subtitle_artifact_id),
            expected
        );
        assert_eq!(
            initial_translation_generation_request_id(subtitle_artifact_id),
            initial_translation_generation_request_id(subtitle_artifact_id)
        );
    }
}
