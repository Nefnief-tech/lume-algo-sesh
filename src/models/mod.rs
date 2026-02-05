// Model exports
pub mod domain;
pub mod requests;
pub mod responses;

pub use domain::{UserProfile, UserPreferences, MatchEvent, MatchEventType, UserMatch, ScoredMatch, BoundingBox, CandidateQuery, ScoringWeights};
pub use requests::{FindMatchesRequest, RecordEventRequest};
pub use responses::{FindMatchesResponse, HealthResponse, ErrorResponse, RecordEventResponse};
