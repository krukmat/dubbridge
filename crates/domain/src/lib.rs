// T1: S1 domain — public API surface for dubbridge-domain crate
// S3-T1: recording module added
// S3-P1: platform_ingest module added (types in sub-task 3)
pub mod artifact;
pub mod asset;
pub mod audit;
pub mod consent;
pub mod haa;
pub mod ingestion;
pub mod p2p_publication;
pub mod p2p_recovery;
pub mod platform_ingest;
pub mod playback;
pub mod recording;
pub mod review;
pub mod rights;
pub mod workspace;
