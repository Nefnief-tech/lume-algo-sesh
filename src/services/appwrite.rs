use crate::models::{UserProfile, UserPreferences, MatchEvent};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use thiserror::Error;

/// Errors that can occur when interacting with Appwrite
#[derive(Debug, Error)]
pub enum AppwriteError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("API returned error: {0}")]
    ApiError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: invalid API key or token")]
    Unauthorized,

    #[error("Invalid response format: {0}")]
    InvalidResponse(String),
}

/// Appwrite API client
///
/// Handles all communication with the Appwrite backend including:
/// - Fetching user preferences
/// - Querying candidate profiles
/// - Recording match events
pub struct AppwriteClient {
    base_url: String,
    api_key: String,
    project_id: String,
    database_id: String,
    client: Client,
    collections: AppwriteCollections,
}

/// Collection IDs in Appwrite
#[derive(Debug, Clone)]
pub struct AppwriteCollections {
    pub user_profiles: String,
    pub user_preferences: String,
    pub match_events: String,
    pub user_matches: String,
}

impl AppwriteClient {
    /// Create a new Appwrite client
    pub fn new(
        base_url: String,
        api_key: String,
        project_id: String,
        database_id: String,
        collections: AppwriteCollections,
    ) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            base_url,
            api_key,
            project_id,
            database_id,
            client,
            collections,
        }
    }

    /// Fetch user preferences for a given user ID
    pub async fn get_preferences(
        &self,
        user_id: &str,
    ) -> Result<UserPreferences, AppwriteError> {
        // Build Appwrite query format: JSON array of query strings
        let query_json = format!(r#"["userId={}"]"#, user_id);
        let encoded_query = urlencoding::encode(&query_json);

        let url = format!(
            "{}/databases/{}/collections/{}/documents?query={}",
            self.base_url.trim_end_matches('/'),
            self.database_id,
            self.collections.user_preferences,
            encoded_query
        );

        tracing::debug!("Fetching preferences from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("X-Appwrite-Key", &self.api_key)
            .header("X-Appwrite-Project", &self.project_id)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppwriteError::ApiError(format!(
                "Failed to fetch preferences: {}",
                response.status()
            )));
        }

        let json: Value = response.json().await?;

        // Check if documents exist
        let documents = json
            .get("documents")
            .and_then(|d| d.as_array())
            .ok_or_else(|| AppwriteError::InvalidResponse("Missing documents array".into()))?;

        let doc = documents
            .first()
            .ok_or_else(|| AppwriteError::NotFound(format!("Preferences not found for user {}", user_id)))?;

        // Extract preferences data from Appwrite document format
        let data = doc.get("data").unwrap_or(doc);

        serde_json::from_value(data.clone())
            .map_err(|e| AppwriteError::InvalidResponse(format!("Failed to parse preferences: {}", e)))
    }

    /// Query candidate profiles based on the provided query parameters
    pub async fn query_candidates(
        &self,
        user_id: &str,
        preferences: &UserPreferences,
        exclude_ids: &[String],
        _limit: usize,
    ) -> Result<Vec<UserProfile>, AppwriteError> {
        let url = format!(
            "{}/databases/{}/collections/{}/documents",
            self.base_url.trim_end_matches('/'),
            self.database_id,
            self.collections.user_profiles
        );

        // Build Appwrite queries
        let mut queries = vec![
            format!("equal(\"isActive\", true)"),
            format!("equal(\"isTimeout\", false)"),
            format!("notEqual(\"userId\", \"{}\")", user_id), // Exclude self
        ];

        // Add gender preference filter
        if !preferences.preferred_genders.is_empty() {
            let gender_filter = preferences
                .preferred_genders
                .iter()
                .map(|g| format!("\"{}\"", g))
                .collect::<Vec<_>>()
                .join(",");
            queries.push(format!("in(\"gender\", [{}])", gender_filter));
        }

        // Add age range filter
        queries.push(format!("greaterThan(\"age\", {})", preferences.min_age as i32 - 1));
        queries.push(format!("lessThan(\"age\", {})", preferences.max_age as i32 + 1));

        // Add geospatial bounding box filter
        let bbox = crate::core::distance::calculate_bounding_box(
            preferences.latitude,
            preferences.longitude,
            preferences.max_distance_km as f64,
        );
        queries.push(format!("greaterThan(\"latitude\", {})", bbox.min_lat));
        queries.push(format!("lessThan(\"latitude\", {})", bbox.max_lat));
        queries.push(format!("greaterThan(\"longitude\", {})", bbox.min_lon));
        queries.push(format!("lessThan(\"longitude\", {})", bbox.max_lon));

        // Add exclude user IDs
        for id in exclude_ids {
            queries.push(format!("notEqual(\"userId\", \"{}\")", id));
        }

        // Build query array for Appwrite
        let queries_json = serde_json::to_string(&queries).unwrap();
        let encoded_queries = urlencoding::encode(&queries_json);

        // Build full URL with query parameter
        let full_url = format!("{}?query={}", url, encoded_queries);

        let response = self
            .client
            .get(&full_url)
            .header("X-Appwrite-Key", &self.api_key)
            .header("X-Appwrite-Project", &self.project_id)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppwriteError::ApiError(format!(
                "Failed to query candidates: {}",
                response.status()
            )));
        }

        let json: Value = response.json().await?;

        let total = json
            .get("total")
            .and_then(|t| t.as_u64())
            .unwrap_or(0);

        let documents = json
            .get("documents")
            .and_then(|d| d.as_array())
            .ok_or_else(|| AppwriteError::InvalidResponse("Missing documents array".into()))?;

        let profiles: Vec<UserProfile> = documents
            .iter()
            .filter_map(|doc| {
                let data = doc.get("data").unwrap_or(doc);
                serde_json::from_value(data.clone()).ok()
            })
            .filter(|p: &UserProfile| p.user_id != user_id && !exclude_ids.contains(&p.user_id))
            .collect();

        tracing::debug!("Queried {} candidates (total: {})", profiles.len(), total);

        Ok(profiles)
    }

    /// Get a single profile by user ID
    pub async fn get_profile(&self, user_id: &str) -> Result<UserProfile, AppwriteError> {
        // Build Appwrite query format: JSON array of query strings
        let query_json = format!(r#"["userId={}"]"#, user_id);
        let encoded_query = urlencoding::encode(&query_json);

        let url = format!(
            "{}/databases/{}/collections/{}/documents?query={}",
            self.base_url.trim_end_matches('/'),
            self.database_id,
            self.collections.user_profiles,
            encoded_query
        );

        tracing::debug!("Fetching profile for user: {}", user_id);

        let response = self
            .client
            .get(&url)
            .header("X-Appwrite-Key", &self.api_key)
            .header("X-Appwrite-Project", &self.project_id)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_else(|_| "Unable to read body".to_string());
            tracing::error!("Failed to fetch profile for {}: {} - {}", user_id, status, body);
            return Err(AppwriteError::ApiError(format!(
                "Failed to fetch profile: {}",
                status
            )));
        }

        let json: Value = response.json().await?;

        let documents = json
            .get("documents")
            .and_then(|d| d.as_array())
            .ok_or_else(|| AppwriteError::InvalidResponse("Missing documents array".into()))?;

        let doc = documents
            .first()
            .ok_or_else(|| AppwriteError::NotFound(format!("Profile not found for user {}", user_id)))?;

        let data = doc.get("data").unwrap_or(doc);

        serde_json::from_value(data.clone())
            .map_err(|e| AppwriteError::InvalidResponse(format!("Failed to parse profile: {}", e)))
    }

    /// Record a match event
    pub async fn record_event(&self, event: MatchEvent) -> Result<(), AppwriteError> {
        let url = format!(
            "{}/databases/{}/collections/{}/documents",
            self.base_url.trim_end_matches('/'),
            self.database_id,
            self.collections.match_events
        );

        let mut payload = serde_json::to_value(&event).unwrap();
        // Add Appwrite-specific fields
        if let Some(obj) = payload.as_object_mut() {
            obj.insert("$id".to_string(), Value::String(uuid::Uuid::new_v4().to_string()));
        }

        let response = self
            .client
            .post(&url)
            .header("X-Appwrite-Key", &self.api_key)
            .header("X-Appwrite-Project", &self.project_id)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppwriteError::ApiError(format!(
                "Failed to record event: {}",
                response.status()
            )));
        }

        tracing::debug!("Recorded event: {:?} -> {:?}", event.user_id, event.target_user_id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_appwrite_client_creation() {
        let collections = AppwriteCollections {
            user_profiles: "user_profiles".to_string(),
            user_preferences: "user_preferences".to_string(),
            match_events: "match_events".to_string(),
            user_matches: "user_matches".to_string(),
        };

        let client = AppwriteClient::new(
            "https://appwrite.test/v1".to_string(),
            "test_key".to_string(),
            "test_project".to_string(),
            "test_db".to_string(),
            collections,
        );

        assert_eq!(client.base_url, "https://appwrite.test/v1");
        assert_eq!(client.api_key, "test_key");
    }
}
