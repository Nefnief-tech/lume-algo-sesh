# Lume Matching Algorithm - Flutter Migration Guide

Guide to migrate your existing Flutter dating app to use the new Lume matching algorithm.

---

## Quick Start: What You Need

1. Copy the algorithm files into your project
2. Replace your current match fetching with the new service
3. Update your UI models (if needed)

---

## 1. Copy Algorithm Files

Create these files in your Flutter app:

### `lib/matching/distance_calculator.dart`

```dart
import 'dart:math';

class DistanceCalculator {
  static const double earthRadiusKm = 6371.0;

  /// Calculate Haversine distance between two coordinates in kilometers
  static double haversineDistance(
    double lat1,
    double lon1,
    double lat2,
    double lon2,
  ) {
    final lat1Rad = lat1 * pi / 180;
    final lat2Rad = lat2 * pi / 180;
    final deltaLat = (lat2 - lat1) * pi / 180;
    final deltaLon = (lon2 - lon1) * pi / 180;

    final a = (sin(deltaLat / 2) * sin(deltaLat / 2)) +
        (cos(lat1Rad) *
            cos(lat2Rad) *
            sin(deltaLon / 2) *
            sin(deltaLon / 2));

    final c = 2 * atan2(sqrt(a), sqrt(1 - a));
    return earthRadiusKm * c;
  }

  /// Calculate bounding box for fast pre-filtering
  static BoundingBox calculateBoundingBox({
    required double centerLat,
    required double centerLon,
    required double radiusKm,
  }) {
    final latDelta = radiusKm / 111.0;
    final lonDelta = radiusKm / (111.0 * cos(centerLat * pi / 180).abs());

    return BoundingBox(
      minLat: centerLat - latDelta,
      maxLat: centerLat + latDelta,
      minLon: centerLon - lonDelta,
      maxLon: centerLon + lonDelta,
    );
  }
}

class BoundingBox {
  final double minLat;
  final double maxLat;
  final double minLon;
  final double maxLon;

  BoundingBox({
    required this.minLat,
    required this.maxLat,
    required this.minLon,
    required this.maxLon,
  });

  bool contains(double lat, double lon) {
    return lat >= minLat &&
        lat <= maxLat &&
        lon >= minLon &&
        lon <= maxLon;
  }
}
```

### `lib/matching/match_scorer.dart`

