use std::time::{SystemTime, UNIX_EPOCH};

use axum::http::{Method, StatusCode, header};
use dubbridge_auth::Claims;
use dubbridge_db::playback_repo;
use dubbridge_domain::{asset::AssetId, playback::PlaybackGrantId, workspace::OrgRole};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use uuid::Uuid;

mod support;

use support::{
    PlaybackTestContext, body_string, count_audit_events_by_kind, insert_membership,
    insert_playback_fixture, json_body, mark_asset_ready_with_manifest, response_body_bytes,
    send_request,
};

#[tokio::test]
async fn valid_grant_returns_manifest_with_short_lived_segment_references() {
    let Some(ctx) = PlaybackTestContext::delivery_suite().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
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
    mark_asset_ready_with_manifest(&ctx.pool, fixture.asset.id).await;
    ctx.put_manifest(
        fixture.asset.id,
        "#EXTM3U\n#EXT-X-VERSION:3\n#EXTINF:6.0,\nprepared/asset-123/segment_00000.ts\n#EXTINF:6.0,\nprepared/asset-123/segment_00001.ts\n#EXT-X-ENDLIST\n",
    )
    .await;
    ctx.put_segment(
        fixture.asset.id,
        "segment_00000.ts",
        b"segment-zero".to_vec(),
    )
    .await;
    ctx.put_segment(
        fixture.asset.id,
        "segment_00001.ts",
        b"segment-one".to_vec(),
    )
    .await;

    let grant_id = issue_grant(&ctx, fixture.asset.id).await;
    let response = send_request(
        &ctx.app,
        Method::GET,
        &format!(
            "/assets/{}/playback/{}/manifest",
            fixture.asset.id.0, grant_id
        ),
        None,
    )
    .await;
    let status = response.status();
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string)
        .expect("content type");
    let body = body_string(response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(content_type, "application/vnd.apple.mpegurl");
    assert!(body.contains("#EXTM3U"));
    assert!(body.contains(&format!(
        "/assets/{}/playback/segments/segment_00000.ts?token=",
        fixture.asset.id.0
    )));
    assert!(body.contains(&format!(
        "/assets/{}/playback/segments/segment_00001.ts?token=",
        fixture.asset.id.0
    )));
    assert!(!body.contains("prepared/asset-123/segment_00000.ts"));
    assert!(!body.contains(&format!("assets/{}/prepared/hls/", fixture.asset.id.0)));
    assert_ne!(grant_id, Uuid::nil());
}

#[tokio::test]
async fn valid_short_lived_segment_reference_returns_segment_bytes_without_new_audit_row() {
    let Some(ctx) = PlaybackTestContext::delivery_suite().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
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
    mark_asset_ready_with_manifest(&ctx.pool, fixture.asset.id).await;
    ctx.put_manifest(
        fixture.asset.id,
        "#EXTM3U\n#EXT-X-VERSION:3\n#EXTINF:6.0,\nprepared/asset-123/segment_00000.ts\n#EXT-X-ENDLIST\n",
    )
    .await;
    ctx.put_segment(
        fixture.asset.id,
        "segment_00000.ts",
        b"segment-zero".to_vec(),
    )
    .await;

    let grant_id = issue_grant(&ctx, fixture.asset.id).await;
    let manifest_response = send_request(
        &ctx.app,
        Method::GET,
        &format!(
            "/assets/{}/playback/{}/manifest",
            fixture.asset.id.0, grant_id
        ),
        None,
    )
    .await;
    let manifest_body = body_string(manifest_response).await;
    let segment_url = manifest_segment_urls(&manifest_body)
        .into_iter()
        .next()
        .expect("segment url");

    let response = send_request(&ctx.app, Method::GET, &segment_url, None).await;
    let status = response.status();
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string)
        .expect("content type");
    let body = response_body_bytes(response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(content_type, "video/mp2t");
    assert_eq!(body, b"segment-zero");
    assert_eq!(
        count_audit_events_by_kind(&ctx.pool, fixture.asset.id.0, "playback_grant_issued").await,
        1
    );
    assert_eq!(
        count_audit_events_by_kind(&ctx.pool, fixture.asset.id.0, "playback_grant_refused").await,
        0
    );
}

#[tokio::test]
async fn expired_grant_manifest_request_is_denied_fail_closed() {
    let Some(ctx) = PlaybackTestContext::delivery_suite().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
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
    mark_asset_ready_with_manifest(&ctx.pool, fixture.asset.id).await;
    ctx.put_manifest(
        fixture.asset.id,
        "#EXTM3U\n#EXTINF:6.0,\nprepared/asset-123/segment_00000.ts\n",
    )
    .await;

    let grant_id = issue_grant(&ctx, fixture.asset.id).await;
    playback_repo::expire_grant(&ctx.pool, PlaybackGrantId(grant_id))
        .await
        .expect("expire grant");

    let response = send_request(
        &ctx.app,
        Method::GET,
        &format!(
            "/assets/{}/playback/{}/manifest",
            fixture.asset.id.0, grant_id
        ),
        None,
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], "playback grant expired or revoked");
}

