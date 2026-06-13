// Integration tests for consent_gate async entry points.
// Unit tests for the sync decision logic live in consent_gate.rs itself.
// These tests require DUBBRIDGE_DATABASE_URL to be set; they skip gracefully otherwise.
use std::env;

use dubbridge_api::consent_gate::{
    ConsentGateError, append_consent_audited, require_active_consent,
};
use dubbridge_db::consent_repo;
use dubbridge_domain::{
    asset::AssetId,
    consent::{ConsentRow, ConsentScope, ConsentStatus},
};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

async fn setup_pool() -> Option<PgPool> {
    let url = env::var("DUBBRIDGE_DATABASE_URL").ok()?;
    let pool = PgPool::connect(&url).await.expect("connect");
    sqlx::migrate!("../../infra/migrations")
        .run(&pool)
        .await
        .expect("migrations");
    Some(pool)
}

async fn insert_asset(pool: &PgPool) -> AssetId {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO assets (id, title, uploader_id, status) VALUES ($1, $2, $3, $4)")
        .bind(id)
        .bind("consent-gate-test-asset")
        .bind(Uuid::new_v4())
        .bind("finalized")
        .execute(pool)
        .await
        .expect("insert asset");
    AssetId(id)
}

fn grant_row(asset_id: AssetId, scope: ConsentScope) -> ConsentRow {
    ConsentRow {
        id: Uuid::new_v4(),
        asset_id,
        scope,
        status: ConsentStatus::Grant,
        evidence_ref: None,
        granted_by: Uuid::new_v4(),
        happened_at: OffsetDateTime::now_utc(),
    }
}

fn revoke_row(asset_id: AssetId, scope: ConsentScope) -> ConsentRow {
    ConsentRow {
        id: Uuid::new_v4(),
        asset_id,
        scope,
        status: ConsentStatus::Revoke,
        evidence_ref: None,
        granted_by: Uuid::new_v4(),
        happened_at: OffsetDateTime::now_utc(),
    }
}

// IT-1: no consent row → require_active_consent returns Err(NoActiveConsent)
#[tokio::test]
async fn no_consent_row_returns_no_active_consent() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };
    let asset_id = insert_asset(&pool).await;
    let result = require_active_consent(&pool, asset_id, &ConsentScope::VoiceClone).await;
    assert!(
        matches!(result, Err(ConsentGateError::NoActiveConsent { .. })),
        "expected NoActiveConsent, got {result:?}"
    );
}

// IT-2: grant row present → require_active_consent returns Ok(())
#[tokio::test]
async fn grant_row_returns_ok() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };
    let asset_id = insert_asset(&pool).await;
    consent_repo::append_consent(&pool, &grant_row(asset_id, ConsentScope::VoiceClone))
        .await
        .expect("append grant");
    let result = require_active_consent(&pool, asset_id, &ConsentScope::VoiceClone).await;
    assert!(result.is_ok(), "expected Ok(()), got {result:?}");
}

// IT-3: grant then revoke → require_active_consent returns Err(NoActiveConsent)
#[tokio::test]
async fn grant_then_revoke_returns_no_active_consent() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };
    let asset_id = insert_asset(&pool).await;
    consent_repo::append_consent(&pool, &grant_row(asset_id, ConsentScope::VoiceClone))
        .await
        .expect("append grant");
    // ensure revoke has a later timestamp
    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    consent_repo::append_consent(&pool, &revoke_row(asset_id, ConsentScope::VoiceClone))
        .await
        .expect("append revoke");
    let result = require_active_consent(&pool, asset_id, &ConsentScope::VoiceClone).await;
    assert!(
        matches!(result, Err(ConsentGateError::NoActiveConsent { .. })),
        "expected NoActiveConsent after revoke, got {result:?}"
    );
}

// IT-4 (HP-3): append_consent_audited with Grant → Ok(()) + consent row written
#[tokio::test]
async fn append_consent_audited_grant_returns_ok() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };
    let asset_id = insert_asset(&pool).await;
    let row = grant_row(asset_id, ConsentScope::VoiceClone);
    let result = append_consent_audited(&pool, &row).await;
    assert!(result.is_ok(), "expected Ok(()), got {result:?}");
    // verify the consent row was written
    let status = consent_repo::latest_consent_status(&pool, asset_id, &ConsentScope::VoiceClone)
        .await
        .expect("latest status");
    assert_eq!(status, Some(ConsentStatus::Grant));
}

// IT-5 (HP-4): append_consent_audited with Revoke → Ok(()) + row written
#[tokio::test]
async fn append_consent_audited_revoke_returns_ok() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };
    let asset_id = insert_asset(&pool).await;
    // grant first so there is a base row
    append_consent_audited(&pool, &grant_row(asset_id, ConsentScope::VoiceClone))
        .await
        .expect("grant");
    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    let result =
        append_consent_audited(&pool, &revoke_row(asset_id, ConsentScope::VoiceClone)).await;
    assert!(result.is_ok(), "expected Ok(()), got {result:?}");
    let status = consent_repo::latest_consent_status(&pool, asset_id, &ConsentScope::VoiceClone)
        .await
        .expect("latest status");
    assert_eq!(status, Some(ConsentStatus::Revoke));
}
