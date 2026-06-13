// S-160-T1b: review domain entity per ADR-030
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    asset::AssetId,
    workspace::{OrgId, ProjectId},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewError {
    UnknownVerdict(String),
    UnknownTaskState(String),
    UnknownPublicationStatus(String),
}

impl std::fmt::Display for ReviewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownVerdict(s) => write!(f, "unknown review verdict: {s}"),
            Self::UnknownTaskState(s) => write!(f, "unknown review task state: {s}"),
            Self::UnknownPublicationStatus(s) => write!(f, "unknown publication status: {s}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReviewTaskId(pub Uuid);

impl ReviewTaskId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ReviewTaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ReviewTaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Immutable verdict values stored in review_decisions.verdict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewVerdict {
    Approved,
    Rejected,
}

impl std::fmt::Display for ReviewVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Approved => "approved",
            Self::Rejected => "rejected",
        };
        write!(f, "{s}")
    }
}

impl FromStr for ReviewVerdict {
    type Err = ReviewError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "approved" => Ok(Self::Approved),
            "rejected" => Ok(Self::Rejected),
            other => Err(ReviewError::UnknownVerdict(other.to_string())),
        }
    }
}

/// Derived state of a review task. Pending is represented by absence of decisions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewTaskState {
    Pending,
    Approved,
    Rejected,
}

impl ReviewTaskState {
    pub fn is_publishable(&self) -> bool {
        matches!(self, Self::Approved)
    }
}

impl std::fmt::Display for ReviewTaskState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
        };
        write!(f, "{s}")
    }
}

impl FromStr for ReviewTaskState {
    type Err = ReviewError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "approved" => Ok(Self::Approved),
            "rejected" => Ok(Self::Rejected),
            other => Err(ReviewError::UnknownTaskState(other.to_string())),
        }
    }
}

/// Persisted publication status values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicationStatus {
    Published,
}

impl std::fmt::Display for PublicationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Published => "published",
        };
        write!(f, "{s}")
    }
}

