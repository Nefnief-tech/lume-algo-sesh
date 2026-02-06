use crate::models::{UserProfile, UserPreferences, CandidateQuery};

/// Check if a profile matches the user's demographic preferences
///
/// This is Stage 2 of the multi-stage filtering pipeline.
#[inline]
pub fn matches_demographics(
    profile: &UserProfile,
    preferences: &UserPreferences,
) -> bool {
    // Skip if not active or is timed out
    if !profile.is_active || profile.timeout() {
        return false;
    }

    // Check gender preference
    if !preferences.preferred_genders.is_empty()
        && !preferences.preferred_genders.contains(&profile.gender) {
        return false;
    }

    // Check age range
    if profile.age < preferences.min_age || profile.age > preferences.max_age {
        return false;
    }

    // Check height range
    if profile.height_cm < preferences.min_height_cm
        || profile.height_cm > preferences.max_height_cm {
        return false;
    }

    true
}

/// Check if a profile matches the user's soft preferences
///
/// This is Stage 3 - preference matching for scoring.
/// Returns a score factor (0.0 to 1.0) for soft preference alignment.
#[inline]
pub fn calculate_preference_score(
    profile: &UserProfile,
    preferences: &UserPreferences,
) -> (f64, Vec<String>) {
    let mut score = 0.0;
    let mut max_score = 0.0;
    let mut shared_sports = Vec::new();

    // Hair color preference (0 or 1 point)
    max_score += 1.0;
    if preferences.preferred_hair_colors.is_empty()
        || preferences.preferred_hair_colors.contains(&profile.hair_color) {
        score += 1.0;
    }

    // Sports preference - count overlapping sports
    for sport in &profile.sports_preferences {
        if preferences.preferred_sports.contains(sport) {
            shared_sports.push(sport.clone());
        }
    }

    // Normalize sports score (more shared sports = better, but diminishing returns)
    let shared_count = shared_sports.len() as f64;
    let sports_score = if shared_count > 0.0 {
        (shared_count.min(5.0) / 5.0) * 2.0  // Max 2 points for sports
    } else {
        0.0
    };
    score += sports_score;
    max_score += 2.0;

    // Normalize to 0-1 range
    let normalized = if max_score > 0.0 {
        score / max_score
    } else {
        0.0
    };

    (normalized, shared_sports)
}

/// Check if a profile is within the candidate query constraints
#[inline]
pub fn matches_query_constraints(
    profile: &UserProfile,
    query: &CandidateQuery,
) -> bool {
    // Check bounding box (Stage 1 - geospatial pre-filter)
    if !super::distance::is_within_bounding_box(
        profile.latitude,
        profile.longitude,
        &query.bounding_box,
    ) {
        return false;
    }

    // Check excluded users
    if query.exclude_user_ids.contains(&profile.user_id) {
        return false;
    }

    // Check age range
    if profile.age < query.min_age || profile.age > query.max_age {
        return false;
    }

    // Check height range
    if profile.height_cm < query.min_height_cm
        || profile.height_cm > query.max_height_cm {
        return false;
    }

    // Check gender preferences
    if !query.preferred_genders.is_empty()
        && !query.preferred_genders.contains(&profile.gender) {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_profile(age: u8, gender: &str, height_cm: u16) -> UserProfile {
        UserProfile {
            user_id: "test_user".to_string(),
            name: "Test User".to_string(),
            age,
            height_cm,
            hair_color: "brown".to_string(),
            gender: gender.to_string(),
            latitude: 40.7128,
            longitude: -74.0060,
            is_verified: Some(true),
            is_active: true,
            is_timeout: Some(false),
            image_file_ids: vec![],
            description: None,
            sports_preferences: vec!["tennis".to_string(), "swimming".to_string()],
            created_at: Some(Utc::now()),
        }
    }

    fn create_test_preferences() -> UserPreferences {
        UserPreferences {
            user_id: "pref_user".to_string(),
            preferred_genders: vec!["female".to_string()],
            min_age: 21,
            max_age: 35,
            min_height_cm: 160,
            max_height_cm: 180,
            preferred_hair_colors: vec![],
            preferred_sports: vec!["tennis".to_string()],
            max_distance_km: 50,
            latitude: 40.7128,
            longitude: -74.0060,
        }
    }

    #[test]
    fn test_demographics_match() {
        let profile = create_test_profile(25, "female", 170);
        let preferences = create_test_preferences();

        assert!(matches_demographics(&profile, &preferences));
    }

    #[test]
    fn test_demographics_fail_age() {
        let profile = create_test_profile(40, "female", 170);
        let preferences = create_test_preferences();

        assert!(!matches_demographics(&profile, &preferences));
    }

    #[test]
    fn test_demographics_fail_gender() {
        let profile = create_test_profile(25, "male", 170);
        let preferences = create_test_preferences();

        assert!(!matches_demographics(&profile, &preferences));
    }

    #[test]
    fn test_inactive_user_filtered() {
        let mut profile = create_test_profile(25, "female", 170);
        profile.is_active = false;
        let preferences = create_test_preferences();

        assert!(!matches_demographics(&profile, &preferences));
    }

    #[test]
    fn test_preference_score() {
        let profile = create_test_profile(25, "female", 170);
        let preferences = create_test_preferences();

        let (score, shared) = calculate_preference_score(&profile, &preferences);

        assert!(score > 0.0);
        assert_eq!(shared, vec!["tennis"]);
    }
}
