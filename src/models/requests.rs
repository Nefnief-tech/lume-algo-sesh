use serde::{Deserialize, Serialize};
use validator::Validate;

/// Request to find matches
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct FindMatchesRequest {
    #[validate(length(min = 1))]
    #[serde(alias = "user_id", rename = "userId")]
    pub user_id: String,
    #[serde(default = "default_limit")]
    #[serde(alias = "limit", rename = "limit")]
    pub limit: u16,
    #[serde(default)]
    #[serde(alias = "excludeUserIds", rename = "excludeUserIds")]
    pub exclude_user_ids: Vec<String>,
    #[serde(alias = "cursor", rename = "cursor")]
    pub cursor: Option<String>,
}

fn default_limit() -> u16 {
    20
}

/// Health check request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthRequest;

/// Request to record a match event
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RecordEventRequest {
    #[validate(length(min = 1))]
    #[serde(alias = "user_id", rename = "userId")]
    pub user_id: String,
    #[validate(length(min = 1))]
    #[serde(alias = "targetUserId", rename = "targetUserId")]
    pub target_user_id: String,
    #[serde(alias = "eventType", rename = "eventType")]
    pub event_type: String,
}
