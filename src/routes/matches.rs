use actix_web::{web, HttpResponse, Responder};
use validator::Validate;
use crate::models::{FindMatchesRequest, RecordEventRequest, FindMatchesResponse, HealthResponse, RecordEventResponse, ErrorResponse, MatchEvent, MatchEventType};
use crate::services::{AppwriteClient, CacheManager, CacheKey, PostgresClient, EventType};
use crate::core::Matcher;
use std::sync::Arc;

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub appwrite: Arc<AppwriteClient>,
    pub cache: Arc<CacheManager>,
    pub postgres: Arc<PostgresClient>,
    pub matcher: Matcher,
}

/// Configure all match-related routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/health", web::get().to(health_check))
        .route("/matches/find", web::post().to(find_matches))
        .route("/matches/event", web::post().to(record_event))
        .route("/matches/seen", web::get().to(get_seen_profiles))
        .route("/debug/echo", web::post().to(debug_echo));
}

/// Health check endpoint
async fn health_check(state: web::Data<AppState>) -> impl Responder {
    // Check PostgreSQL health
    let pg_healthy = state.postgres.health_check().await.unwrap_or(false);

    let status = if pg_healthy { "healthy" } else { "degraded" };

    HttpResponse::Ok().json(HealthResponse {
        status: status.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now(),
    })
}

/// Debug endpoint to echo raw JSON for debugging
async fn debug_echo(
    body: web::Bytes,
    req: actix_web::HttpRequest,
) -> impl Responder {
    let body_str = String::from_utf8_lossy(&body);
    tracing::info!("DEBUG echo - path: {}, method: {}, body: {}", req.path(), req.method(), body_str);
    HttpResponse::Ok().json(serde_json::json!({
        "path": req.path(),
        "method": req.method().to_string(),
        "body": body_str,
    }))
}

/// Find matches endpoint
///
/// POST /api/v1/matches/find
///
/// Request body:
/// ```json
/// {
///   "userId": "string",
///   "limit": 20,
///   "excludeUserIds": ["string"],
///   "cursor": "string"
/// }
/// ```
async fn find_matches(
    state: web::Data<AppState>,
    req: web::Json<FindMatchesRequest>,
    http_req: actix_web::HttpRequest,
) -> impl Responder {
    // Validate request
    if let Err(errors) = req.validate() {
        tracing::info!("Validation failed for find_matches request: field_errors={:?}", errors);
        tracing::info!("Request data: userId={:?}, limit={:?}, excludeUserIds={:?}",
            req.user_id, req.limit, req.exclude_user_ids);
        tracing::info!("Request path: {}, method: {}", http_req.path(), http_req.method());
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "Validation failed".to_string(),
            message: errors.to_string(),
            status_code: 400,
        });
    }

    let user_id = &req.user_id;
    // Cap limit at 100 to prevent excessive queries
    let limit = req.limit.min(100) as usize;

    tracing::info!("Finding matches for user: {}, limit: {}", user_id, limit);

    // Note: Caching disabled for matches endpoint to ensure seen profiles are always up-to-date

    // Fetch already seen profiles from PostgreSQL to prevent repeats
    let mut seen_profile_ids = match state.postgres.get_seen_profiles(user_id).await {
        Ok(ids) => ids,
        Err(e) => {
            tracing::warn!("Failed to fetch seen profiles for {}, proceeding without filtering: {}", user_id, e);
            vec![]
        }
    };

    // Add client-provided exclude IDs (if any)
    seen_profile_ids.extend(req.exclude_user_ids.clone());

    tracing::debug!("Excluding {} seen profiles for user {}", seen_profile_ids.len(), user_id);

    // Fetch user profile to get location data
    let user_profile = match state.appwrite.get_profile(user_id).await {
        Ok(profile) => profile,
        Err(e) => {
            tracing::error!("Failed to fetch profile for {}: {}", user_id, e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch user profile".to_string(),
                message: e.to_string(),
                status_code: 500,
            });
        }
    };

    // Fetch user preferences from Appwrite
    let mut preferences = match state.appwrite.get_preferences(user_id).await {
        Ok(prefs) => prefs,
        Err(e) => {
            tracing::error!("Failed to fetch preferences for {}: {}", user_id, e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch preferences".to_string(),
                message: e.to_string(),
                status_code: 500,
            });
        }
    };

    // Update preferences with location from user profile
    preferences.latitude = user_profile.latitude;
    preferences.longitude = user_profile.longitude;

    // Query candidates from Appwrite
    let candidates = match state
        .appwrite
        .query_candidates(user_id, &preferences, &seen_profile_ids, limit * 5)
        .await
    {
        Ok(candidates) => candidates,
        Err(e) => {
            tracing::error!("Failed to query candidates for {}: {}", user_id, e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to query candidates".to_string(),
                message: e.to_string(),
                status_code: 500,
            });
        }
    };

    tracing::debug!("Found {} candidates for {}", candidates.len(), user_id);

    // Run matching algorithm
    let result = state
        .matcher
        .find_matches(&preferences, candidates, limit);

    // Build response
    let response = FindMatchesResponse {
        matches: result.matches,
        next_cursor: None,  // TODO: implement cursor-based pagination
        total_results: result.total_candidates,
    };

    tracing::info!(
        "Returning {} matches for user {} (from {} candidates)",
        response.matches.len(),
        user_id,
        result.total_candidates
    );

    HttpResponse::Ok().json(response)
}

