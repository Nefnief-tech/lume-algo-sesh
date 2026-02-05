# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Lume Algo is a high-performance matching service for the Lume dating app, written in Rust. It implements a 4-stage filtering pipeline to find compatible matches based on geolocation, demographics, preferences, and a weighted scoring algorithm.

## Commands

### Building
```bash
cargo build --release          # Optimized release build
cargo check                     # Quick compile check
```

### Testing
```bash
cargo test                      # Run all tests
cargo test --lib               # Library tests only
cargo test --test unit_tests   # Unit tests file
cargo test --test integration_tests  # Integration tests
cargo test matching::          # Tests in matching module
```

### Benchmarks
```bash
cargo bench                     # Run Criterion benchmarks
```

### Running
```bash
# Local (requires Redis)
cargo run

# Docker Compose (includes Redis)
docker-compose up -d

# Build Docker image
docker build -t lume-algo:latest .
```

## Architecture

### 4-Stage Matching Pipeline

The core algorithm in `src/core/matcher.rs` processes candidates through:

1. **Geospatial bounding box** (`distance.rs`) - Fast pre-filter eliminating 90%+ of candidates using rectangular bounds around user's location
2. **Demographic filters** (`filters.rs`) - Age range, height range, gender preference, active status
3. **Preference matching** (`filters.rs`) - Hair color, sports overlap
4. **Scoring** (`scoring.rs`) - Weighted formula (distance×0.35 + age×0.20 + sports×0.25 + verified×0.10 + height×0.10) × 100

### Module Structure

- `src/core/` - Algorithm implementation (matcher, scoring, distance, filters)
- `src/models/` - Domain types (UserProfile, UserPreferences, ScoredMatch, ScoringWeights)
- `src/services/` - External integrations (Appwrite client, Redis cache manager)
- `src/routes/` - HTTP handlers (`/api/v1/matches/find`, `/api/v1/matches/event`, `/api/v1/health`)
- `src/config.rs` - TOML + env var configuration loader

### Data Flow

Request → AppState (Arc-shared) → fetch preferences from Appwrite → query candidates → Matcher::find_matches() → cache results → return sorted matches

### Caching Strategy

Two-tier cache: L1 in-memory (Moka, 1000 entries) + L2 Redis (TTL 300s). Cache keys use pattern `matches:{userId}`.

### Configuration

Loaded from `config/default.toml` with `LUME_` prefixed env var overrides. Key settings:
- `LUME_APPWRITE__ENDPOINT`, `LUME_APPWRITE__API_KEY`, `LUME_APPWRITE__PROJECT_ID`
- `LUME_CACHE__REDIS_URL`
- `LUME_SCORING__WEIGHTS__*` for tuning scoring formula

### Appwrite Collections Required

- `user_profiles` - (isActive, isTimeout, gender, age, latitude, longitude, heightCm, hairColor, sportsPreferences, isVerified, imageFileIds, description)
- `user_preferences` - (userId, preferredGenders, minAge, maxAge, minHeightCm, maxHeightCm, preferredHairColors, preferredSports, maxDistanceKm, latitude, longitude)
- `match_events` - Track user interactions
- `user_matches` - Mutual match cache

### Library Exports

The `lib.rs` exports: `Matcher`, `ScoringWeights`, `haversine_distance`, `calculate_bounding_box`, `UserProfile`, `UserPreferences`, `ScoredMatch`, `FindMatchesRequest`, `FindMatchesResponse` for use as a library or testing.
