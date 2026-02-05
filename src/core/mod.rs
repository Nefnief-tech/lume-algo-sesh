// Core algorithm exports
pub mod distance;
pub mod filters;
pub mod matcher;
pub mod scoring;

pub use distance::{haversine_distance, calculate_bounding_box, is_within_bounding_box};
pub use filters::{matches_demographics, calculate_preference_score, matches_query_constraints};
pub use matcher::{Matcher, MatchResult};
pub use scoring::calculate_match_score;
