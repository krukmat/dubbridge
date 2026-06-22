use axum::http::{Method, StatusCode};
use dubbridge_db::{playback_repo, preparation_repo};
use dubbridge_domain::{artifact::PreparationStatus, workspace::OrgRole};
use uuid::Uuid;

mod support;

use support::{
    PlaybackTestContext, count_audit_events_by_kind, count_playback_grants, insert_membership,
    insert_playback_fixture, json_body, mark_asset_ready_with_manifest, send_request,
};

#[tokio::test]
async fn authorized_reviewer_ready_asset_receives_grant_and_audit_row() {
    let ctx = if let Some(ctx) = PlaybackTestContext::grant_suite().await {
        ctx
    } else {
        eprintln!("skipping playback grant test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let fixture = insert_playback_fixture(&ctx.pool).await;
    prepare_reviewer_ready_fixture(&ctx, &fixture).await;

    let response = send_request(
        &ctx.app,
        Method::POST,
        &format!("/assets/{}/playback-grants", fixture.asset.id.0),
        Some(&ctx.write_token),
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;
    let grant_id = Uuid::parse_str(body["grant_id"].as_str().expect("grant id")).expect("uuid");

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(count_playback_grants(&ctx.pool).await, 1);
    let issued_events =
        count_audit_events_by_kind(&ctx.pool, fixture.asset.id.0, "playback_grant_issued").await;
    let refused_events =
        count_audit_events_by_kind(&ctx.pool, fixture.asset.id.0, "playback_grant_refused").await;
    assert_eq!(issued_events, 1);
    assert_eq!(refused_events, 0);

    let grant = playback_repo::get_active_grant(
        &ctx.pool,
        dubbridge_domain::playback::PlaybackGrantId(grant_id),
    )
    .await
    .expect("load grant")
    .expect("grant exists");
    assert_eq!(grant.asset_id, fixture.asset.id);
    assert_eq!(grant.principal.principal_id, ctx.reviewer_id);
}

#[tokio::test]
async fn unauthenticated_request_returns_401_and_writes_no_grant_row() {
    let ctx = if let Some(ctx) = PlaybackTestContext::grant_suite().await {
        ctx
    } else {
        eprintln!("skipping playback grant test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let fixture = insert_playback_fixture(&ctx.pool).await;

    let response = send_request(
        &ctx.app,
        Method::POST,
        &format!("/assets/{}/playback-grants", fixture.asset.id.0),
        None,
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(count_playback_grants(&ctx.pool).await, 0);
    assert_eq!(
        count_audit_events_by_kind(&ctx.pool, fixture.asset.id.0, "playback_grant_issued").await,
        0
    );
}

#[tokio::test]
async fn authenticated_non_member_returns_403_and_writes_no_grant_row() {
    let ctx = if let Some(ctx) = PlaybackTestContext::grant_suite().await {
        ctx
    } else {
        eprintln!("skipping playback grant test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let fixture = insert_playback_fixture(&ctx.pool).await;
    prepare_reviewer_ready_fixture(&ctx, &fixture).await;

    let response = send_request(
        &ctx.app,
        Method::POST,
        &format!("/assets/{}/playback-grants", fixture.asset.id.0),
        ctx.outsider_write_token(),
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], "asset not found");
    assert_eq!(count_playback_grants(&ctx.pool).await, 0);
    assert_eq!(
        count_audit_events_by_kind(&ctx.pool, fixture.asset.id.0, "playback_grant_issued").await,
        0
    );
}

#[tokio::test]
async fn not_ready_asset_returns_fail_closed_denial_and_writes_no_grant_row() {
    let ctx = if let Some(ctx) = PlaybackTestContext::grant_suite().await {
        ctx
    } else {
        eprintln!("skipping playback grant test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let fixture = insert_playback_fixture(&ctx.pool).await;
    insert_membership(
        &ctx.pool,
        fixture.org.id,
        ctx.reviewer_id,
        OrgRole::Reviewer,
    )
    .await;
    preparation_repo::upsert_preparation_status(
        &ctx.pool,
        fixture.asset.id,
        PreparationStatus::InProgress,
        None,
    )
    .await
    .expect("mark in progress");

    let response = send_request(
        &ctx.app,
        Method::POST,
        &format!("/assets/{}/playback-grants", fixture.asset.id.0),
        Some(&ctx.write_token),
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"], "asset not ready for playback");
    assert_eq!(count_playback_grants(&ctx.pool).await, 0);
    assert_eq!(
        count_audit_events_by_kind(&ctx.pool, fixture.asset.id.0, "playback_grant_issued").await,
        0
    );
}

async fn prepare_reviewer_ready_fixture(
    ctx: &PlaybackTestContext,
    fixture: &support::PlaybackFixture,
) {
    insert_membership(
        &ctx.pool,
        fixture.org.id,
        ctx.reviewer_id,
        OrgRole::Reviewer,
    )
    .await;
    mark_asset_ready_with_manifest(&ctx.pool, fixture.asset.id).await;
}
