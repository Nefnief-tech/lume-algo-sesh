use crate::models::{UserProfile, UserPreferences, ScoredMatch, ScoringWeights, CandidateQuery};
use crate::core::{
    distance::{calculate_bounding_box, haversine_distance},
    filters::{matches_demographics, matches_query_constraints},
    scoring::calculate_match_score,
};

/// Result of the matching process
#[derive(Debug)]
pub struct MatchResult {
    pub matches: Vec<ScoredMatch>,
    pub total_candidates: usize,
}

/// Main matching orchestrator - implements the multi-stage filtering pipeline
///
/// # Pipeline Stages
/// 1. Geospatial bounding box pre-filter
/// 2. Demographic filtering
/// 3. Preference matching
/// 4. Scoring and ranking
#[derive(Debug, Clone)]
pub struct Matcher {
    weights: ScoringWeights,
}

impl Matcher {
    pub fn new(weights: ScoringWeights) -> Self {
        Self { weights }
    }

    pub fn with_default_weights() -> Self {
        Self {
            weights: ScoringWeights::default(),
        }
    }

    /// Find matches for a user based on their preferences
    ///
    /// This implements the complete multi-stage filtering pipeline.
    ///
    /// # Arguments
    /// * `preferences` - The user's matching preferences
    /// * `candidates` - All potential candidates from the database
    /// * `limit` - Maximum number of matches to return
    ///
    /// # Returns
    /// MatchResult containing scored and ranked matches
    pub fn find_matches(
        &self,
        preferences: &UserPreferences,
        candidates: Vec<UserProfile>,
        limit: usize,
    ) -> MatchResult {
        let total_candidates = candidates.len();

        // Build candidate query
        let bounding_box = calculate_bounding_box(
            preferences.latitude,
            preferences.longitude,
            preferences.max_distance_km as f64,
        );

        let query = CandidateQuery {
            bounding_box,
            preferred_genders: preferences.preferred_genders.clone(),
            min_age: preferences.min_age,
            max_age: preferences.max_age,
            min_height_cm: preferences.min_height_cm,
            max_height_cm: preferences.max_height_cm,
            exclude_user_ids: vec![preferences.user_id.clone()], // Exclude self
            limit,
        };

        // Multi-stage filtering pipeline
        let mut scored_matches: Vec<ScoredMatch> = candidates
            .into_iter()
            // Stage 1: Geospatial + basic query pre-filter
            .filter(|profile| matches_query_constraints(profile, &query))
            // Stage 2: Demographic filtering
            .filter(|profile| matches_demographics(profile, preferences))
            // Stage 3 & 4: Calculate scores
            .filter_map(|profile| {
                let (score, shared_sports) = calculate_match_score(
                    &profile,
                    preferences,
                    &self.weights,
                );

                // Only include profiles with a minimum score
                if score >= 5.0 {
                    let distance_km = haversine_distance(
                        preferences.latitude,
                        preferences.longitude,
                        profile.latitude,
                        profile.longitude,
                    );

                    let is_verified = profile.verified();

                    Some(ScoredMatch {
                        user_id: profile.user_id,
                        name: profile.name,
                        age: profile.age,
                        height_cm: profile.height_cm,
                        hair_color: profile.hair_color,
                        gender: profile.gender,
                        distance_km,
                        match_score: score,
                        shared_sports,
                        is_verified,
                        image_file_ids: profile.image_file_ids,
                        description: profile.description,
                    })
                } else {
                    None
                }
            })
            .collect();

        // Sort by score (descending) and then by distance (ascending)
        scored_matches.sort_by(|a, b| {
            b.match_score
                .partial_cmp(&a.match_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    a.distance_km
                        .partial_cmp(&b.distance_km)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });

        // Limit results
        scored_matches.truncate(limit);

        MatchResult {
            matches: scored_matches,
            total_candidates,
        }
    }
}

impl Default for Matcher {
    fn default() -> Self {
        Self::with_default_weights()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_candidate(
        id: &str,
        age: u8,
        gender: &str,
        lat: f64,
        lon: f64,
        is_verified: bool,
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
            is_verified: Some(is_verified),
            is_active: true,
            is_timeout: Some(false),
            image_file_ids: vec![],
            description: None,
            sports_preferences: vec!["tennis".to_string()],
            created_at: Some(Utc::now()),
        }
    }

    fn create_preferences() -> UserPreferences {
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
            latitude: 40.7128,  // New York
            longitude: -74.0060,
        }
    }

    #[test]
    fn test_find_matches_basic() {
        let matcher = Matcher::with_default_weights();
        let preferences = create_preferences();

        let candidates = vec![
            create_candidate("1", 25, "female", 40.72, -74.01, true),  // Close match
            create_candidate("2", 40, "female", 40.72, -74.01, true),  // Too old
            create_candidate("3", 25, "male", 40.72, -74.01, true),    // Wrong gender
        ];

        let result = matcher.find_matches(&preferences, candidates, 10);

        // Should only match the first candidate
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].user_id, "1");
    }

    #[test]
    fn test_matches_sorted_by_score() {
        let matcher = Matcher::with_default_weights();
        let preferences = create_preferences();

        let candidates = vec![
            create_candidate("1", 25, "female", 40.72, -74.01, true),   // Close
            create_candidate("2", 28, "female", 40.72, -74.01, false),  // Further, unverified
        ];

        let result = matcher.find_matches(&preferences, candidates, 10);

        assert_eq!(result.matches.len(), 2);
        // First match should have higher score (verified + closer age to mid)
        assert!(result.matches[0].match_score >= result.matches[1].match_score);
    }

    #[test]
    fn test_respects_limit() {
        let matcher = Matcher::with_default_weights();
        let preferences = create_preferences();

        let candidates: Vec<UserProfile> = (0..20)
            .map(|i| {
                create_candidate(
                    &i.to_string(),
                    25 + (i % 10) as u8,
                    "female",
                    40.72 + (i as f64 * 0.001),
                    -74.01,
                    true,
                )
            })
            .collect();

        let result = matcher.find_matches(&preferences, candidates, 5);

        assert!(result.matches.len() <= 5);
    }

    #[test]
    fn test_distance_filtering() {
        let matcher = Matcher::with_default_weights();
        let preferences = create_preferences();

        let candidates = vec![
            create_candidate("1", 25, "female", 40.72, -74.01, true),   // ~1km away
            create_candidate("2", 25, "female", 41.5, -74.0, true),     // ~90km away
            create_candidate("3", 25, "female", 45.0, -74.0, true),     // >400km away
        ];

        let result = matcher.find_matches(&preferences, candidates, 10);

        // First two should be within 50km, third should be filtered out
        assert!(result.matches.len() <= 2);
    }
}
