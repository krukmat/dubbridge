// S-125-T1: playback-grant domain contract (ADR-032)
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    asset::AssetId,
    workspace::{OrgId, ProjectId},
};

// ── Errors ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PlaybackError {
    #[error("unknown grant status: {0}")]
    UnknownGrantStatus(String),
    #[error("unknown playback scope: {0}")]
    UnknownPlaybackScope(String),
    /// expiry is at or before issued_at — structurally invalid.
    #[error("expiry must be after issued_at")]
    InvalidExpiry,
}

// ── Enums ─────────────────────────────────────────────────────────────────────

/// Lifecycle status of a playback grant row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrantStatus {
    Active,
    Expired,
    Revoked,
}

impl std::fmt::Display for GrantStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Active => "active",
            Self::Expired => "expired",
            Self::Revoked => "revoked",
        };
        write!(f, "{s}")
    }
}

impl FromStr for GrantStatus {
    type Err = PlaybackError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(Self::Active),
            "expired" => Ok(Self::Expired),
            "revoked" => Ok(Self::Revoked),
            other => Err(PlaybackError::UnknownGrantStatus(other.to_owned())),
        }
    }
}

/// Scope of a playback grant — what kind of consumer is allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackScope {
    /// Authenticated reviewer within the asset's org/project (review-time playback).
    Review,
}

impl std::fmt::Display for PlaybackScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Review => "review",
            }
        )
    }
}

impl FromStr for PlaybackScope {
    type Err = PlaybackError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "review" => Ok(Self::Review),
            other => Err(PlaybackError::UnknownPlaybackScope(other.to_owned())),
        }
    }
}

// ── Grant ID ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlaybackGrantId(pub Uuid);

impl PlaybackGrantId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for PlaybackGrantId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for PlaybackGrantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ── Principal binding ─────────────────────────────────────────────────────────

/// Identity context bound to a playback grant at issuance time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrantPrincipal {
    pub principal_id: Uuid,
    pub org_id: OrgId,
    pub project_id: ProjectId,
}

// ── PlaybackGrant ─────────────────────────────────────────────────────────────

/// A backend-owned, scoped, expiring authorization to play one prepared asset's
/// HLS package (ADR-032). Clients never hold raw object-store keys; they hold a
/// `PlaybackGrantId` and receive rewritten manifests/segments from the delivery
/// boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackGrant {
    pub id: PlaybackGrantId,
    pub asset_id: AssetId,
    pub scope: PlaybackScope,
    pub principal: GrantPrincipal,
    pub status: GrantStatus,
    pub issued_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
}

impl PlaybackGrant {
    /// Construct and validate a new grant. Returns `Err(InvalidExpiry)` if
    /// `expires_at` is not strictly after `issued_at`.
    pub fn new(
        id: PlaybackGrantId,
        asset_id: AssetId,
        scope: PlaybackScope,
        principal: GrantPrincipal,
        issued_at: OffsetDateTime,
        expires_at: OffsetDateTime,
    ) -> Result<Self, PlaybackError> {
        if expires_at <= issued_at {
            return Err(PlaybackError::InvalidExpiry);
        }
        Ok(Self {
            id,
            asset_id,
            scope,
            principal,
            status: GrantStatus::Active,
            issued_at,
            expires_at,
        })
    }

    /// Evaluate the grant against a caller-supplied `now`. Returns `true` only
    /// when the grant is `Active` and has not yet reached its expiry.
    pub fn is_valid_at(&self, now: OffsetDateTime) -> bool {
        self.status == GrantStatus::Active && now < self.expires_at
    }
}

// ── Denial reasons ────────────────────────────────────────────────────────────

/// Structured reason for a playback grant refusal. Used by the API layer to
/// produce fail-closed denials without leaking internal state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaybackDenial {
    /// The asset's preparation status is not `Ready`.
    NotReady,
    /// No `HlsManifest` lineage row exists for the asset.
    MissingManifest,
    /// The caller is not authenticated.
    Unauthenticated,
    /// The caller is authenticated but not authorised for the asset's org/project.
    Unauthorized,
    /// The grant exists but is expired or revoked.
    GrantInvalid,
}

