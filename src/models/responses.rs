use serde::{Deserialize, Serialize};
use crate::models::domain::ScoredMatch;

/// Response for find matches endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindMatchesResponse {
    pub matches: Vec<ScoredMatch>,
    pub next_cursor: Option<String>,
    pub total_results: usize,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub status_code: u16,
}

/// Record event response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordEventResponse {
    pub success: bool,
    pub event_id: String,
}
