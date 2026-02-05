# Lume Algo - High-Performance Matching Service

A Rust-based matching service for the Lume dating app, implementing a multi-stage filtering algorithm with Redis caching for sub-100ms response times.

## Architecture

```
Request → Geospatial Pre-filter → Demographic Filter → Preference Match → Scoring → Ranked Results
```

### Multi-Stage Filtering Pipeline

1. **Stage 1: Geospatial Bounding Box** - Fastest pre-filter (eliminates 90%+)
2. **Stage 2: Demographic Filters** - Age, height, gender, active status
3. **Stage 3: Preference Matching** - Hair color, sports overlap
4. **Stage 4: Scoring** - Weighted scoring formula (0-100)

## Quick Start

### Using Docker Compose

```bash
# Copy environment file
cp .env.example .env

# Edit .env with your Appwrite credentials
nano .env

# Start services
docker-compose up -d

# Check health
curl http://localhost:8080/api/v1/health
```

### Building from Source

```bash
# Install Rust (https://rustup.rs/)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
cargo build --release

# Run
./target/release/lume-algo
```

## API Endpoints

### Find Matches

```bash
POST /api/v1/matches/find
Content-Type: application/json

{
  "userId": "user_123",
  "limit": 20,
  "excludeUserIds": ["user_456", "user_789"]
}
```

**Response:**

```json
{
  "matches": [
    {
      "userId": "user_abc",
      "name": "Jane Doe",
      "age": 25,
      "heightCm": 170,
      "hairColor": "brown",
      "gender": "female",
      "distanceKm": 5.2,
      "matchScore": 85.5,
      "sharedSports": ["tennis", "swimming"],
      "isVerified": true,
      "imageFileIds": ["file_123"],
      "description": "Love outdoor activities!"
    }
  ],
  "nextCursor": null,
  "totalResults": 150
}
```

### Record Match Event

```bash
POST /api/v1/matches/event
Content-Type: application/json

{
  "userId": "user_123",
  "targetUserId": "user_abc",
  "eventType": "liked"
}
```

### Health Check

```bash
GET /api/v1/health
```

## Configuration

Configuration is loaded from `config/default.toml` and can be overridden with environment variables prefixed with `LUME_`.

```bash
# Server
LUME_SERVER__HOST=0.0.0.0
LUME_SERVER__PORT=8080

# Appwrite
LUME_APPWRITE__ENDPOINT=https://appwrite.lume.com/v1
LUME_APPWRITE__API_KEY=your_api_key
LUME_APPWRITE__PROJECT_ID=your_project_id

# Cache
LUME_CACHE__REDIS_URL=redis://localhost:6379
LUME_CACHE__TTL_SECS=300

# Scoring Weights
LUME_SCORING__WEIGHTS__DISTANCE=0.35
LUME_SCORING__WEIGHTS__AGE=0.20
LUME_SCORING__WEIGHTS__SPORTS=0.25
LUME_SCORING__WEIGHTS__VERIFIED=0.10
LUME_SCORING__WEIGHTS__HEIGHT=0.10
```

## Development

### Running Tests

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration_tests

# Benchmarks
cargo bench
```

### Project Structure

```
lume-algo/
├── src/
│   ├── main.rs           # Entry point
│   ├── lib.rs            # Library exports
│   ├── config.rs         # Configuration loading
│   ├── core/             # Algorithm implementation
│   │   ├── matcher.rs    # Main orchestration
│   │   ├── scoring.rs    # Score calculation
│   │   ├── distance.rs   # Geospatial calculations
│   │   └── filters.rs    # Individual filters
│   ├── models/           # Data structures
│   │   ├── domain.rs     # Domain types
│   │   ├── requests.rs   # API requests
│   │   └── responses.rs  # API responses
│   ├── services/         # External services
│   │   ├── appwrite.rs   # Appwrite client
│   │   └── cache.rs      # Redis + L1 cache
│   └── routes/           # HTTP endpoints
├── tests/                # Integration tests
├── config/               # Configuration files
├── k8s/                  # Kubernetes manifests
└── Dockerfile
```

## Performance

| Metric | Target |
|--------|--------|
| p95 Latency | <50ms |
| p99 Latency | <100ms |
| Throughput | 50,000+ matches/second |
| Memory | <100MB RSS |

## Deployment

### Kubernetes

```bash
# Create namespace
kubectl create namespace lume

# Apply manifests
kubectl apply -f k8s/ -n lume

# Check status
kubectl get pods -n lume
```

### Docker

```bash
# Build image
docker build -t lume-algo:latest .

# Run container
docker run -p 8080:8080 \
  -e APPWRITE_ENDPOINT=https://appwrite.lume.com/v1 \
  -e APPWRITE_API_KEY=your_key \
  -e REDIS_URL=redis://redis:6379 \
  lume-algo:latest
```

## Appwrite Integration

The service integrates with your existing Appwrite instance. Ensure the following collections exist:

- `user_profiles` - User profile data
- `user_preferences` - User matching preferences
- `match_events` - Match interaction tracking
- `user_matches` - Mutual match cache

### Recommended Indexes

```javascript
// User Profiles
db.user_profiles.createIndex({ isActive: 1, isTimeout: 1, gender: 1, age: 1 })
db.user_profiles.createIndex({ latitude: 1, longitude: 1 })
db.user_profiles.createIndex({ latitude: 1, longitude: 1, age: 1 })

// User Preferences
db.user_preferences.createIndex({ userId: 1 }, { unique: true })
```

## License

Copyright (c) 2025 Lume Team. All rights reserved.
