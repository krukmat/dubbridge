//! This module is pure (no IO/clock/crypto). Decisions are the caller's PostgreSQL-state snapshot only.
//! It never consults or is aware of any queue/message-broker. Convergence to the
//! correct terminal action must hold with or without one (D3/O4, HP-T4-2).

use crate::p2p_publication::{K1LineageId, PublicationState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchOutcome {
    Unknown,
    Success,
    Failure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DispatchAttempt {
    pub lease_held: bool,
    pub attempts: u32,
    pub max_attempts: u32,
    pub last_outcome: Option<DispatchOutcome>,
    pub confirmed_lineage: Option<K1LineageId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryAction {
    NoOp,
    ClaimAndDispatch,
    WaitForLease,
    Retry,
    MarkFailed,
    ConfirmReady,
}

pub fn decide_recovery_action(
    state: PublicationState,
    lineage_id: K1LineageId,
    attempt: DispatchAttempt,
) -> RecoveryAction {
    // 1. If state is terminal, never take further action.
    if state.is_terminal() {
        return RecoveryAction::NoOp;
    }

    // 2. If last was Success and the confirmed lineage matches exactly, confirm ready.
    if attempt.last_outcome == Some(DispatchOutcome::Success)
        && attempt.confirmed_lineage == Some(lineage_id)
    {
        return RecoveryAction::ConfirmReady;
    }

    // 3. If a lease is currently held, wait for it to expire before dispatching again.
    if attempt.lease_held {
        return RecoveryAction::WaitForLease;
    }

    // 4. Determine retryability based on last outcome.
    let retryable = matches!(
        attempt.last_outcome,
        None | Some(DispatchOutcome::Unknown) | Some(DispatchOutcome::Failure),
    );

    if retryable && attempt.attempts == 0 {
        return RecoveryAction::ClaimAndDispatch;
    } else if retryable && attempt.attempts < attempt.max_attempts {
        return RecoveryAction::Retry;
    } else if retryable {
        return RecoveryAction::MarkFailed;
    }

    // 5. Anomalous/defensive case: last_outcome is Success but lineage didn't match step 2.
    // Never fabricate Ready without proof; retry instead.
    RecoveryAction::Retry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hp_t4a_1_no_lease_no_prior_attempt_claims_and_dispatches() {
        let lineage = K1LineageId::new();
        let attempt = DispatchAttempt {
            lease_held: false,
            attempts: 0,
            max_attempts: 3,
            last_outcome: None,
            confirmed_lineage: None,
        };
        let state = PublicationState::Building;
        assert_eq!(
            decide_recovery_action(state, lineage, attempt),
            RecoveryAction::ClaimAndDispatch
        );
    }

    #[test]
    fn hp_t4a_2_reconciler_recovers_without_queue_after_lease_expires() {
        let lineage = K1LineageId::new();
        let attempt = DispatchAttempt {
            lease_held: false,
            attempts: 1,
            max_attempts: 3,
            last_outcome: Some(DispatchOutcome::Failure),
            confirmed_lineage: None,
        };
        let state = PublicationState::PublishPending;
        assert_eq!(
            decide_recovery_action(state, lineage, attempt),
            RecoveryAction::Retry
        );
    }

    #[test]
    fn ec_t4a_1_unknown_outcome_is_retried_then_marked_failed_at_budget() {
        let lineage = K1LineageId::new();
        let attempt1 = DispatchAttempt {
            lease_held: false,
            max_attempts: 2,
            last_outcome: Some(DispatchOutcome::Unknown),
            confirmed_lineage: None,
            attempts: 1,
        };
        let attempt2 = DispatchAttempt {
            lease_held: false,
            max_attempts: 2,
            last_outcome: Some(DispatchOutcome::Unknown),
            confirmed_lineage: None,
            attempts: 2,
        };
        assert_eq!(
            decide_recovery_action(PublicationState::Reconciling, lineage, attempt1),
            RecoveryAction::Retry
        );
        assert_eq!(
            decide_recovery_action(PublicationState::Reconciling, lineage, attempt2),
            RecoveryAction::MarkFailed
        );
        // Neither call should return ConfirmReady
        assert_ne!(
            decide_recovery_action(PublicationState::Reconciling, lineage, attempt1),
            RecoveryAction::ConfirmReady
        );
        assert_ne!(
            decide_recovery_action(PublicationState::Reconciling, lineage, attempt2),
            RecoveryAction::ConfirmReady
        );
    }

    #[test]
    fn ec_t4a_2_duplicate_after_ready_is_idempotent_noop() {
        let lineage = K1LineageId::new();
        let attempt = DispatchAttempt {
            lease_held: false,
            attempts: 5,
            max_attempts: 3,
            last_outcome: Some(DispatchOutcome::Success),
            confirmed_lineage: Some(lineage),
        };
        let state = PublicationState::Ready;
        assert_eq!(
            decide_recovery_action(state, lineage, attempt),
            RecoveryAction::NoOp
        );
    }

    #[test]
    fn failed_state_is_also_terminal_and_takes_no_action() {
        let lineage = K1LineageId::new();
        let attempt = DispatchAttempt {
            lease_held: true,
            attempts: 0,
            max_attempts: 3,
            last_outcome: None,
            confirmed_lineage: None,
        };
        let state = PublicationState::Failed;
        assert_eq!(
            decide_recovery_action(state, lineage, attempt),
            RecoveryAction::NoOp
        );
    }

    #[test]
    fn confirm_ready_requires_lineage_to_match_exactly() {
        let lineage = K1LineageId::new();
        let other_lineage = K1LineageId::new();
        let attempt = DispatchAttempt {
            lease_held: false,
            attempts: 1,
            max_attempts: 3,
            last_outcome: Some(DispatchOutcome::Success),
            confirmed_lineage: Some(other_lineage),
        };
        let state = PublicationState::Publishing;
        assert_eq!(
            decide_recovery_action(state, lineage, attempt),
            RecoveryAction::Retry
        );
        // Now with matching lineage
        assert_eq!(
            decide_recovery_action(
                state,
                lineage,
                DispatchAttempt {
                    lease_held: false,
                    attempts: 1,
                    max_attempts: 3,
                    last_outcome: Some(DispatchOutcome::Success),
                    confirmed_lineage: Some(lineage),
                }
            ),
            RecoveryAction::ConfirmReady
        );
    }

    #[test]
    fn lease_held_wins_over_retry_even_with_budget_remaining() {
        let lineage = K1LineageId::new();
        let attempt = DispatchAttempt {
            lease_held: true,
            attempts: 1,
            max_attempts: 3,
            last_outcome: None,
            confirmed_lineage: None,
        };
        let state = PublicationState::Publishing;
        assert_eq!(
            decide_recovery_action(state, lineage, attempt),
            RecoveryAction::WaitForLease
        );
    }
}
