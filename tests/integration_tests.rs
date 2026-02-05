// Integration tests for Lume Algo

use lume_algo::core::{Matcher, distance::{haversine_distance, calculate_bounding_box}};
use lume_algo::models::{UserProfile, UserPreferences, ScoringWeights};
use chrono::Utc;

fn create_test_profile(
    id: &str,
    age: u8,
    gender: &str,
    lat: f64,
    lon: f64,
) -> UserProfile {
    UserProfile {
        user_id: id.to_string(),
        name: format!("User {}", id),
        age,
        height_cm: 170,
        hair_color: "brown".to_string(),
        gender: gender.to_string(),
        latitude: lat,
        longitude: lon,
        is_verified: true,
        is_active: true,
        is_timeout: false,
        image_file_ids: vec![],
        description: None,
        sports_preferences: vec!["tennis".to_string()],
        created_at: Utc::now(),
    }
}

fn create_test_preferences(lat: f64, lon: f64) -> UserPreferences {
    UserPreferences {
        user_id: "current_user".to_string(),
        preferred_genders: vec!["female".to_string()],
        min_age: 21,
        max_age: 35,
        min_height_cm: 160,
        max_height_cm: 180,
        preferred_hair_colors: vec![],
        preferred_sports: vec!["tennis".to_string()],
        max_distance_km: 50,
        latitude: lat,
        longitude: lon,
    }
}

#[test]
fn test_integration_end_to_end_matching() {
    let matcher = Matcher::with_default_weights();
    let preferences = create_test_preferences(40.7128, -74.0060); // New York

    // Create diverse candidates
    let candidates = vec![
        create_test_profile("1", 25, "female", 40.72, -74.01),    // Good match
        create_test_profile("2", 28, "female", 40.73, -74.02),    // Good match
        create_test_profile("3", 30, "female", 40.71, -74.00),    // Good match
        create_test_profile("4", 22, "female", 40.70, -73.99),    // Good match
        create_test_profile("5", 40, "female", 40.72, -74.01),    // Too old
        create_test_profile("6", 25, "male", 40.72, -74.01),      // Wrong gender
        create_test_profile("7", 25, "female", 41.5, -74.0),      // Too far
        create_test_profile("8", 25, "female", 40.72, -74.01),    // Duplicate (should be handled)
    ];

    let result = matcher.find_matches(&preferences, candidates, 5);

    // Should have at least 3 good matches
    assert!(result.matches.len() >= 3, "Expected at least 3 matches, got {}", result.matches.len());

    // All matches should be female
    for m in &result.matches {
        assert_eq!(m.gender, "female");
    }

    // All matches should be within age range
    for m in &result.matches {
        assert!(m.age >= 21 && m.age <= 35);
    }

    // All matches should be sorted by score
    for i in 1..result.matches.len() {
        assert!(
            result.matches[i - 1].match_score >= result.matches[i].match_score,
            "Matches not sorted by score"
        );
    }
}

#[test]
fn test_distance_accuracy() {
    // Test known distances
    let nyc_lat = 40.7128;
    let nyc_lon = -74.0060;

    // Distance to same point should be 0
    let distance = haversine_distance(nyc_lat, nyc_lon, nyc_lat, nyc_lon);
    assert!((distance).abs() < 0.01);

    // Distance to nearby point
    let distance = haversine_distance(nyc_lat, nyc_lon, 40.72, -74.01);
    assert!(distance > 0.0 && distance < 2.0, "Expected ~1km, got {}", distance);

    // Distance to LA (approximately 3944 km)
    let la_lat = 34.0522;
    let la_lon = -118.2437;
    let distance = haversine_distance(nyc_lat, nyc_lon, la_lat, la_lon);
    assert!((distance - 3944.0).abs() < 100.0, "Expected ~3944km, got {}", distance);
}

#[test]
fn test_bounding_box_filtering() {
    let center_lat = 40.7128;
    let center_lon = -74.0060;
    let radius_km = 10.0;

    let bbox = calculate_bounding_box(center_lat, center_lon, radius_km);

    // Points within bounding box
    let inside_lat = 40.71;
    let inside_lon = -74.0;

    let distance_to_inside = haversine_distance(center_lat, center_lon, inside_lat, inside_lon);
    assert!(distance_to_inside < radius_km, "Test point should be within radius");

    // Points far outside
    let far_lat = 50.0;
    let far_lon = -80.0;

    let distance_to_far = haversine_distance(center_lat, center_lon, far_lat, far_lon);
    assert!(distance_to_far > radius_km * 10.0, "Test point should be far outside");
}

#[test]
fn test_score_range() {
    let matcher = Matcher::with_default_weights();
    let preferences = create_test_preferences(40.7128, -74.0060);

    let candidates = vec![
        create_test_profile("1", 25, "female", 40.72, -74.01),
        create_test_profile("2", 28, "female", 40.73, -74.02),
        create_test_profile("3", 30, "female", 40.71, -74.00),
    ];

    let result = matcher.find_matches(&preferences, candidates, 10);

    for m in &result.matches {
        assert!(
            m.match_score >= 0.0 && m.match_score <= 100.0,
            "Score {} is out of range [0, 100]",
            m.match_score
        );
    }
}

#[test]
fn test_max_limit_enforcement() {
    let matcher = Matcher::with_default_weights();
    let preferences = create_test_preferences(40.7128, -74.0060);

    let candidates: Vec<UserProfile> = (0..50)
        .map(|i| {
            create_test_profile(
                &i.to_string(),
                25 + (i % 10) as u8,
                "female",
                40.72 + (i as f64 * 0.0001),
                -74.01,
            )
        })
        .collect();

    let result = matcher.find_matches(&preferences, candidates, 10);

    assert!(result.matches.len() <= 10, "Should not exceed limit of 10");
}