```dart
import 'dart:math';
import 'distance_calculator.dart';

class MatchScorer {
  final ScoringWeights weights;

  const MatchScorer({this.weights = const ScoringWeights()});

  /// Calculate match score (0-100) for a profile
  ScoredMatchResult calculate({
    required Map<String, dynamic> profile,
    required Map<String, dynamic> preferences,
  }) {
    // Get coordinates
    final profileLat = (profile['latitude'] as num).toDouble();
    final profileLon = (profile['longitude'] as num).toDouble();
    final prefLat = (preferences['latitude'] as num).toDouble();
    final prefLon = (preferences['longitude'] as num).toDouble();
    final maxDist = (preferences['maxDistanceKm'] as num).toDouble();

    // Calculate distance
    final distanceKm = DistanceCalculator.haversineDistance(
      prefLat,
      prefLon,
      profileLat,
      profileLon,
    );

    // Component scores
    final distanceScore = _distanceScore(distanceKm, maxDist);
    final ageScore = _ageScore(
      (profile['age'] as num).toDouble(),
      (preferences['minAge'] as num).toDouble(),
      (preferences['maxAge'] as num).toDouble(),
    );
    final (sportsScore, shared) = _sportsScore(
      profile['sportsPreferences'] as List<dynamic>? ?? [],
      preferences['preferredSports'] as List<dynamic>? ?? [],
    );
    final verifiedScore = profile['isVerified'] == true ? 1.0 : 0.0;
    final heightScore = _heightScore(
      (profile['heightCm'] as num).toDouble(),
      (preferences['minHeightCm'] as num).toDouble(),
      (preferences['maxHeightCm'] as num).toDouble(),
    );

    // Weighted combination
    final totalScore = (
      distanceScore * weights.distance +
      ageScore * weights.age +
      sportsScore * weights.sports +
      verifiedScore * weights.verified +
      heightScore * weights.height
    ) * 100;

    return ScoredMatchResult(
      score: totalScore.clamp(0.0, 100.0),
      distanceKm: distanceKm,
      sharedSports: shared,
    );
  }

  double _distanceScore(double distance, double maxDistance) {
    if (distance >= maxDistance) return 0.0;
    return exp(-distance / (maxDistance * 0.5));
  }

  double _ageScore(double age, double minAge, double maxAge) {
    final mid = (minAge + maxAge) / 2;
    final range = maxAge - minAge;
    if (range <= 0) return 1.0;
    final deviation = (age - mid).abs();
    return (1.0 - (deviation / (range / 2)).clamp(0.0, 1.0));
  }

  double _heightScore(double height, double minH, double maxH) {
    final mid = (minH + maxH) / 2;
    final range = maxH - minH;
    if (range <= 0) return 1.0;
    final deviation = (height - mid).abs();
    return (1.0 - (deviation / (range / 2)).clamp(0.0, 1.0));
  }

  (double score, List<String> shared) _sportsScore(
    List<dynamic> profileSports,
    List<dynamic> preferredSports,
  ) {
    final shared = <String>[];
    for (final sport in profileSports) {
      if (preferredSports.contains(sport)) {
        shared.add(sport as String);
      }
    }
    final count = shared.length.toDouble();
    final score = (count.clamp(0.0, 5.0) / 5.0) * 2.0;
    return (score, shared);
  }
}

class ScoringWeights {
  final double distance;
  final double age;
  final double sports;
  final double verified;
  final double height;

  const ScoringWeights({
    this.distance = 0.35,
    this.age = 0.20,
    this.sports = 0.25,
    this.verified = 0.10,
    this.height = 0.10,
  });
}

class ScoredMatchResult {
  final double score;
  final double distanceKm;
  final List<String> sharedSports;

  ScoredMatchResult({
    required this.score,
    required this.distanceKm,
    required this.sharedSports,
  });
}
```

### `lib/matching/matching_pipeline.dart`