impl std::fmt::Display for PlaybackDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::NotReady => "asset not ready for playback",
            Self::MissingManifest => "prepared HLS manifest not found",
            Self::Unauthenticated => "authentication required",
            Self::Unauthorized => "not authorised for this asset",
            Self::GrantInvalid => "playback grant expired or revoked",
        };
        write!(f, "{s}")
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use time::Duration;
    use uuid::Uuid;

    fn org_id() -> OrgId {
        OrgId(Uuid::new_v4())
    }

    fn project_id() -> ProjectId {
        ProjectId(Uuid::new_v4())
    }

    fn principal() -> GrantPrincipal {
        GrantPrincipal {
            principal_id: Uuid::new_v4(),
            org_id: org_id(),
            project_id: project_id(),
        }
    }

    fn now() -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }

    // ── HP-1: valid grant construction ────────────────────────────────────────

    #[test]
    fn valid_grant_is_active() {
        let issued = now();
        let expires = issued + Duration::hours(1);
        let grant = PlaybackGrant::new(
            PlaybackGrantId::new(),
            AssetId(Uuid::new_v4()),
            PlaybackScope::Review,
            principal(),
            issued,
            expires,
        )
        .unwrap();
        assert_eq!(grant.status, GrantStatus::Active);
    }

    // ── HP-2: expiry evaluation ───────────────────────────────────────────────

    #[test]
    fn grant_is_valid_before_expiry() {
        let issued = now();
        let expires = issued + Duration::hours(1);
        let grant = PlaybackGrant::new(
            PlaybackGrantId::new(),
            AssetId(Uuid::new_v4()),
            PlaybackScope::Review,
            principal(),
            issued,
            expires,
        )
        .unwrap();
        let before = issued + Duration::minutes(30);
        assert!(grant.is_valid_at(before));
    }

    #[test]
    fn grant_is_invalid_at_expiry() {
        let issued = now();
        let expires = issued + Duration::hours(1);
        let grant = PlaybackGrant::new(
            PlaybackGrantId::new(),
            AssetId(Uuid::new_v4()),
            PlaybackScope::Review,
            principal(),
            issued,
            expires,
        )
        .unwrap();
        assert!(!grant.is_valid_at(expires));
    }

    #[test]
    fn grant_is_invalid_after_expiry() {
        let issued = now();
        let expires = issued + Duration::hours(1);
        let grant = PlaybackGrant::new(
            PlaybackGrantId::new(),
            AssetId(Uuid::new_v4()),
            PlaybackScope::Review,
            principal(),
            issued,
            expires,
        )
        .unwrap();
        assert!(!grant.is_valid_at(expires + Duration::seconds(1)));
    }

    // ── EC-1: unknown stored token → typed error, never allow ────────────────

    #[test]
    fn unknown_grant_status_is_error() {
        let err = "unknown_token".parse::<GrantStatus>().unwrap_err();
        assert_eq!(
            err,
            PlaybackError::UnknownGrantStatus("unknown_token".to_owned())
        );
    }

    #[test]
    fn unknown_playback_scope_is_error() {
        let err = "audience".parse::<PlaybackScope>().unwrap_err();
        assert_eq!(
            err,
            PlaybackError::UnknownPlaybackScope("audience".to_owned())
        );
    }

    #[test]
    fn empty_string_grant_status_is_error() {
        assert!("".parse::<GrantStatus>().is_err());
    }

    #[test]
    fn empty_string_playback_scope_is_error() {
        assert!("".parse::<PlaybackScope>().is_err());
    }

    #[test]
    fn all_grant_status_variants_roundtrip() {
        for s in ["active", "expired", "revoked"] {
            let parsed: GrantStatus = s.parse().unwrap();
            assert_eq!(parsed.to_string(), s);
        }
    }

    #[test]
    fn all_playback_scope_variants_roundtrip() {
        let parsed: PlaybackScope = "review".parse().unwrap();
        assert_eq!(parsed.to_string(), "review");
    }

    // ── EC-2: expiry before issued → construction rejected ────────────────────

    #[test]
    fn expiry_equal_to_issued_is_rejected() {
        let ts = now();
        let err = PlaybackGrant::new(
            PlaybackGrantId::new(),
            AssetId(Uuid::new_v4()),
            PlaybackScope::Review,
            principal(),
            ts,
            ts,
        )
        .unwrap_err();
        assert_eq!(err, PlaybackError::InvalidExpiry);
    }

    #[test]
    fn expiry_before_issued_is_rejected() {
        let issued = now();
        let err = PlaybackGrant::new(
            PlaybackGrantId::new(),
            AssetId(Uuid::new_v4()),
            PlaybackScope::Review,
            principal(),
            issued,
            issued - Duration::seconds(1),
        )
        .unwrap_err();
        assert_eq!(err, PlaybackError::InvalidExpiry);
    }

    // ── Additional behavioural coverage ──────────────────────────────────────

    #[test]
    fn non_active_grant_is_invalid_even_before_expiry() {
        let issued = now();
        let expires = issued + Duration::hours(1);
        let mut grant = PlaybackGrant::new(
            PlaybackGrantId::new(),
            AssetId(Uuid::new_v4()),
            PlaybackScope::Review,
            principal(),
            issued,
            expires,
        )
        .unwrap();
        grant.status = GrantStatus::Revoked;
        assert!(!grant.is_valid_at(issued + Duration::minutes(1)));
    }

    #[test]
    fn denial_display_does_not_leak_internals() {
        for denial in [
            PlaybackDenial::NotReady,
            PlaybackDenial::MissingManifest,
            PlaybackDenial::Unauthenticated,
            PlaybackDenial::Unauthorized,
            PlaybackDenial::GrantInvalid,
        ] {
            let msg = denial.to_string();
            assert!(!msg.contains("s3://"));
            assert!(!msg.contains("minio"));
        }
    }
}
