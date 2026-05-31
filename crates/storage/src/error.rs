// S1-T4: storage error types
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("object not found: {key}")]
    NotFound { key: String },

    #[error("io error on key '{key}': {source}")]
    Io {
        key: String,
        #[source]
        source: std::io::Error,
    },

    #[error("storage backend error: {0}")]
    Backend(String),
}
