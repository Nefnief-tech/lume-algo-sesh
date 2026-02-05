mod config;
mod core;
mod models;
mod routes;
mod services;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer, HttpResponse, middleware, error, http::StatusCode};
use config::Settings;
use routes::matches::AppState;
use services::{AppwriteClient, AppwriteCollections, CacheManager, PostgresClient};
use core::Matcher;
use models::ScoringWeights;
use std::sync::Arc;
use tracing::{info, error};

/// JSON error response for JSON payload errors
#[derive(Debug, serde::Serialize)]
pub struct JsonError {
    pub error: String,
    pub message: String,
    pub status_code: u16,
}

impl std::fmt::Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.error, self.message)
    }
}

impl std::error::Error for JsonError {}

impl error::ResponseError for JsonError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(StatusCode::from_u16(self.status_code).unwrap_or(StatusCode::BAD_REQUEST))
            .content_type("application/json")
            .body(serde_json::to_string(self).unwrap())
    }
}

/// Handle JSON payload errors
pub fn handle_json_payload_error(err: error::JsonPayloadError, req: &actix_web::HttpRequest) -> actix_web::Error {
    tracing::info!("JSON payload error on {}: {}", req.path(), err);
    JsonError {
        error: "invalid_json".to_string(),
        message: format!("Invalid JSON: {}", err),
        status_code: 400,
    }
    .into()
}

/// Handle query payload errors
pub fn handle_query_payload_error(err: error::QueryPayloadError, _req: &actix_web::HttpRequest) -> actix_web::Error {
    JsonError {
        error: "invalid_query".to_string(),
        message: format!("Invalid query: {}", err),
        status_code: 400,
    }
    .into()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load .env file if present
    dotenv::dotenv().ok();

    // Initialize logging
    let log_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    let log_format = std::env::var("LOG_FORMAT").unwrap_or_else(|_| "json".to_string());

    let subscriber = tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true);

    if log_format == "pretty" {
        subscriber.pretty().init();
    } else {
        subscriber.init();
    }

    info!("Starting Lume Algo matching service...");

    // Load configuration
    let settings = Settings::load().unwrap_or_else(|e| {
        error!("Failed to load configuration: {}", e);
        panic!("Configuration error: {}", e);
    });

    info!("Configuration loaded successfully");

    // Initialize Appwrite client
    let appwrite_collections = AppwriteCollections {
        user_profiles: settings.collection.user_profiles,
        user_preferences: settings.collection.user_preferences,
        match_events: settings.collection.match_events,
        user_matches: settings.collection.user_matches,
    };

    let appwrite = Arc::new(AppwriteClient::new(
        settings.appwrite.endpoint,
        settings.appwrite.api_key,
        settings.appwrite.project_id,
        settings.appwrite.database_id,
        appwrite_collections,
    ));

    info!("Appwrite client initialized");

    // Initialize cache manager (optional - app can work without it)
    let cache_ttl = settings.cache.ttl_secs.unwrap_or(300);
    let l1_cache_size = settings.cache.l1_cache_size.unwrap_or(1000);

    let cache = match CacheManager::new(
        &settings.cache.redis_url,
        l1_cache_size,
        cache_ttl,
    ).await {
        Ok(c) => {
            info!("Cache manager initialized (L1: {} entries, TTL: {}s)", l1_cache_size, cache_ttl);
            Arc::new(c)
        }
        Err(e) => {
            error!("Failed to connect to Redis ({}), running without cache", e);
            // Create a dummy cache manager that fails gracefully
            // For now, we'll continue without cache - seen profiles still work via PostgreSQL
            error!("Caching disabled - seen profiles will still be tracked via PostgreSQL");
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Redis connection required"));
        }
    };

    // Initialize PostgreSQL client
    let db_max_conn = settings.database.max_connections.unwrap_or(10);
    let db_min_conn = settings.database.min_connections.unwrap_or(1);

    let postgres = Arc::new(
        PostgresClient::from_settings(
            &settings.database.url,
            Some(db_max_conn),
            Some(db_min_conn),
            settings.database.acquire_timeout_secs,
            settings.database.idle_timeout_secs,
        )
        .await
        .unwrap_or_else(|e| {
            error!("Failed to connect to PostgreSQL: {}", e);
            panic!("PostgreSQL connection error: {}", e);
        }),
    );

    info!("PostgreSQL client initialized (max: {} connections)", db_max_conn);

    // Initialize matcher with configured weights
    let weights = ScoringWeights {
        distance: settings.scoring.weights.distance,
        age: settings.scoring.weights.age,
        sports: settings.scoring.weights.sports,
        verified: settings.scoring.weights.verified,
        height: settings.scoring.weights.height,
    };

    let matcher = Matcher::new(weights);

    info!("Matcher initialized with weights: {:?}", weights);

    // Build application state
    let app_state = AppState {
        appwrite,
        cache,
        postgres,
        matcher,
    };

    // Configure HTTP server
    let host = settings.server.host.clone();
    let port = settings.server.port;
    let workers = settings.server.workers.unwrap_or(4);

    info!("Starting HTTP server on {}:{}", host, port);

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .app_data(web::JsonConfig::default().error_handler(handle_json_payload_error))
            .app_data(web::QueryConfig::default().error_handler(handle_query_payload_error))
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(routes::configure_routes)
    })
    .workers(workers)
    .bind((host, port))?
    .run()
    .await
}
