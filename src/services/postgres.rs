use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use std::time::Duration;
use thiserror::Error;

/// Errors that can occur when interacting with PostgreSQL
#[derive(Debug, Error)]
pub enum PostgresError {
    #[error("Connection pool error: {0}")]
    PoolError(#[from] deadpool_postgres::PoolError),

    #[error("SQLx error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    MigrateError(#[from] sqlx::migrate::MigrateError),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Event types for match interactions
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "event_type", rename_all = "lowercase")]
pub enum EventType {
    Viewed,
    Liked,
    Passed,
    Matched,
}

impl From<crate::models::MatchEventType> for EventType {
    fn from(value: crate::models::MatchEventType) -> Self {
        match value {
            crate::models::MatchEventType::Viewed => EventType::Viewed,
            crate::models::MatchEventType::Liked => EventType::Liked,
            crate::models::MatchEventType::Passed => EventType::Passed,
            crate::models::MatchEventType::Matched => EventType::Matched,
        }
    }
}

/// Record of a seen profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeenProfile {
    pub user_id: String,
    pub target_user_id: String,
    pub event_type: EventType,
    pub seen_at: chrono::DateTime<chrono::Utc>,
}

/// PostgreSQL client for tracking seen profiles
///
/// This client maintains a separate database from Appwrite specifically
/// for tracking which profiles a user has already seen, ensuring the
/// matching algorithm doesn't return the same profiles repeatedly.
pub struct PostgresClient {
    pool: PgPool,
}

impl PostgresClient {
    /// Create a new PostgreSQL client from a connection string
    pub async fn new(
        database_url: &str,
        max_connections: u32,
        min_connections: u32,
    ) -> Result<Self, PostgresError> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections as u32)
            .min_connections(min_connections as u32)
            .acquire_timeout(Duration::from_secs(5))
            .idle_timeout(Duration::from_secs(600))
            .test_before_acquire(true)
            .connect(database_url)
            .await?;

        // Run migrations on startup
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    /// Create a new PostgreSQL client from settings
    pub async fn from_settings(
        url: &str,
        max_connections: Option<u32>,
        min_connections: Option<u32>,
        _acquire_timeout_secs: Option<u64>,
        _idle_timeout_secs: Option<u64>,
    ) -> Result<Self, PostgresError> {
        tracing::info!("Connecting to PostgreSQL with URL: {}", url);

        Self::new(
            url,
            max_connections.unwrap_or(10),
            min_connections.unwrap_or(1),
        )
        .await
    }

    /// Record that a user has seen a profile
    ///
    /// Uses INSERT ... ON CONFLICT to handle duplicates gracefully.
    /// If the record already exists, it updates the event_type and seen_at.
    pub async fn record_seen(
        &self,
        user_id: &str,
        target_user_id: &str,
        event_type: EventType,
    ) -> Result<(), PostgresError> {
        let query = r#"
            INSERT INTO seen_profiles (user_id, target_user_id, event_type, seen_at)
            VALUES ($1, $2, $3, NOW())
            ON CONFLICT (user_id, target_user_id)
            DO UPDATE SET
                event_type = EXCLUDED.event_type,
                seen_at = EXCLUDED.seen_at
        "#;

        sqlx::query(query)
            .bind(user_id)
            .bind(target_user_id)
            .bind(&event_type)
            .execute(&self.pool)
            .await?;

        tracing::debug!(
            "Recorded seen profile: {} -> {} ({:?})",
            user_id,
            target_user_id,
            event_type
        );

        Ok(())
    }

    /// Get all user IDs that the given user has already seen
    ///
    /// Returns a vector of target_user_ids that should be excluded
    /// from future matching results.
    pub async fn get_seen_profiles(&self, user_id: &str) -> Result<Vec<String>, PostgresError> {
        let query = r#"
            SELECT target_user_id
            FROM seen_profiles
            WHERE user_id = $1
        "#;

        let rows = sqlx::query(query).bind(user_id).fetch_all(&self.pool).await?;

        let seen_ids: Vec<String> = rows
            .iter()
            .map(|row| row.get("target_user_id"))
            .collect();

        tracing::debug!("User {} has seen {} profiles", user_id, seen_ids.len());

        Ok(seen_ids)
    }

    /// Get seen profiles with pagination (for debugging/admin)
    pub async fn get_seen_profiles_paginated(
        &self,
        user_id: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<SeenProfile>, PostgresError> {
        let query = r#"
            SELECT user_id, target_user_id, event_type, seen_at
            FROM seen_profiles
            WHERE user_id = $1
            ORDER BY seen_at DESC
            LIMIT $2 OFFSET $3
        "#;

        let rows = sqlx::query(query)
            .bind(user_id)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await?;

        let profiles: Result<Vec<SeenProfile>, _> = rows
            .iter()
            .map(|row| {
                Ok(SeenProfile {
                    user_id: row.get("user_id"),
                    target_user_id: row.get("target_user_id"),
                    event_type: row.get("event_type"),
                    seen_at: row.get("seen_at"),
                })
            })
            .collect();

        profiles
    }

    /// Remove a seen profile record (e.g., if a match was reset)
    pub async fn remove_seen(
        &self,
        user_id: &str,
        target_user_id: &str,
    ) -> Result<bool, PostgresError> {
        let query = r#"
            DELETE FROM seen_profiles
            WHERE user_id = $1 AND target_user_id = $2
        "#;

        let result = sqlx::query(query)
            .bind(user_id)
            .bind(target_user_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Clear all seen profiles for a user
    pub async fn clear_seen_profiles(&self, user_id: &str) -> Result<u64, PostgresError> {
        let query = r#"
            DELETE FROM seen_profiles
            WHERE user_id = $1
        "#;

        let result = sqlx::query(query).bind(user_id).execute(&self.pool).await?;

        tracing::info!(
            "Cleared {} seen profiles for user {}",
            result.rows_affected(),
            user_id
        );

        Ok(result.rows_affected())
    }

    /// Get statistics about seen profiles for a user
    pub async fn get_seen_stats(&self, user_id: &str) -> Result<SeenStats, PostgresError> {
        let query = r#"
            SELECT
                COUNT(*) as total_seen,
                COUNT(*) FILTER (WHERE event_type = 'viewed') as viewed,
                COUNT(*) FILTER (WHERE event_type = 'liked') as liked,
                COUNT(*) FILTER (WHERE event_type = 'passed') as passed,
                COUNT(*) FILTER (WHERE event_type = 'matched') as matched,
                MAX(seen_at) as last_seen_at
            FROM seen_profiles
            WHERE user_id = $1
        "#;

        let row = sqlx::query(query).bind(user_id).fetch_one(&self.pool).await?;

        Ok(SeenStats {
            user_id: user_id.to_string(),
            total_seen: row.get("total_seen"),
            viewed: row.get("viewed"),
            liked: row.get("liked"),
            passed: row.get("passed"),
            matched: row.get("matched"),
            last_seen_at: row.get("last_seen_at"),
        })
    }

    /// Health check for the database connection
    pub async fn health_check(&self) -> Result<bool, PostgresError> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| true)
            .map_err(Into::into)
    }
}

/// Statistics about a user's seen profiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeenStats {
    pub user_id: String,
    pub total_seen: i64,
    pub viewed: i64,
    pub liked: i64,
    pub passed: i64,
    pub matched: i64,
    pub last_seen_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_conversion() {
        // Test EventType::Viewed can be converted to string
        let event_type = EventType::Viewed;
        assert_eq!(format!("{:?}", event_type), "Viewed");
    }
}