```dart
import 'distance_calculator.dart';
import 'match_scorer.dart';

class MatchingPipeline {
  final MatchScorer scorer;

  const MatchingPipeline({this.scorer = const MatchScorer()});

  /// Find matches using the 4-stage filtering pipeline
  List<ScoredMatch> findMatches({
    required Map<String, dynamic> preferences,
    required List<Map<String, dynamic>> candidates,
    required int limit,
    List<String> excludeUserIds = const [],
  }) {
    final results = <ScoredMatch>[];
    final userId = preferences['userId'] as String;

    // Stage 1: Geospatial pre-filter (bounding box)
    final bbox = DistanceCalculator.calculateBoundingBox(
      centerLat: (preferences['latitude'] as num).toDouble(),
      centerLon: (preferences['longitude'] as num).toDouble(),
      radiusKm: (preferences['maxDistanceKm'] as num).toDouble(),
    );

    for (final candidate in candidates) {
      final candidateId = candidate['userId'] as String;

      // Exclude self and specified users
      if (candidateId == userId || excludeUserIds.contains(candidateId)) {
        continue;
      }

      // Stage 1: Check bounding box
      final lat = (candidate['latitude'] as num).toDouble();
      final lon = (candidate['longitude'] as num).toDouble();
      if (!bbox.contains(lat, lon)) continue;

      // Stage 2: Demographic filters
      if (!_matchesDemographics(candidate, preferences)) continue;

      // Stage 3 & 4: Calculate score
      final scoreResult = scorer.calculate(
        profile: candidate,
        preferences: preferences,
      );

      if (scoreResult.score >= 10.0) {
        results.add(ScoredMatch.fromCandidate(
          candidate: candidate,
          scoreResult: scoreResult,
        ));
      }
    }

    // Sort by score desc, then distance asc
    results.sort((a, b) {
      final scoreCompare = b.matchScore.compareTo(a.matchScore);
      if (scoreCompare != 0) return scoreCompare;
      return a.distanceKm.compareTo(b.distanceKm);
    });

    return results.take(limit).toList();
  }

  bool _matchesDemographics(
    Map<String, dynamic> profile,
    Map<String, dynamic> prefs,
  ) {
    // Active check
    if (profile['isActive'] != true) return false;
    if (profile['isTimeout'] == true) return false;

    // Gender check
    final preferredGenders = prefs['preferredGenders'] as List<dynamic>? ?? [];
    if (preferredGenders.isNotEmpty &&
        !preferredGenders.contains(profile['gender'])) {
      return false;
    }

    // Age check
    final age = profile['age'] as int;
    final minAge = prefs['minAge'] as int;
    final maxAge = prefs['maxAge'] as int;
    if (age < minAge || age > maxAge) return false;

    // Height check
    final height = profile['heightCm'] as int;
    final minHeight = prefs['minHeightCm'] as int;
    final maxHeight = prefs['maxHeightCm'] as int;
    if (height < minHeight || height > maxHeight) return false;

    return true;
  }
}

class ScoredMatch {
  final String userId;
  final String name;
  final int age;
  final int heightCm;
  final String hairColor;
  final String gender;
  final double distanceKm;
  final double matchScore;
  final List<String> sharedSports;
  final bool isVerified;
  final List<String> imageFileIds;
  final String? description;

  ScoredMatch({
    required this.userId,
    required this.name,
    required this.age,
    required this.heightCm,
    required this.hairColor,
    required this.gender,
    required this.distanceKm,
    required this.matchScore,
    required this.sharedSports,
    required this.isVerified,
    required this.imageFileIds,
    this.description,
  });

  factory ScoredMatch.fromCandidate({
    required Map<String, dynamic> candidate,
    required ScoredMatchResult scoreResult,
  }) {
    return ScoredMatch(
      userId: candidate['userId'] as String,
      name: candidate['name'] as String,
      age: candidate['age'] as int,
      heightCm: candidate['heightCm'] as int,
      hairColor: candidate['hairColor'] as String,
      gender: candidate['gender'] as String,
      distanceKm: scoreResult.distanceKm,
      matchScore: scoreResult.score,
      sharedSports: scoreResult.sharedSports,
      isVerified: candidate['isVerified'] as bool? ?? false,
      imageFileIds: (candidate['imageFileIds'] as List<dynamic>?)
              ?.cast<String>() ??
          [],
      description: candidate['description'] as String?,
    );
  }
}
```

---

## 2. Update Your Match Service

Replace your existing match fetching with:

### Option A: Use the Lume API Service (Recommended)

```dart
// lib/services/lume_match_service.dart
import 'dart:convert';
import 'package:http/http.dart' as http;

class LumeMatchService {
  final String baseUrl;
  final String apiKey;

  LumeMatchService({
    required this.baseUrl,
    required this.apiKey,
  });

  Future<List<ScoredMatch>> fetchMatches({
    required String userId,
    int limit = 20,
    List<String> excludeUserIds = const [],
  }) async {
    final response = await http.post(
      Uri.parse('$baseUrl/api/v1/matches/find'),
      headers: {
        'Content-Type': 'application/json',
        'X-Appwrite-Key': apiKey,
      },
      body: jsonEncode({
        'userId': userId,
        'limit': limit,
        'excludeUserIds': excludeUserIds,
      }),
    );

    if (response.statusCode == 200) {
      final json = jsonDecode(response.body) as Map<String, dynamic>;
      final matches = (json['matches'] as List<dynamic>?)
              ?.map((m) => _parseMatch(m as Map<String, dynamic>))
              .toList() ??
          [];
      return matches;
    }
    throw Exception('Failed to fetch matches: ${response.statusCode}');
  }

  Future<void> recordEvent({
    required String userId,
    required String targetUserId,
    required String eventType, // 'viewed', 'liked', 'passed', 'matched'
  }) async {
    await http.post(
      Uri.parse('$baseUrl/api/v1/matches/event'),
      headers: {
        'Content-Type': 'application/json',
        'X-Appwrite-Key': apiKey,
      },
      body: jsonEncode({
        'userId': userId,
        'targetUserId': targetUserId,
        'eventType': eventType,
      }),
    );
  }

  ScoredMatch _parseMatch(Map<String, dynamic> json) {
    return ScoredMatch(
      userId: json['userId'] as String,
      name: json['name'] as String,
      age: json['age'] as int,
      heightCm: json['heightCm'] as int,
      hairColor: json['hairColor'] as String,
      gender: json['gender'] as String,
      distanceKm: (json['distanceKm'] as num).toDouble(),
      matchScore: (json['matchScore'] as num).toDouble(),
      sharedSports: (json['sharedSports'] as List<dynamic>?)
              ?.cast<String>() ??
          [],
      isVerified: json['isVerified'] as bool? ?? false,
      imageFileIds: (json['imageFileIds'] as List<dynamic>?)
              ?.cast<String>() ??
          [],
      description: json['description'] as String?,
    );
  }
}
```

