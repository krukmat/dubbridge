// S1-T4: storage crate — StorageAdapter trait, LocalFsAdapter, config, path helpers
pub mod adapter;
pub mod config;
pub mod error;
pub mod local;

pub use adapter::StorageAdapter;
pub use config::StorageConfig;
pub use error::StorageError;
pub use local::LocalFsAdapter;

use config::StorageConfig as Cfg;

/// Build the appropriate adapter from config.
/// Currently always returns LocalFsAdapter; S2 adds the MinIO/S3 variant.
pub fn build_adapter(config: &Cfg) -> Box<dyn StorageAdapter> {
    Box::new(LocalFsAdapter::new(&config.base_path))
}

// Path helpers — owned by this crate so key layout stays in one place (ADR-006).
pub fn asset_prefix(asset_id: &str) -> String {
    format!("assets/{asset_id}/")
}

pub fn recording_prefix(session_id: &str) -> String {
    format!("recordings/{session_id}/")
}

#[cfg(test)]
mod tests {
    use super::*;

    // T1-T3: ADR-006 storage key layout contract
    #[test]
    fn asset_prefix_format() {
        assert_eq!(asset_prefix("abc-123"), "assets/abc-123/");
    }

    #[test]
    fn asset_prefix_preserves_id_exactly() {
        let id = "550e8400-e29b-41d4-a716-446655440000";
        assert!(asset_prefix(id).starts_with("assets/"));
        assert!(asset_prefix(id).contains(id));
        assert!(asset_prefix(id).ends_with('/'));
    }

    #[test]
    fn recording_prefix_format() {
        assert_eq!(recording_prefix("session-xyz"), "recordings/session-xyz/");
    }

    #[test]
    fn recording_prefix_preserves_id_exactly() {
        let id = "session-550e8400";
        assert!(recording_prefix(id).starts_with("recordings/"));
        assert!(recording_prefix(id).contains(id));
        assert!(recording_prefix(id).ends_with('/'));
    }
}
