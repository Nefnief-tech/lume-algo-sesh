use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::path::Path;

/// Application configuration
#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub server: ServerSettings,
    pub appwrite: AppwriteSettings,
    pub collection: CollectionSettings,
    pub database: DatabaseSettings,
    pub cache: CacheSettings,
    pub matching: MatchingSettings,
    pub scoring: ScoringSettings,
    pub logging: LoggingSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppwriteSettings {
    pub endpoint: String,
    pub api_key: String,
    pub project_id: String,
    pub database_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CollectionSettings {
    pub user_profiles: String,
    pub user_preferences: String,
    pub match_events: String,
    pub user_matches: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseSettings {
    pub url: String,
    pub max_connections: Option<u32>,
    pub min_connections: Option<u32>,
    pub acquire_timeout_secs: Option<u64>,
    pub idle_timeout_secs: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CacheSettings {
    pub redis_url: String,
    pub pool_size: Option<u32>,
    pub ttl_secs: Option<u64>,
    pub connection_timeout_secs: Option<u64>,
    pub l1_cache_size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatchingSettings {
    pub max_distance_km: Option<u16>,
    pub default_limit: Option<u8>,
    pub max_limit: Option<u8>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScoringSettings {
    #[serde(default)]
    pub weights: WeightsConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WeightsConfig {
    #[serde(default = "default_distance_weight")]
    pub distance: f64,
    #[serde(default = "default_age_weight")]
    pub age: f64,
    #[serde(default = "default_sports_weight")]
    pub sports: f64,
    #[serde(default = "default_verified_weight")]
    pub verified: f64,
    #[serde(default = "default_height_weight")]
    pub height: f64,
}

impl Default for WeightsConfig {
    fn default() -> Self {
        Self {
            distance: default_distance_weight(),
            age: default_age_weight(),
            sports: default_sports_weight(),
            verified: default_verified_weight(),
            height: default_height_weight(),
        }
    }
}

fn default_distance_weight() -> f64 { 0.35 }
fn default_age_weight() -> f64 { 0.20 }
fn default_sports_weight() -> f64 { 0.25 }
fn default_verified_weight() -> f64 { 0.10 }
fn default_height_weight() -> f64 { 0.10 }

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingSettings {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_format")]
    pub format: String,
}

fn default_log_level() -> String { "info".to_string() }
fn default_log_format() -> String { "json".to_string() }

impl Settings {
    /// Load configuration from file and environment variables
    ///
    /// Configuration is loaded in the following order (later overrides earlier):
    /// 1. Default values in the struct
    /// 2. Configuration file (config/default.toml)
    /// 3. Environment variables (prefixed with LUME_)
    pub fn load() -> Result<Self, ConfigError> {
        let mut settings = Config::builder()
            // Add default config file
            .add_source(File::with_name("config/default").required(false))
            // Add local config file (for development overrides)
            .add_source(File::with_name("config/local").required(false))
            // Add environment variables (prefixed with LUME_)
            // e.g., LUME_SERVER__PORT -> server.port
            .add_source(
                Environment::with_prefix("LUME")
                    .prefix_separator("__")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;

        // Substitute environment variables in string values
        // e.g., ${VAR_NAME} gets replaced with the value of VAR_NAME
        settings = substitute_env_vars(settings)?;

        settings.try_deserialize()
    }

    /// Load configuration from a custom path
    pub fn load_from<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let settings = Config::builder()
            .add_source(File::from(path.as_ref()))
            .add_source(
                Environment::with_prefix("LUME")
                    .prefix_separator("__")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;

        settings.try_deserialize()
    }
}

/// Substitute environment variables in config values
/// Variables are in the format ${VAR_NAME} or ${VAR_NAME:-default}
fn substitute_env_vars(settings: Config) -> Result<Config, ConfigError> {
    use std::env;

    // Get the database URL from environment (with default)
    // We check DATABASE_URL first, then LUME_DATABASE__URL
    let database_url = env::var("DATABASE_URL")
        .or_else(|_| env::var("LUME_DATABASE__URL"))
        .unwrap_or_else(|_| "postgres://lume:password@localhost:5432/lume_algo".to_string());

    // Get Appwrite settings from environment
    let appwrite_endpoint = env::var("LUME_APPWRITE__ENDPOINT")
        .ok();
    let appwrite_api_key = env::var("LUME_APPWRITE__API_KEY")
        .ok();
    let appwrite_project_id = env::var("LUME_APPWRITE__PROJECT_ID")
        .ok();
    let appwrite_database_id = env::var("LUME_APPWRITE__DATABASE_ID")
        .ok();

    // Build a new config with the overrides
    let mut builder = Config::builder()
        .add_source(settings)
        .set_override("database.url", database_url)?;

    if let Some(endpoint) = appwrite_endpoint {
        builder = builder.set_override("appwrite.endpoint", endpoint)?;
    }
    if let Some(api_key) = appwrite_api_key {
        builder = builder.set_override("appwrite.api_key", api_key)?;
    }
    if let Some(project_id) = appwrite_project_id {
        builder = builder.set_override("appwrite.project_id", project_id)?;
    }
    if let Some(database_id) = appwrite_database_id {
        builder = builder.set_override("appwrite.database_id", database_id)?;
    }

    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_weights() {
        let weights = WeightsConfig::default();
        assert_eq!(weights.distance, 0.35);
        assert_eq!(weights.age, 0.20);
        assert_eq!(weights.sports, 0.25);
        assert_eq!(weights.verified, 0.10);
        assert_eq!(weights.height, 0.10);
    }

    #[test]
    fn test_default_logging() {
        let level = default_log_level();
        let format = default_log_format();
        assert_eq!(level, "info");
        assert_eq!(format, "json");
    }
}