### Option B: Client-Side Matching (Keep your current API)

```dart
// lib/services/client_match_service.dart
import '../matching/matching_pipeline.dart';

class ClientMatchService {
  final MatchingPipeline pipeline = const MatchingPipeline();

  Future<List<ScoredMatch>> fetchMatches({
    required Map<String, dynamic> preferences,
    required List<Map<String, dynamic>> candidates,
    int limit = 20,
    List<String> excludeUserIds = const [],
  }) async {
    // Run matching in isolate for performance
    return pipeline.findMatches(
      preferences: preferences,
      candidates: candidates,
      limit: limit,
      excludeUserIds: excludeUserIds,
    );
  }
}
```

---

## 3. Update Your Existing Models (If Needed)

If your current user model doesn't have all required fields, add them:

```dart
// Add these fields to your existing user model if missing
class UserModel {
  // ... your existing fields ...

  // Required for matching algorithm:
  final double latitude;
  final double longitude;
  final int heightCm;
  final String hairColor;
  final List<String> sportsPreferences;
  final bool isVerified;
  final bool isActive;
  final bool isTimeout;

  // ... rest of your model ...
}
```

---

## 4. Update Your Provider/Bloc

Replace your current match loading logic:

### Riverpod Example

```dart
// Before (your current code)
final matchesProvider = FutureProvider.autoDispose((ref) {
  return ref.watch(apiService).getMatches();
});

// After (new matching service)
final matchesProvider = FutureProvider.autoDispose((ref) {
  final prefs = ref.watch(userPreferencesProvider);
  final service = ref.watch(lumeMatchServiceProvider);
  return service.fetchMatches(
    userId: prefs.userId,
    limit: 20,
    excludeUserIds: prefs.seenUserIds,
  );
});
```

### Bloc Example

```dart
// Before
class MatchBloc extends Bloc<MatchEvent, MatchState> {
  final ApiRepository repository;

  Future<void> _onLoadMatches(
    LoadMatches event,
    Emitter<MatchState> emit,
  ) async {
    emit(MatchLoading());
    try {
      final matches = await repository.getMatches();
      emit(MatchLoaded(matches));
    } catch (e) {
      emit(MatchError(e.toString()));
    }
  }
}

// After
class MatchBloc extends Bloc<MatchEvent, MatchState> {
  final LumeMatchService repository;

  Future<void> _onLoadMatches(
    LoadMatches event,
    Emitter<MatchState> emit,
  ) async {
    emit(MatchLoading());
    try {
      final matches = await repository.fetchMatches(
        userId: event.userId,
        limit: event.limit,
        excludeUserIds: event.excludeUserIds,
      );
      emit(MatchLoaded(matches));
    } catch (e) {
      emit(MatchError(e.toString()));
    }
  }
}
```

---

## 5. Update Match Score Display

Add match score badge to your existing match card:

