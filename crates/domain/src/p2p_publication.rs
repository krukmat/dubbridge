// MVP0-P2P P2.T1a: pure publication identity and lifecycle contract (ADR-044/O4).
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum P2pPublicationError {
    #[error("unknown P2P publication state: {0}")]
    UnknownState(String),
    #[error("invalid P2P publication transition: {from} -> {to}")]
    InvalidTransition {
        from: PublicationState,
        to: PublicationState,
    },
    #[error("ready requires durable external confirmation for the same K1 lineage")]
    MissingReadyConfirmation,
    #[error("ready confirmation lineage does not match the publication lineage")]
    ConfirmationLineageMismatch,
}

/// Stable logical identity for one P2P publication/package lineage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct P2pPublicationId(pub Uuid);

impl P2pPublicationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for P2pPublicationId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for P2pPublicationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Stable K1 lineage identifier. Retries/reconciliation must preserve this value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct K1LineageId(pub Uuid);

impl K1LineageId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for K1LineageId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for K1LineageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Authoritative semantic publication state frozen by P2.T0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicationState {
    Building,
    PublishPending,
    Publishing,
    Reconciling,
    Ready,
    Failed,
}

impl PublicationState {
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Ready | Self::Failed)
    }

    pub fn is_ready(self) -> bool {
        self == Self::Ready
    }

    fn permits_non_ready_transition(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Building, Self::PublishPending)
                | (Self::Building, Self::Failed)
                | (Self::PublishPending, Self::Publishing)
                | (Self::PublishPending, Self::Failed)
                | (Self::Publishing, Self::Reconciling)
                | (Self::Publishing, Self::Failed)
                | (Self::Reconciling, Self::Publishing)
                | (Self::Reconciling, Self::Failed)
        )
    }
}

impl std::fmt::Display for PublicationState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Building => "building",
            Self::PublishPending => "publish_pending",
            Self::Publishing => "publishing",
            Self::Reconciling => "reconciling",
            Self::Ready => "ready",
            Self::Failed => "failed",
        };
        write!(f, "{value}")
    }
}

impl FromStr for PublicationState {
    type Err = P2pPublicationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "building" => Ok(Self::Building),
            "publish_pending" => Ok(Self::PublishPending),
            "publishing" => Ok(Self::Publishing),
            "reconciling" => Ok(Self::Reconciling),
            "ready" => Ok(Self::Ready),
            "failed" => Ok(Self::Failed),
            other => Err(P2pPublicationError::UnknownState(other.to_owned())),
        }
    }
}

/// Pure aggregate used to enforce same-lineage lifecycle rules before persistence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct P2pPublicationLifecycle {
    pub id: P2pPublicationId,
    pub lineage_id: K1LineageId,
    pub state: PublicationState,
}

impl P2pPublicationLifecycle {
    pub fn new(id: P2pPublicationId, lineage_id: K1LineageId) -> Self {
        Self {
            id,
            lineage_id,
            state: PublicationState::Building,
        }
    }

