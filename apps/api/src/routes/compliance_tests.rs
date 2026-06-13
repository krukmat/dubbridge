use std::{collections::HashMap, env, sync::Arc};

use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use dubbridge_auth::{AuthenticatedPrincipal, SharedTokenVerifier};
use dubbridge_db::{asset_repo, audit_repo, consent_repo, error::DbError, rights_repo};
use dubbridge_domain::{asset::AssetId, consent::ConsentScope};
use dubbridge_storage::LocalFsAdapter;
use sqlx::PgPool;
use tempfile::TempDir;
use uuid::Uuid;

use super::*;

#[derive(Clone, Default)]
struct StubTokenVerifier {
    responses:
        HashMap<String, Result<AuthenticatedPrincipal, dubbridge_auth::TokenVerificationError>>,
}

impl dubbridge_auth::TokenVerifier for StubTokenVerifier {
    fn verify_access_token(
        &self,
        token: &str,
    ) -> Result<AuthenticatedPrincipal, dubbridge_auth::TokenVerificationError> {
        self.responses
            .get(token)
            .cloned()
            .unwrap_or(Err(dubbridge_auth::TokenVerificationError::MalformedToken))
    }
}

async fn setup_pool() -> Option<PgPool> {
    let url = env::var("DUBBRIDGE_DATABASE_URL").ok()?;
    let pool = PgPool::connect(&url).await.expect("connect");
    sqlx::migrate!("../../infra/migrations")
        .run(&pool)
        .await
        .expect("migrations");
    Some(pool)
}

fn state(pool: PgPool) -> Arc<AppState> {
    let storage_dir = TempDir::new().expect("temp dir");
    let verifier: SharedTokenVerifier = Arc::new(StubTokenVerifier::default());
    Arc::new(AppState::new(
        pool,
        Box::new(LocalFsAdapter::new(storage_dir.path())),
        verifier,
        dubbridge_config::AppConfig::from_env(),
    ))
}

fn principal(subject_id: Uuid) -> AuthenticatedPrincipal {
    AuthenticatedPrincipal::new(
        subject_id,
        ["assets:read", "assets:ingest"]
            .into_iter()
            .map(str::to_string),
    )
}

async fn insert_asset(
    pool: &PgPool,
    title: &str,
    uploader_id: Uuid,
) -> dubbridge_domain::asset::Asset {
    let asset = dubbridge_domain::asset::Asset::new_pending(title.to_string(), uploader_id);
    asset_repo::insert_asset(pool, &asset)
        .await
        .expect("insert asset");
    asset
}

#[tokio::test]
async fn get_audit_timeline_handler_returns_owned_events() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };
    let owner_id = Uuid::new_v4();
    let asset = insert_asset(&pool, "audit-handler", owner_id).await;
    let event = dubbridge_domain::audit::AuditEvent::new_consent(
        asset.id,
        dubbridge_domain::audit::AuditEventKind::ConsentGranted,
        Some("scope=voice_clone".to_string()),
    );
    audit_repo::insert_audit_event(&pool, &event)
        .await
        .expect("insert audit");

    let response = get_audit_timeline(
        Path(asset.id.0),
        State(state(pool.clone())),
        Extension(principal(owner_id)),
    )
    .await
    .expect("audit timeline");

    assert_eq!(response.0.events.len(), 1);
    assert_eq!(
        response.0.events[0].event_kind,
        dubbridge_domain::audit::AuditEventKind::ConsentGranted
    );
}

#[tokio::test]
async fn get_rights_ledger_handler_returns_owned_entries() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };
    let owner_id = Uuid::new_v4();
    let asset = insert_asset(&pool, "rights-handler", owner_id).await;
    let record = dubbridge_domain::rights::RightsRecord::new(
        asset.id,
        &dubbridge_domain::rights::RightsBasis {
            owner: "Acme".to_string(),
            license_type: dubbridge_domain::rights::LicenseType::AllRightsReserved,
            source_type: dubbridge_domain::rights::SourceType::DirectUpload,
            proof_reference: "proof-123".to_string(),
        },
    );
    rights_repo::insert_rights_record(&pool, &record)
        .await
        .expect("insert rights");

    let response = get_rights_ledger(
        Path(asset.id.0),
        State(state(pool.clone())),
        Extension(principal(owner_id)),
    )
    .await
    .expect("rights ledger");

    assert_eq!(response.0.entries.len(), 1);
    assert_eq!(response.0.entries[0].owner, "Acme");
}

#[tokio::test]
async fn get_consent_ledger_handler_returns_current_status() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };
    let owner_id = Uuid::new_v4();
    let asset = insert_asset(&pool, "consent-handler", owner_id).await;
    let grant = new_grant(asset.id, ConsentScope::VoiceClone, "proof-1", owner_id).expect("grant");
    consent_repo::append_consent(&pool, &grant)
        .await
        .expect("append grant");

    let response = get_consent_ledger(
        Path(asset.id.0),
        State(state(pool.clone())),
        Extension(principal(owner_id)),
    )
    .await
    .expect("consent ledger");

    assert_eq!(response.0.current_status, Some(ConsentStatus::Grant));
    assert_eq!(response.0.rows.len(), 1);
}

#[tokio::test]
async fn record_consent_handler_persists_row() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };
    let owner_id = Uuid::new_v4();
    let asset = insert_asset(&pool, "record-consent-handler", owner_id).await;

    let response = record_consent(
        State(state(pool.clone())),
        Extension(principal(owner_id)),
        Json(ConsentMutationRequest {
            asset_id: asset.id.0,
            scope: ConsentScope::VoiceClone,
            status: ConsentStatus::Grant,
            evidence_ref: Some("proof-xyz".to_string()),
        }),
    )
    .await
    .expect("record consent");

    assert_eq!(response.0, StatusCode::CREATED);
    let latest = consent_repo::latest_consent_status(&pool, asset.id, &ConsentScope::VoiceClone)
        .await
        .expect("latest status");
    assert_eq!(latest, Some(ConsentStatus::Grant));
}

