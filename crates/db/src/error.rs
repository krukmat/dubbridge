// T3: S1 db errors — typed so API layer can map to correct HTTP status
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("database connection failed: {0}")]
    ConnectionFailed(#[from] sqlx::Error),

    #[error("record not found")]
    NotFound,

    #[error("query failed: {0}")]
    QueryFailed(sqlx::Error),

    // H1-T2: fail-closed decoding — unknown persisted governance values must never silently
    // coerce to a fallback variant (ADR-008).
    #[error("unknown stored value in {field}: {value}")]
    UnknownStoredValue { field: &'static str, value: String },
}