#[tokio::test]
async fn missing_stored_manifest_fails_closed_without_fabricated_playlist() {
    let Some(ctx) = PlaybackTestContext::delivery_suite().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
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
    mark_asset_ready_with_manifest(&ctx.pool, fixture.asset.id).await;

    let grant_id = issue_grant(&ctx, fixture.asset.id).await;
    let response = send_request(
        &ctx.app,
        Method::GET,
        &format!(
            "/assets/{}/playback/{}/manifest",
            fixture.asset.id.0, grant_id
        ),
        None,
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body["error"], "prepared HLS manifest not found");
}

#[tokio::test]
async fn invalid_utf8_manifest_fails_closed_without_leaking_storage_key() {
    let Some(ctx) = PlaybackTestContext::delivery_suite().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
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
    mark_asset_ready_with_manifest(&ctx.pool, fixture.asset.id).await;
    ctx.put_manifest_bytes(fixture.asset.id, vec![0xff, 0xfe, 0xfd])
        .await;

    let grant_id = issue_grant(&ctx, fixture.asset.id).await;
    let response = send_request(
        &ctx.app,
        Method::GET,
        &format!(
            "/assets/{}/playback/{}/manifest",
            fixture.asset.id.0, grant_id
        ),
        None,
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    let error = body["error"].as_str().expect("error string");
    assert!(error.contains("stored manifest is not valid UTF-8"));
    assert!(!error.contains("prepared/hls"));
    assert!(!error.contains("index.m3u8"));
}

#[tokio::test]
async fn expired_short_lived_segment_reference_is_denied_fail_closed() {
    let Some(ctx) = PlaybackTestContext::delivery_suite().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let fixture = insert_playback_fixture(&ctx.pool).await;
    ctx.put_segment(
        fixture.asset.id,
        "segment_00000.ts",
        b"segment-zero".to_vec(),
    )
    .await;

    let expired_token = expired_segment_token(fixture.asset.id, "segment_00000.ts");

    let response = send_request(
        &ctx.app,
        Method::GET,
        &format!(
            "/assets/{}/playback/segments/segment_00000.ts?token={expired_token}",
            fixture.asset.id.0
        ),
        None,
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], "segment reference expired or invalid");
}

#[tokio::test]
async fn scoped_segment_reference_cannot_be_replayed_against_another_asset() {
    let Some(ctx) = PlaybackTestContext::delivery_suite().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let fixture_a = insert_playback_fixture(&ctx.pool).await;
    let fixture_b = insert_playback_fixture(&ctx.pool).await;
    insert_membership(
        &ctx.pool,
        fixture_a.org.id,
        ctx.reviewer_id,
        OrgRole::Reviewer,
    )
    .await;
    mark_asset_ready_with_manifest(&ctx.pool, fixture_a.asset.id).await;
    ctx.put_manifest(
        fixture_a.asset.id,
        "#EXTM3U\n#EXTINF:6.0,\nprepared/asset-123/segment_00000.ts\n#EXT-X-ENDLIST\n",
    )
    .await;
    ctx.put_segment(
        fixture_a.asset.id,
        "segment_00000.ts",
        b"segment-zero".to_vec(),
    )
    .await;

    let grant_id = issue_grant(&ctx, fixture_a.asset.id).await;
    let manifest_response = send_request(
        &ctx.app,
        Method::GET,
        &format!(
            "/assets/{}/playback/{}/manifest",
            fixture_a.asset.id.0, grant_id
        ),
        None,
    )
    .await;
    let manifest_body = body_string(manifest_response).await;
    let segment_url = manifest_segment_urls(&manifest_body)
        .into_iter()
        .next()
        .expect("segment url");
    let replay_url = segment_url.replace(
        &fixture_a.asset.id.0.to_string(),
        &fixture_b.asset.id.0.to_string(),
    );

    let response = send_request(&ctx.app, Method::GET, &replay_url, None).await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], "segment reference expired or invalid");
}

async fn issue_grant(ctx: &PlaybackTestContext, asset_id: AssetId) -> Uuid {
    let response = send_request(
        &ctx.app,
        Method::POST,
        &format!("/assets/{}/playback-grants", asset_id.0),
        Some(&ctx.write_token),
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::CREATED);
    Uuid::parse_str(body["grant_id"].as_str().expect("grant id")).expect("uuid")
}

fn manifest_segment_urls(manifest: &str) -> Vec<String> {
    manifest
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .map(str::to_string)
        .collect()
}

fn expired_segment_token(asset_id: AssetId, filename: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("unix epoch")
        .as_secs();
    let claims = Claims {
        sub: asset_id.0.to_string(),
        workspace_id: asset_id.0.to_string(),
        iat: now - 7200,
        nbf: now - 7200,
        exp: now - 3600,
        scope: format!("playback_segment:{filename}"),
    };

    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret("local-dev-jwt-secret-placeholder".as_bytes()),
    )
    .expect("token")
}
