// Unit tests for Lume Algo

use lume_algo::core::{
    distance::{haversine_distance, calculate_bounding_box, is_within_bounding_box},
    filters::{matches_demographics, calculate_preference_score},
    scoring::calculate_match_score,
};
use lume_algo::models::{UserProfile, UserPreferences, ScoringWeights};
use chrono::Utc;

#[test]
fn test_haversine_distance_zero() {
    let distance = haversine_distance(40.7128, -74.0060, 40.7128, -74.0060);
    assert!(distance < 0.01);
}

#[test]
fn test_haversine_distance_manhattan_to_brooklyn() {
    // Manhattan to Brooklyn is approximately 5-10 km
    let manhattan_lat = 40.7580;
    let manhattan_lon = -73.9855;
    let brooklyn_lat = 40.6782;
    let brooklyn_lon = -73.9442;

    let distance = haversine_distance(manhattan_lat, manhattan_lon, brooklyn_lat, brooklyn_lon);
    assert!(distance > 5.0 && distance < 15.0);
}

#[test]
fn test_bounding_box_creation() {
    let bbox = calculate_bounding_box(40.7128, -74.0060, 10.0);

    assert!(bbox.min_lat < 40.7128);
    assert!(bbox.max_lat > 40.7128);
    assert!(bbox.min_lon < -74.0060);
    assert!(bbox.max_lon > -74.0060);

    // Bounding box should be roughly 0.18 degrees in latitude (10km / 111km per degree)
    let lat_span = bbox.max_lat - bbox.min_lat;
    assert!((lat_span - 0.18).abs() < 0.02);
}

#[test]
fn test_point_within_bbox() {
    let bbox = calculate_bounding_box(40.7128, -74.0060, 10.0);

    // Center point is within
    assert!(is_within_bounding_box(40.7128, -74.0060, &bbox));

    // Close point is within
    assert!(is_within_bounding_box(40.71, -74.0, &bbox));

    // Far point is not within
    assert!(!is_within_bounding_box(50.0, -80.0, &bbox));

    // Point just outside latitude is not within
    assert!(!is_within_bounding_box(bbox.max_lat + 0.01, -74.0, &bbox));
}

#[test]
fn test_demographics_match_pass() {
    let profile = UserProfile {
        user_id: "test".to_string(),
        name: "Test".to_string(),
        age: 25,
        height_cm: 170,
        hair_color: "brown".to_string(),
        gender: "female".to_string(),
        latitude: 40.7128,
        longitude: -74.0060,
        is_verified: true,
        is_active: true,
        is_timeout: false,
        image_file_ids: vec![],
        description: None,
        sports_preferences: vec![],
        created_at: Utc::now(),
    };

    let preferences = UserPreferences {
        user_id: "pref".to_string(),
        preferred_genders: vec!["female".to_string()],
        min_age: 21,
        max_age: 30,
        min_height_cm: 160,
        max_height_cm: 180,
        preferred_hair_colors: vec![],
        preferred_sports: vec![],
        max_distance_km: 50,
        latitude: 40.7128,
        longitude: -74.0060,
    };

    assert!(matches_demographics(&profile, &preferences));
}

#[test]
fn test_demographics_fail_inactive() {
    let profile = UserProfile {
        user_id: "test".to_string(),
        name: "Test".to_string(),
        age: 25,
        height_cm: 170,
        hair_color: "brown".to_string(),
        gender: "female".to_string(),
        latitude: 40.7128,
        longitude: -74.0060,
        is_verified: true,
        is_active: false, // Inactive
        is_timeout: false,
        image_file_ids: vec![],
        description: None,
        sports_preferences: vec![],
        created_at: Utc::now(),
    };

    let preferences = UserPreferences {
        user_id: "pref".to_string(),
        preferred_genders: vec!["female".to_string()],
        min_age: 21,
        max_age: 30,
        min_height_cm: 160,
        max_height_cm: 180,
        preferred_hair_colors: vec![],
        preferred_sports: vec![],
        max_distance_km: 50,
        latitude: 40.7128,
        longitude: -74.0060,
    };

    assert!(!matches_demographics(&profile, &preferences));
}