    /// Apply a lifecycle transition. `Ready` is special: it requires durable
    /// external-publication confirmation bound to this exact K1 lineage.
    pub fn transition(
        &mut self,
        next: PublicationState,
        confirmed_lineage: Option<K1LineageId>,
    ) -> Result<(), P2pPublicationError> {
        if self.state.is_terminal() {
            return Err(P2pPublicationError::InvalidTransition {
                from: self.state,
                to: next,
            });
        }

        if next == PublicationState::Ready {
            let confirmed = confirmed_lineage.ok_or(P2pPublicationError::MissingReadyConfirmation)?;
            if confirmed != self.lineage_id {
                return Err(P2pPublicationError::ConfirmationLineageMismatch);
            }
            if !matches!(self.state, PublicationState::Publishing | PublicationState::Reconciling) {
                return Err(P2pPublicationError::InvalidTransition {
                    from: self.state,
                    to: next,
                });
            }
            self.state = PublicationState::Ready;
            return Ok(());
        }

        if !self.state.permits_non_ready_transition(next) {
            return Err(P2pPublicationError::InvalidTransition {
                from: self.state,
                to: next,
            });
        }

        self.state = next;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lifecycle() -> P2pPublicationLifecycle {
        P2pPublicationLifecycle::new(P2pPublicationId::new(), K1LineageId::new())
    }

    #[test]
    fn hp_t1a_1_identity_and_lineage_survive_allowed_transitions() {
        let mut publication = lifecycle();
        let original_id = publication.id;
        let original_lineage = publication.lineage_id;

        publication
            .transition(PublicationState::PublishPending, None)
            .unwrap();
        publication
            .transition(PublicationState::Publishing, None)
            .unwrap();
        publication
            .transition(PublicationState::Reconciling, None)
            .unwrap();
        publication
            .transition(PublicationState::Publishing, None)
            .unwrap();

        assert_eq!(publication.id, original_id);
        assert_eq!(publication.lineage_id, original_lineage);
    }

    #[test]
    fn hp_t1a_2_unknown_outcome_is_reconciling_and_non_ready() {
        let mut publication = lifecycle();
        publication
            .transition(PublicationState::PublishPending, None)
            .unwrap();
        publication
            .transition(PublicationState::Publishing, None)
            .unwrap();
        publication
            .transition(PublicationState::Reconciling, None)
            .unwrap();

        assert_eq!(publication.state, PublicationState::Reconciling);
        assert!(!publication.state.is_ready());
    }

    #[test]
    fn ec_t1a_1_ready_requires_same_lineage_confirmation() {
        let mut publication = lifecycle();
        publication
            .transition(PublicationState::PublishPending, None)
            .unwrap();
        publication
            .transition(PublicationState::Publishing, None)
            .unwrap();

        assert_eq!(
            publication.transition(PublicationState::Ready, None),
            Err(P2pPublicationError::MissingReadyConfirmation)
        );
        assert_eq!(
            publication.transition(PublicationState::Ready, Some(K1LineageId::new())),
            Err(P2pPublicationError::ConfirmationLineageMismatch)
        );

        let lineage = publication.lineage_id;
        publication
            .transition(PublicationState::Ready, Some(lineage))
            .unwrap();
        assert!(publication.state.is_ready());
    }

    #[test]
    fn ec_t1a_2_terminal_states_do_not_regress() {
        let mut ready = lifecycle();
        ready.transition(PublicationState::PublishPending, None).unwrap();
        ready.transition(PublicationState::Publishing, None).unwrap();
        let ready_lineage = ready.lineage_id;
        ready
            .transition(PublicationState::Ready, Some(ready_lineage))
            .unwrap();
        assert!(ready
            .transition(PublicationState::Reconciling, None)
            .is_err());

        let mut failed = lifecycle();
        failed.transition(PublicationState::Failed, None).unwrap();
        assert!(failed
            .transition(PublicationState::PublishPending, None)
            .is_err());
    }

    #[test]
    fn ec_t1a_3_illegal_direct_ready_is_rejected_even_with_matching_lineage() {
        let mut publication = lifecycle();
        let lineage = publication.lineage_id;
        assert_eq!(
            publication.transition(PublicationState::Ready, Some(lineage)),
            Err(P2pPublicationError::InvalidTransition {
                from: PublicationState::Building,
                to: PublicationState::Ready,
            })
        );
    }

    #[test]
    fn all_non_ready_transition_pairs_follow_the_frozen_state_machine() {
        use PublicationState::*;

        let allowed = [
            (Building, PublishPending),
            (Building, Failed),
            (PublishPending, Publishing),
            (PublishPending, Failed),
            (Publishing, Reconciling),
            (Publishing, Failed),
            (Reconciling, Publishing),
            (Reconciling, Failed),
        ];
        let states = [Building, PublishPending, Publishing, Reconciling, Ready, Failed];

        for from in states {
            for to in states {
                if to == Ready {
                    continue;
                }
                assert_eq!(
                    from.permits_non_ready_transition(to),
                    allowed.contains(&(from, to)),
                    "unexpected transition rule for {from} -> {to}"
                );
            }
        }
    }

    #[test]
    fn state_round_trips_through_storage_tokens() {
        for state in [
            PublicationState::Building,
            PublicationState::PublishPending,
            PublicationState::Publishing,
            PublicationState::Reconciling,
            PublicationState::Ready,
            PublicationState::Failed,
        ] {
            assert_eq!(PublicationState::from_str(&state.to_string()).unwrap(), state);
        }
    }
}