#[tokio::test]
async fn get_audit_timeline_handler_denies_foreign_asset() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };
    let owner_id = Uuid::new_v4();
    let outsider_id = Uuid::new_v4();
    let asset = insert_asset(&pool, "foreign-audit-handler", outsider_id).await;

    let err = get_audit_timeline(
        Path(asset.id.0),
        State(state(pool.clone())),
        Extension(principal(owner_id)),
    )
    .await
    .unwrap_err();

    assert_eq!(err.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn record_consent_handler_revoke_persists_row() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };
    let owner_id = Uuid::new_v4();
    let asset = insert_asset(&pool, "record-consent-revoke", owner_id).await;
    let grant = new_grant(
        asset.id,
        ConsentScope::VoiceClone,
        "proof-before-revoke",
        owner_id,
    )
    .expect("grant");
    consent_repo::append_consent(&pool, &grant)
        .await
        .expect("append grant");

    let response = record_consent(
        State(state(pool.clone())),
        Extension(principal(owner_id)),
        Json(ConsentMutationRequest {
            asset_id: asset.id.0,
            scope: ConsentScope::VoiceClone,
            status: ConsentStatus::Revoke,
            evidence_ref: None,
        }),
    )
    .await
    .expect("record revoke");

    assert_eq!(response.0, StatusCode::CREATED);
    let latest = consent_repo::latest_consent_status(&pool, asset.id, &ConsentScope::VoiceClone)
        .await
        .expect("latest status");
    assert_eq!(latest, Some(ConsentStatus::Revoke));
}

#[tokio::test]
async fn record_consent_handler_rejects_missing_evidence_ref() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };
    let owner_id = Uuid::new_v4();
    let asset = insert_asset(&pool, "record-consent-invalid", owner_id).await;

    let err = record_consent(
        State(state(pool.clone())),
        Extension(principal(owner_id)),
        Json(ConsentMutationRequest {
            asset_id: asset.id.0,
            scope: ConsentScope::VoiceClone,
            status: ConsentStatus::Grant,
            evidence_ref: None,
        }),
    )
    .await
    .unwrap_err();

    assert_eq!(err.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(err.message.contains("evidence_ref"));
}

#[tokio::test]
async fn ensure_owned_asset_rejects_non_owner() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };
    let owner_id = Uuid::new_v4();
    let outsider_id = Uuid::new_v4();
    let asset = insert_asset(&pool, "ownership-check", outsider_id).await;

    let err = ensure_owned_asset(&pool, asset.id, owner_id)
        .await
        .unwrap_err();

    assert_eq!(err.status, StatusCode::FORBIDDEN);
}

#[test]
fn build_consent_row_grant_requires_evidence_ref() {
    let err = build_consent_row(
        ConsentMutationRequest {
            asset_id: Uuid::new_v4(),
            scope: ConsentScope::VoiceClone,
            status: ConsentStatus::Grant,
            evidence_ref: None,
        },
        Uuid::new_v4(),
    )
    .unwrap_err();

    assert_eq!(err.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(err.message.contains("evidence_ref"));
}

#[test]
fn build_consent_row_revoke_succeeds_without_evidence_ref() {
    let asset_id = Uuid::new_v4();
    let row = build_consent_row(
        ConsentMutationRequest {
            asset_id,
            scope: ConsentScope::TtsSynthesis,
            status: ConsentStatus::Revoke,
            evidence_ref: None,
        },
        Uuid::new_v4(),
    )
    .expect("row");

    assert_eq!(row.asset_id, AssetId(asset_id));
    assert_eq!(row.status, ConsentStatus::Revoke);
    assert!(row.evidence_ref.is_none());
}

#[test]
fn scoped_db_not_found_maps_to_forbidden() {
    let error = ApiError::from_scoped_db(DbError::NotFound);
    assert_eq!(error.status, StatusCode::FORBIDDEN);
    assert_eq!(error.message, "asset not found");
}

#[test]
fn consent_gate_audit_failure_maps_fail_closed() {
    let error = ApiError::from_consent_gate(ConsentGateError::AuditFailed("disk full".to_string()));
    assert_eq!(error.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(error.message.contains("audit persistence failed"));
    assert!(error.message.contains("disk full"));
}

#[test]
fn api_error_from_db_covers_query_and_unknown_value_paths() {
    let query_error = ApiError::from_db(DbError::QueryFailed(sqlx::Error::RowNotFound));
    assert_eq!(query_error.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(query_error.message.contains("database operation failed"));

    let value_error = ApiError::from_db(DbError::UnknownStoredValue {
        field: "rights_records.license_type",
        value: "temporary".to_string(),
    });
    assert_eq!(value_error.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(value_error.message.contains("corrupt stored value"));
}

#[test]
fn api_error_from_consent_gate_covers_remaining_variants() {
    let db_error = ApiError::from_consent_gate(ConsentGateError::Db("timeout".to_string()));
    assert_eq!(db_error.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(db_error.message.contains("timeout"));

    let no_active = ApiError::from_consent_gate(ConsentGateError::NoActiveConsent {
        asset_id: AssetId::new(),
        scope: ConsentScope::VoiceClone,
    });
    assert_eq!(no_active.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(no_active.message.contains("no active consent"));
}

#[test]
fn api_error_into_response_serializes_status() {
    let response = ApiError::forbidden("asset not found").into_response();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
