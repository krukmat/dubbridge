use dubbridge_auth::AuthSuccess;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub workspace_name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthSuccessResponse {
    pub token: String,
    pub user_id: String,
    pub workspace_id: String,
}

impl From<AuthSuccess> for AuthSuccessResponse {
    fn from(value: AuthSuccess) -> Self {
        Self {
            token: value.token,
            user_id: value.user_id.to_string(),
            workspace_id: value.workspace_id.to_string(),
        }
    }
}
