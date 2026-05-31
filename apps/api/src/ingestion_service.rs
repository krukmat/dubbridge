// H1-T1: thin re-export — finalization logic lives in the app-neutral
// crates/ingestion boundary consumable by both apps/api and apps/worker-runner.
pub use dubbridge_ingestion::{IngestionServiceError, finalize_ingestion_core};
