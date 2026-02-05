//! Lume Algo - High-performance matching service for Lume dating app
//!
//! This library provides the core matching algorithm used by the Lume dating app.
//! It implements a multi-stage filtering pipeline for efficient user matching.

pub mod config;
pub mod core;
pub mod models;
pub mod routes;
pub mod services;

// Re-export commonly used types
pub use core::{Matcher, distance::{haversine_distance, calculate_bounding_box}};
pub use models::{UserProfile, UserPreferences, ScoredMatch, ScoringWeights, FindMatchesRequest, FindMatchesResponse};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_exports() {
        // Verify that the library exports work correctly
        let bbox = calculate_bounding_box(40.7128, -74.0060, 10.0);
        assert!(bbox.min_lat < 40.7128);
    }
}
