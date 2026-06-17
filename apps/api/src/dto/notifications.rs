use dubbridge_db::notification_repo::NotificationRow;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct NotificationResponse {
    pub id: Uuid,
    pub kind: String,
    pub ref_entity_type: String,
    pub ref_entity_id: Uuid,
    pub actor_subject_id: Option<Uuid>,
    pub read_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
}

impl From<NotificationRow> for NotificationResponse {
    fn from(row: NotificationRow) -> Self {
        Self {
            id: row.id,
            kind: row.kind.to_string(),
            ref_entity_type: row.ref_entity_type.to_string(),
            ref_entity_id: row.ref_entity_id,
            actor_subject_id: row.actor_subject_id,
            read_at: row.read_at,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct NotificationListResponse {
    pub notifications: Vec<NotificationResponse>,
}

#[derive(Debug, Deserialize)]
pub struct MarkNotificationsReadRequest {
    pub ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterPushTokenRequest {
    pub token: String,
    pub platform: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use dubbridge_db::notification_repo::{NotificationKind, RefEntityType};

    #[test]
    fn notification_response_maps_repo_fields() {
        let id = Uuid::new_v4();
        let actor = Uuid::new_v4();
        let ref_id = Uuid::new_v4();
        let now = OffsetDateTime::now_utc();

        let row = NotificationRow {
            id,
            recipient_subject_id: Uuid::new_v4(),
            kind: NotificationKind::ReviewTaskDecided,
            ref_entity_type: RefEntityType::ReviewTask,
            ref_entity_id: ref_id,
            actor_subject_id: Some(actor),
            read_at: None,
            created_at: now,
        };

        let response = NotificationResponse::from(row);
        assert_eq!(response.id, id);
        assert_eq!(response.kind, "review_task_decided");
        assert_eq!(response.ref_entity_type, "review_task");
        assert_eq!(response.ref_entity_id, ref_id);
        assert_eq!(response.actor_subject_id, Some(actor));
        assert!(response.read_at.is_none());
    }
}
