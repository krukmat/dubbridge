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
}
