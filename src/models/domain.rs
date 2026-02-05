use serde::{Deserialize, Serialize};

/// User profile with demographic and location data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    #[serde(rename = "userId")]
    pub user_id: String,
    pub name: String,
    pub age: u8,
    #[serde(rename = "heightCm")]
    pub height_cm: u16,
    #[serde(rename = "hairColor")]
    pub hair_color: String,
    pub gender: String,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(rename = "isVerified", default)]
    pub is_verified: Option<bool>,
    #[serde(rename = "isActive", default = "default_true")]
    pub is_active: bool,
    #[serde(rename = "isTimeout", default)]
    pub is_timeout: Option<bool>,
    #[serde(rename = "imageFileIds", default)]
    pub image_file_ids: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "sportsPreferences", default)]
    pub sports_preferences: Vec<String>,
    #[serde(default)]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl UserProfile {
    /// Helper to get is_verified as a bool, defaulting to false
    pub fn verified(&self) -> bool {
        self.is_verified.unwrap_or(false)
    }

    /// Helper to get is_timeout as a bool, defaulting to false
    pub fn timeout(&self) -> bool {
        self.is_timeout.unwrap_or(false)
    }
}

fn default_true() -> bool { true }

/// User matching preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "preferredGenders")]
    pub preferred_genders: Vec<String>,
    #[serde(rename = "minAge")]
    pub min_age: u8,
    #[serde(rename = "maxAge")]
    pub max_age: u8,
    #[serde(rename = "minHeightCm")]
    pub min_height_cm: u16,
    #[serde(rename = "maxHeightCm")]
    pub max_height_cm: u16,
    #[serde(rename = "preferredHairColors")]
    pub preferred_hair_colors: Vec<String>,
    #[serde(rename = "preferredSports")]
    pub preferred_sports: Vec<String>,
    #[serde(rename = "maxDistanceKm")]
    pub max_distance_km: u16,
    #[serde(default)]
    pub latitude: f64,
    #[serde(default)]
    pub longitude: f64,
}

/// Match event for tracking user interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchEvent {
    pub user_id: String,
    pub target_user_id: String,
    pub event_type: MatchEventType,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchEventType {
    Viewed,
    Liked,
    Passed,
    Matched,
}

/// Cached mutual match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMatch {
    pub user1_id: String,
    pub user2_id: String,
    pub matched_at: chrono::DateTime<chrono::Utc>,
    pub is_active: bool,
}

/// Scored match result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredMatch {
    #[serde(rename = "userId")]
    pub user_id: String,
    pub name: String,
    pub age: u8,
    #[serde(rename = "heightCm")]
    pub height_cm: u16,
    #[serde(rename = "hairColor")]
    pub hair_color: String,
    pub gender: String,
    #[serde(rename = "distanceKm")]
    pub distance_km: f64,
    #[serde(rename = "matchScore")]
    pub match_score: f64,
    #[serde(rename = "sharedSports")]
    pub shared_sports: Vec<String>,
    #[serde(rename = "isVerified")]
    pub is_verified: bool,
    #[serde(rename = "imageFileIds")]
    pub image_file_ids: Vec<String>,
    pub description: Option<String>,
}

/// Geospatial bounding box
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lon: f64,
    pub max_lon: f64,
}

/// Candidate query parameters
#[derive(Debug, Clone)]
pub struct CandidateQuery {
    pub bounding_box: BoundingBox,
    pub preferred_genders: Vec<String>,
    pub min_age: u8,
    pub max_age: u8,
    pub min_height_cm: u16,
    pub max_height_cm: u16,
    pub exclude_user_ids: Vec<String>,
    pub limit: usize,
}

/// Scoring weights
#[derive(Debug, Clone, Copy)]
pub struct ScoringWeights {
    pub distance: f64,
    pub age: f64,
    pub sports: f64,
    pub verified: f64,
    pub height: f64,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            distance: 0.35,
            age: 0.20,
            sports: 0.25,
            verified: 0.10,
            height: 0.10,
        }
    }
}