impl FromStr for PublicationStatus {
    type Err = ReviewError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "published" => Ok(Self::Published),
            other => Err(ReviewError::UnknownPublicationStatus(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewTask {
    pub id: ReviewTaskId,
    pub org_id: OrgId,
    pub project_id: ProjectId,
    pub asset_id: AssetId,
    pub target_language_id: Uuid,
    pub assignee_subject_id: Option<Uuid>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub assigned_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewDecisionRow {
    pub id: Uuid,
    pub review_task_id: ReviewTaskId,
    pub verdict: ReviewVerdict,
    pub comment: Option<String>,
    pub reviewer_subject_id: Uuid,
    pub happened_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicationRow {
    pub id: Uuid,
    pub review_task_id: ReviewTaskId,
    pub status: PublicationStatus,
    pub published_by: Uuid,
    pub published_at: OffsetDateTime,
}

/// Current review state = latest decision row by happened_at, fail-closed at decode boundary.
pub fn derive_review_state(rows: &[ReviewDecisionRow]) -> ReviewTaskState {
    rows.iter()
        .max_by_key(|row| row.happened_at)
        .map(|row| match row.verdict {
            ReviewVerdict::Approved => ReviewTaskState::Approved,
            ReviewVerdict::Rejected => ReviewTaskState::Rejected,
        })
        .unwrap_or(ReviewTaskState::Pending)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task_id() -> ReviewTaskId {
        ReviewTaskId::new()
    }

    fn actor() -> Uuid {
        Uuid::new_v4()
    }

    fn approved_row(task_id: ReviewTaskId) -> ReviewDecisionRow {
        ReviewDecisionRow {
            id: Uuid::new_v4(),
            review_task_id: task_id,
            verdict: ReviewVerdict::Approved,
            comment: Some("approved".to_string()),
            reviewer_subject_id: actor(),
            happened_at: OffsetDateTime::now_utc(),
        }
    }

    fn rejected_row(task_id: ReviewTaskId) -> ReviewDecisionRow {
        ReviewDecisionRow {
            id: Uuid::new_v4(),
            review_task_id: task_id,
            verdict: ReviewVerdict::Rejected,
            comment: Some("rejected".to_string()),
            reviewer_subject_id: actor(),
            happened_at: OffsetDateTime::now_utc(),
        }
    }

    // HP-1: latest persisted decision approved -> ReviewTaskState::Approved
    #[test]
    fn hp1_latest_approved_derives_approved_state() {
        let row = approved_row(task_id());
        assert_eq!(
            derive_review_state(std::slice::from_ref(&row)),
            ReviewTaskState::Approved
        );
        assert!(derive_review_state(&[row]).is_publishable());
    }

    // HP-2: no decisions -> ReviewTaskState::Pending
    #[test]
    fn hp2_no_decisions_derives_pending_state() {
        let state = derive_review_state(&[]);
        assert_eq!(state, ReviewTaskState::Pending);
        assert!(!state.is_publishable());
    }

    // EC-1: unknown verdict string fails closed
    #[test]
    fn ec1_unknown_verdict_fails_closed() {
        let err = ReviewVerdict::from_str("pending").unwrap_err();
        assert!(matches!(err, ReviewError::UnknownVerdict(_)));
    }

    // EC-2: latest rejected decision -> Rejected and not publishable
    #[test]
    fn ec2_latest_rejected_is_not_publishable() {
        let id = task_id();
        let approved = approved_row(id);
        std::thread::sleep(std::time::Duration::from_millis(1));
        let rejected = rejected_row(id);
        let state = derive_review_state(&[approved, rejected]);
        assert_eq!(state, ReviewTaskState::Rejected);
        assert!(!state.is_publishable());
    }

    // EC-3: unknown persisted publication state fails closed
    #[test]
    fn ec3_unknown_publication_state_fails_closed() {
        let err = PublicationStatus::from_str("queued").unwrap_err();
        assert!(matches!(err, ReviewError::UnknownPublicationStatus(_)));
    }

    #[test]
    fn review_task_state_display_all_variants() {
        assert_eq!(ReviewTaskState::Pending.to_string(), "pending");
        assert_eq!(ReviewTaskState::Approved.to_string(), "approved");
        assert_eq!(ReviewTaskState::Rejected.to_string(), "rejected");
    }

    #[test]
    fn review_verdict_display_matches_migration_values() {
        assert_eq!(ReviewVerdict::Approved.to_string(), "approved");
        assert_eq!(ReviewVerdict::Rejected.to_string(), "rejected");
    }

    #[test]
    fn publication_status_display_matches_migration_values() {
        assert_eq!(PublicationStatus::Published.to_string(), "published");
    }

    #[test]
    fn review_task_state_parse_known_variants_succeeds() {
        assert_eq!(
            ReviewTaskState::from_str("pending").unwrap(),
            ReviewTaskState::Pending
        );
        assert_eq!(
            ReviewTaskState::from_str("approved").unwrap(),
            ReviewTaskState::Approved
        );
        assert_eq!(
            ReviewTaskState::from_str("rejected").unwrap(),
            ReviewTaskState::Rejected
        );
    }

    #[test]
    fn review_task_state_unknown_variant_fails_closed() {
        let err = ReviewTaskState::from_str("in_review").unwrap_err();
        assert!(matches!(err, ReviewError::UnknownTaskState(_)));
    }

    #[test]
    fn review_error_display_includes_unknown_values() {
        assert!(
            ReviewError::UnknownVerdict("pending".into())
                .to_string()
                .contains("pending")
        );
        assert!(
            ReviewError::UnknownTaskState("in_review".into())
                .to_string()
                .contains("in_review")
        );
        assert!(
            ReviewError::UnknownPublicationStatus("queued".into())
                .to_string()
                .contains("queued")
        );
    }
}