```dart
// Add to your existing MatchCard widget
Positioned(
  top: 16,
  right: 16,
  child: Container(
    padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
    decoration: BoxDecoration(
      color: _getScoreColor(match.matchScore).withOpacity(0.9),
      borderRadius: BorderRadius.circular(20),
    ),
    child: Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        const Icon(Icons.favorite, color: Colors.white, size: 16),
        const SizedBox(width: 4),
        Text(
          '${match.matchScore.toStringAsFixed(0)}%',
          style: const TextStyle(
            color: Colors.white,
            fontWeight: FontWeight.bold,
          ),
        ),
      ],
    ),
  ),
);

Color _getScoreColor(double score) {
  if (score >= 80) return Colors.green;
  if (score >= 60) return Colors.lightGreen;
  if (score >= 40) return Colors.orange;
  return Colors.red;
}
```

---

## 6. Record User Events

Update your like/pass handlers:

```dart
// Like button handler
Future<void> _onLike(String userId) async {
  await lumeMatchService.recordEvent(
    userId: currentUserId,
    targetUserId: userId,
    eventType: 'liked',
  );
  // Your existing like logic...
}

// Pass button handler
Future<void> _onPass(String userId) async {
  await lumeMatchService.recordEvent(
    userId: currentUserId,
    targetUserId: userId,
    eventType: 'passed',
  );
  // Your existing pass logic...
}
```

---

## 7. Environment Configuration

Add to your `.env` or config:

```dart
// Lume API endpoint (deploy the Rust service)
const LUME_API_URL = String.fromEnvironment('LUME_API_URL',
    defaultValue: 'https://api.lume.com');

// Your Appwrite API key
const APPWRITE_API_KEY = String.fromEnvironment('APPWRITE_API_KEY');
```

---

## Migration Checklist

- [ ] Copy `distance_calculator.dart` to `lib/matching/`
- [ ] Copy `match_scorer.dart` to `lib/matching/`
- [ ] Copy `matching_pipeline.dart` to `lib/matching/`
- [ ] Create `LumeMatchService` with your API credentials
- [ ] Update user model with required fields (lat/lon, height, etc.)
- [ ] Replace match fetching in Provider/Bloc
- [ ] Add match score display to MatchCard
- [ ] Update like/pass handlers to record events
- [ ] Test with sample data

---

## Testing Your Migration

```dart
// Quick test to verify algorithm works
void testMatching() {
  final pipeline = MatchingPipeline();

  final preferences = {
    'userId': 'test_user',
    'latitude': 40.7128,
    'longitude': -74.0060,
    'minAge': 21,
    'maxAge': 35,
    'minHeightCm': 160,
    'maxHeightCm': 180,
    'preferredGenders': ['female'],
    'preferredSports': ['tennis'],
    'maxDistanceKm': 50,
  };

  final candidates = [
    {
      'userId': '1',
      'name': 'Jane',
      'age': 25,
      'heightCm': 170,
      'hairColor': 'brown',
      'gender': 'female',
      'latitude': 40.72,
      'longitude': -74.01,
      'isVerified': true,
      'isActive': true,
      'isTimeout': false,
      'imageFileIds': [],
      'sportsPreferences': ['tennis'],
    },
  ];

  final matches = pipeline.findMatches(
    preferences: preferences,
    candidates: candidates,
    limit: 10,
  );

  print('Found ${matches.length} matches');
  if (matches.isNotEmpty) {
    print('Top match: ${matches.first.name} - Score: ${matches.first.matchScore}%');
  }
}
```

---

## Quick Reference: Scoring Formula

```
Final Score (0-100) = (
    distanceScore × 0.35 +
    ageScore × 0.20 +
    sportsScore × 0.25 +
    verifiedScore × 0.10 +
    heightScore × 0.10
) × 100

Where:
- distanceScore = e^(-distance / (maxDistance × 0.5))
- ageScore = 1 - (|age - ageRangeMid| / (ageRange / 2))
- sportsScore = (min(sharedSports, 5) / 5) × 2
- verifiedScore = 1.0 if verified else 0.0
- heightScore = 1 - (|height - heightRangeMid| / (heightRange / 2))
```
