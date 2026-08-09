use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::DbError;
use crate::workspace_repo::{
    AssetRow, MemberRow, OrgRow, ProjectRow, asset_from_row, member_from_row, org_from_row,
    parse_asset_status, project_from_row, require_org_role,
};
use dubbridge_domain::asset::IngestionStatus;
use dubbridge_domain::workspace::OrgRole;

#[test]
fn require_org_role_known_variants_succeed() {
    assert!(matches!(require_org_role("owner"), Ok(OrgRole::Owner)));
    assert!(matches!(require_org_role("admin"), Ok(OrgRole::Admin)));
    assert!(matches!(require_org_role("editor"), Ok(OrgRole::Editor)));
    assert!(matches!(
        require_org_role("reviewer"),
        Ok(OrgRole::Reviewer)
    ));
    assert!(matches!(require_org_role("viewer"), Ok(OrgRole::Viewer)));
}

#[test]
fn require_org_role_unknown_value_fails_closed() {
    let err = require_org_role("superuser").unwrap_err();
    assert!(matches!(
        err,
        DbError::UnknownStoredValue {
            field: "org_members.role",
            ..
        }
    ));
    assert!(err.to_string().contains("superuser"));
}

#[test]
fn require_org_role_empty_string_fails_closed() {
    assert!(require_org_role("").is_err());
}

#[test]
fn parse_asset_status_known_variants_succeed() {
    assert!(matches!(
        parse_asset_status("pending"),
        Ok(IngestionStatus::Pending)
    ));
    assert!(matches!(
        parse_asset_status("finalized"),
        Ok(IngestionStatus::Finalized)
    ));
}

#[test]
fn parse_asset_status_unknown_value_fails_closed() {
    let err = parse_asset_status("processing").unwrap_err();
    assert!(matches!(
        err,
        DbError::UnknownStoredValue {
            field: "assets.status",
            ..
        }
    ));
}

#[test]
fn parse_asset_status_all_known_variants_succeed() {
    assert!(matches!(
        parse_asset_status("rejected_missing_rights"),
        Ok(IngestionStatus::RejectedMissingRights)
    ));
    assert!(matches!(
        parse_asset_status("rejected_missing_uploader_context"),
        Ok(IngestionStatus::RejectedMissingUploaderContext)
    ));
}

#[test]
fn org_from_row_maps_fields_correctly() {
    let id = Uuid::new_v4();
    let now = OffsetDateTime::now_utc();
    let row = OrgRow {
        id,
        name: "Test Org".to_string(),
        created_at: now,
        updated_at: now,
    };
    let org = org_from_row(row);
    assert_eq!(org.id.0, id);
    assert_eq!(org.name, "Test Org");
}

#[test]
fn member_from_row_maps_known_role() {
    let org_id = Uuid::new_v4();
    let subject_id = Uuid::new_v4();
    let now = OffsetDateTime::now_utc();
    let row = MemberRow {
        org_id,
        subject_id,
        role: "admin".to_string(),
        joined_at: now,
    };
    let member = member_from_row(row).unwrap();
    assert_eq!(member.org_id.0, org_id);
    assert_eq!(member.subject_id, subject_id);
    assert_eq!(member.role, OrgRole::Admin);
}

#[test]
fn member_from_row_unknown_role_fails_closed() {
    let row = MemberRow {
        org_id: Uuid::new_v4(),
        subject_id: Uuid::new_v4(),
        role: "superuser".to_string(),
        joined_at: OffsetDateTime::now_utc(),
    };
    assert!(member_from_row(row).is_err());
}

#[test]
fn project_from_row_maps_fields_correctly() {
    let id = Uuid::new_v4();
    let org_id = Uuid::new_v4();
    let now = OffsetDateTime::now_utc();
    let row = ProjectRow {
        id,
        org_id,
        name: "My Project".to_string(),
        created_at: now,
        updated_at: now,
    };
    let project = project_from_row(row);
    assert_eq!(project.id.0, id);
    assert_eq!(project.org_id.0, org_id);
    assert_eq!(project.name, "My Project");
}

#[test]
fn asset_from_row_maps_known_status() {
    let id = Uuid::new_v4();
    let uploader = Uuid::new_v4();
    let now = OffsetDateTime::now_utc();
    let row = AssetRow {
        id,
        title: "Demo Reel".to_string(),
        uploader_id: uploader,
        status: "finalized".to_string(),
        created_at: now,
        updated_at: now,
    };
    let asset = asset_from_row(row).unwrap();
    assert_eq!(asset.id.0, id);
    assert_eq!(asset.title, "Demo Reel");
    assert_eq!(asset.uploader_id, uploader);
    assert_eq!(asset.status, IngestionStatus::Finalized);
}

#[test]
fn asset_from_row_unknown_status_fails_closed() {
    let row = AssetRow {
        id: Uuid::new_v4(),
        title: "X".to_string(),
        uploader_id: Uuid::new_v4(),
        status: "corrupted".to_string(),
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
    };
    assert!(asset_from_row(row).is_err());
}