/// Record match event endpoint
///
/// POST /api/v1/matches/event
///
/// Request body:
/// ```json
/// {
///   "userId": "string",
///   "targetUserId": "string",
///   "eventType": "viewed|liked|passed|matched"
/// }
/// ```
async fn record_event(
    state: web::Data<AppState>,
    req: web::Json<RecordEventRequest>,
) -> impl Responder {
    // Validate request
    if let Err(errors) = req.validate() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "Validation failed".to_string(),
            message: errors.to_string(),
            status_code: 400,
        });
    }

    // Parse event type
    let event_type = match req.event_type.to_lowercase().as_str() {
        "viewed" => MatchEventType::Viewed,
        "liked" => MatchEventType::Liked,
        "passed" => MatchEventType::Passed,
        "matched" => MatchEventType::Matched,
        _ => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid event type".to_string(),
                message: "Event type must be one of: viewed, liked, passed, matched".to_string(),
                status_code: 400,
            });
        }
    };

    let event = MatchEvent {
        user_id: req.user_id.clone(),
        target_user_id: req.target_user_id.clone(),
        event_type,
        created_at: chrono::Utc::now(),
    };

    // Record event in PostgreSQL for seen profile tracking (primary source)
    let pg_event_type = EventType::from(event.event_type.clone());
    let postgres_result = state.postgres.record_seen(
        &req.user_id,
        &req.target_user_id,
        pg_event_type,
    ).await;

    // Record event in Appwrite (best-effort, for analytics/backup)
    let appwrite_result = state.appwrite.record_event(event.clone()).await;

    // Handle results - PostgreSQL is the critical one
    match postgres_result {
        Ok(_) => {
            // PostgreSQL succeeded - this is what matters for seen profile tracking
            if let Err(e) = &appwrite_result {
                // Log Appwrite failure but don't fail the request
                tracing::warn!("Event recorded in PostgreSQL but Appwrite recording failed: {}", e);
            } else {
                tracing::debug!(
                    "Recorded event: {} -> {:?} (both PostgreSQL and Appwrite)",
                    req.user_id,
                    req.event_type
                );
            }

            // Invalidate cache for this user
            let cache_key = CacheKey::matches(&req.user_id);
            if let Err(e) = state.cache.delete(&cache_key).await {
                tracing::warn!("Failed to invalidate cache: {}", e);
            }

            HttpResponse::Ok().json(RecordEventResponse {
                success: true,
                event_id: uuid::Uuid::new_v4().to_string(),
            })
        }
        Err(e) => {
            // PostgreSQL failed - this is the critical failure
            tracing::error!("Failed to record event in PostgreSQL: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to record event".to_string(),
                message: e.to_string(),
                status_code: 500,
            })
        }
    }
}

/// Get seen profiles for a user
///
/// GET /api/v1/matches/seen?userId={userId}
///
/// Returns a list of profile IDs the user has already seen, for client-side
/// synchronization and debugging purposes.
async fn get_seen_profiles(
    state: web::Data<AppState>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let user_id = match query.get("userId") {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Missing userId parameter".to_string(),
                message: "userId query parameter is required".to_string(),
                status_code: 400,
            });
        }
    };

    match state.postgres.get_seen_profiles(user_id).await {
        Ok(seen_ids) => {
            HttpResponse::Ok().json(serde_json::json!({
                "userId": user_id,
                "seenProfiles": seen_ids,
                "count": seen_ids.len(),
            }))
        }
        Err(e) => {
            tracing::error!("Failed to fetch seen profiles for {}: {}", user_id, e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch seen profiles".to_string(),
                message: e.to_string(),
                status_code: 500,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_response() {
        let response = HealthResponse {
            status: "healthy".to_string(),
            version: "0.1.0".to_string(),
            timestamp: chrono::Utc::now(),
        };

        assert_eq!(response.status, "healthy");
    }
}