#[test]
fn test_demographics_fail_age() {
    let profile = UserProfile {
        user_id: "test".to_string(),
        name: "Test".to_string(),
        age: 40, // Too old
        height_cm: 170,
        hair_color: "brown".to_string(),
        gender: "female".to_string(),
        latitude: 40.7128,
        longitude: -74.0060,
        is_verified: true,
        is_active: true,
        is_timeout: false,
        image_file_ids: vec![],
        description: None,
        sports_preferences: vec![],
        created_at: Utc::now(),
    };

    let preferences = UserPreferences {
        user_id: "pref".to_string(),
        preferred_genders: vec!["female".to_string()],
        min_age: 21,
        max_age: 30, // Max 30, profile is 40
        min_height_cm: 160,
        max_height_cm: 180,
        preferred_hair_colors: vec![],
        preferred_sports: vec![],
        max_distance_km: 50,
        latitude: 40.7128,
        longitude: -74.0060,
    };

    assert!(!matches_demographics(&profile, &preferences));
}

#[test]
fn test_preference_score_with_shared_sports() {
    let profile = UserProfile {
        user_id: "test".to_string(),
        name: "Test".to_string(),
        age: 25,
        height_cm: 170,
        hair_color: "blonde".to_string(),
        gender: "female".to_string(),
        latitude: 40.7128,
        longitude: -74.0060,
        is_verified: true,
        is_active: true,
        is_timeout: false,
        image_file_ids: vec![],
        description: None,
        sports_preferences: vec!["tennis".to_string(), "swimming".to_string()],
        created_at: Utc::now(),
    };

    let preferences = UserPreferences {
        user_id: "pref".to_string(),
        preferred_genders: vec![],
        min_age: 20,
        max_age: 30,
        min_height_cm: 160,
        max_height_cm: 180,
        preferred_hair_colors: vec!["blonde".to_string()],
        preferred_sports: vec!["tennis".to_string(), "basketball".to_string()],
        max_distance_km: 50,
        latitude: 40.7128,
        longitude: -74.0060,
    };

    let (score, shared) = calculate_preference_score(&profile, &preferences);

    assert!(score > 0.0, "Preference score should be positive");
    assert_eq!(shared, vec!["tennis"], "Should have one shared sport");
}

#[test]
fn test_match_score_within_valid_range() {
    let profile = UserProfile {
        user_id: "test".to_string(),
        name: "Test".to_string(),
        age: 25,
        height_cm: 170,
        hair_color: "brown".to_string(),
        gender: "female".to_string(),
        latitude: 40.7128,
        longitude: -74.0060,
        is_verified: true,
        is_active: true,
        is_timeout: false,
        image_file_ids: vec![],
        description: None,
        sports_preferences: vec!["tennis".to_string()],
        created_at: Utc::now(),
    };

    let preferences = UserPreferences {
        user_id: "pref".to_string(),
        preferred_genders: vec![],
        min_age: 21,
        max_age: 30,
        min_height_cm: 160,
        max_height_cm: 180,
        preferred_hair_colors: vec![],
        preferred_sports: vec!["tennis".to_string()],
        max_distance_km: 50,
        latitude: 40.7128,
        longitude: -74.0060,
    };

    let weights = ScoringWeights::default();
    let (score, _) = calculate_match_score(&profile, &preferences, &weights);

    assert!(score >= 0.0 && score <= 100.0, "Score should be in valid range");
}

#[test]
fn test_verified_user_scores_higher() {
    let verified_profile = UserProfile {
        user_id: "test1".to_string(),
        name: "Test".to_string(),
        age: 25,
        height_cm: 170,
        hair_color: "brown".to_string(),
        gender: "female".to_string(),
        latitude: 40.7128,
        longitude: -74.0060,
        is_verified: true,
        is_active: true,
        is_timeout: false,
        image_file_ids: vec![],
        description: None,
        sports_preferences: vec![],
        created_at: Utc::now(),
    };

    let unverified_profile = UserProfile {
        user_id: "test2".to_string(),
        name: "Test".to_string(),
        age: 25,
        height_cm: 170,
        hair_color: "brown".to_string(),
        gender: "female".to_string(),
        latitude: 40.7128,
        longitude: -74.0060,
        is_verified: false,
        is_active: true,
        is_timeout: false,
        image_file_ids: vec![],
        description: None,
        sports_preferences: vec![],
        created_at: Utc::now(),
    };

    let preferences = UserPreferences {
        user_id: "pref".to_string(),
        preferred_genders: vec![],
        min_age: 21,
        max_age: 30,
        min_height_cm: 160,
        max_height_cm: 180,
        preferred_hair_colors: vec![],
        preferred_sports: vec![],
        max_distance_km: 50,
        latitude: 40.7128,
        longitude: -74.0060,
    };

    let weights = ScoringWeights::default();
    let (verified_score, _) = calculate_match_score(&verified_profile, &preferences, &weights);
    let (unverified_score, _) = calculate_match_score(&unverified_profile, &preferences, &weights);

    assert!(
        verified_score > unverified_score,
        "Verified users should score higher"
    );
}
