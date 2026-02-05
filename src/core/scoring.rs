use crate::models::{UserProfile, UserPreferences, ScoringWeights};
use crate::core::{distance::haversine_distance, filters::calculate_preference_score};

/// Calculate a match score (0-100) for a profile based on user preferences
///
/// Scoring formula:
/// score = (
///     distance_score * 0.35 +      # Closer = higher score
///     age_score * 0.20 +           # Within preferred range = higher
///     sports_score * 0.25 +        # More shared sports = higher
///     verified_bonus * 0.10 +      # isVerified = true
///     height_score * 0.10          # Within preferred height range
/// )
pub fn calculate_match_score(
    profile: &UserProfile,
    preferences: &UserPreferences,
    weights: &ScoringWeights,
) -> (f64, Vec<String>) {
    // Stage 4a: Distance score (closer is better)
    let distance_km = haversine_distance(
        preferences.latitude,
        preferences.longitude,
        profile.latitude,
        profile.longitude,
    );

    let distance_score = calculate_distance_score(distance_km, preferences.max_distance_km);

    // Stage 4b: Age score (closer to middle of preferred range is better)
    let age_score = calculate_age_score(profile.age, preferences.min_age, preferences.max_age);

    // Stage 4c: Sports/preference score
    let (pref_score, shared_sports) = calculate_preference_score(profile, preferences);

    // Stage 4d: Verified bonus
    let verified_score = if profile.verified() { 1.0 } else { 0.0 };

    // Stage 4e: Height score (within preferred range)
    let height_score = calculate_height_score(
        profile.height_cm,
        preferences.min_height_cm,
        preferences.max_height_cm,
    );

    // Weighted combination
    let total_score = (distance_score * weights.distance
        + age_score * weights.age
        + pref_score * weights.sports
        + verified_score * weights.verified
        + height_score * weights.height)
        * 100.0;

    (total_score.min(100.0).max(0.0), shared_sports)
}

/// Calculate distance score (0-1)
/// Closer distance = higher score, exponentially decaying
#[inline]
fn calculate_distance_score(distance_km: f64, max_distance_km: u16) -> f64 {
    let max = max_distance_km as f64;
    if distance_km >= max {
        return 0.0;
    }

    // Exponential decay: score = e^(-distance / max_distance)
    // This gives a smooth curve where nearby users score much higher
    (-distance_km / (max * 0.5)).exp()
}

/// Calculate age score (0-1)
/// Users closer to the middle of the preferred range score higher
#[inline]
fn calculate_age_score(age: u8, min_age: u8, max_age: u8) -> f64 {
    let mid = (min_age + max_age) as f64 / 2.0;
    let range = (max_age - min_age) as f64;
    let age_f = age as f64;

    if range <= 0.0 {
        return 1.0;
    }

    // Score decreases as age moves away from the middle of the range
    let deviation = (age_f - mid).abs();
    let normalized_deviation = deviation / (range / 2.0);

    1.0 - normalized_deviation.min(1.0)
}

/// Calculate height score (0-1)
/// Users closer to the middle of the preferred height range score higher
#[inline]
fn calculate_height_score(height_cm: u16, min_height_cm: u16, max_height_cm: u16) -> f64 {
    let mid = (min_height_cm + max_height_cm) as f64 / 2.0;
    let range = (max_height_cm - min_height_cm) as f64;
    let height_f = height_cm as f64;

    if range <= 0.0 {
        return 1.0;
    }

    // Score decreases as height moves away from the middle
    let deviation = (height_f - mid).abs();
    let normalized_deviation = deviation / (range / 2.0);

    1.0 - normalized_deviation.min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_profile(age: u8, height_cm: u16, is_verified: bool) -> UserProfile {
        UserProfile {
            user_id: "test_user".to_string(),
            name: "Test User".to_string(),
            age,
            height_cm,
            hair_color: "brown".to_string(),
            gender: "female".to_string(),
            latitude: 40.7128,
            longitude: -74.0060,
            is_verified,
            is_active: true,
            is_timeout: false,
            image_file_ids: vec![],
            description: None,
            sports_preferences: vec!["tennis".to_string()],
            created_at: Utc::now(),
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
    fn test_calculate_match_score() {
        let profile = create_test_profile(25, 170, true);
        let preferences = create_test_preferences();
        let weights = ScoringWeights::default();

        let (score, shared) = calculate_match_score(&profile, &preferences, &weights);

        assert!(score >= 0.0 && score <= 100.0);
        assert_eq!(shared, vec!["tennis"]);
    }

    #[test]
    fn test_distance_score() {
        // Very close = high score
        let close = calculate_distance_score(1.0, 50);
        assert!(close > 0.9);

        // At max distance = zero score
        let at_max = calculate_distance_score(50.0, 50);
        assert_eq!(at_max, 0.0);

        // Half distance = moderate score
        let half = calculate_distance_score(25.0, 50);
        assert!(half > 0.3 && half < 0.8);
    }

    #[test]
    fn test_age_score() {
        // Middle of range = max score
        let mid = calculate_age_score(28, 21, 35);
        assert!(mid > 0.9);

        // At edge of range = lower score
        let edge = calculate_age_score(21, 21, 35);
        assert!(edge < 0.5);
    }

    #[test]
    fn test_height_score() {
        // Middle of range = max score
        let mid = calculate_height_score(170, 160, 180);
        assert!(mid > 0.9);

        // At edge = lower score
        let edge = calculate_height_score(160, 160, 180);
        assert!(edge < 0.5);
    }

    #[test]
    fn test_verified_bonus() {
        let verified_profile = create_test_profile(25, 170, true);
        let unverified_profile = create_test_profile(25, 170, false);
        let preferences = create_test_preferences();
        let weights = ScoringWeights::default();

        let (verified_score, _) = calculate_match_score(&verified_profile, &preferences, &weights);
        let (unverified_score, _) = calculate_match_score(&unverified_profile, &preferences, &weights);

        assert!(verified_score > unverified_score);
    }
}
