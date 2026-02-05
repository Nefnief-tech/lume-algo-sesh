// Criterion benchmarks for Lume Algo

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use lume_algo::core::{Matcher, distance::{haversine_distance, calculate_bounding_box}};
use lume_algo::models::{UserProfile, UserPreferences};
use chrono::Utc;

fn create_candidate(id: usize, lat: f64, lon: f64) -> UserProfile {
    UserProfile {
        user_id: id.to_string(),
        name: format!("User {}", id),
        age: 25 + (id % 10) as u8,
        height_cm: 160 + (id % 30) as u16,
        hair_color: "brown".to_string(),
        gender: if id % 2 == 0 { "female" } else { "male" }.to_string(),
        latitude: lat,
        longitude: lon,
        is_verified: id % 3 == 0,
        is_active: true,
        is_timeout: false,
        image_file_ids: vec![],
        description: None,
        sports_preferences: vec!["tennis".to_string()],
        created_at: Utc::now(),
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
        latitude: 40.7128,
        longitude: -74.0060,
    }
}

fn bench_haversine_distance(c: &mut Criterion) {
    c.bench_function("haversine_distance", |b| {
        b.iter(|| {
            haversine_distance(
                black_box(40.7128),
                black_box(-74.0060),
                black_box(40.72),
                black_box(-74.01),
            )
        });
    });
}

fn bench_bounding_box(c: &mut Criterion) {
    c.bench_function("bounding_box_calculation", |b| {
        b.iter(|| {
            calculate_bounding_box(
                black_box(40.7128),
                black_box(-74.0060),
                black_box(50.0),
            )
        });
    });
}

fn bench_matching(c: &mut Criterion) {
    let matcher = Matcher::with_default_weights();
    let preferences = create_preferences();

    let mut group = c.benchmark_group("matching");

    for candidate_count in [10, 50, 100, 500, 1000].iter() {
        let candidates: Vec<UserProfile> = (0..*candidate_count)
            .map(|i| {
                let lat_offset = (i as f64 * 0.001) % 0.5;
                let lon_offset = (i as f64 * 0.001) % 0.5;
                create_candidate(i, 40.7128 + lat_offset, -74.0060 + lon_offset)
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::new("find_matches", candidate_count),
            candidate_count,
            |b, _| {
                b.iter(|| {
                    matcher.find_matches(
                        black_box(&preferences),
                        black_box(candidates.clone()),
                        black_box(20),
                    )
                });
            },
        );
    }

    group.finish();
}

fn bench_filtering_pipeline(c: &mut Criterion) {
    let preferences = create_preferences();
    let candidates: Vec<UserProfile> = (0..100)
        .map(|i| {
            let lat_offset = (i as f64 * 0.001) % 0.5;
            let lon_offset = (i as f64 * 0.001) % 0.5;
            create_candidate(i, 40.7128 + lat_offset, -74.0060 + lon_offset)
        })
        .collect();

    c.bench_function("filtering_pipeline_100_candidates", |b| {
        b.iter(|| {
            let bbox = calculate_bounding_box(
                preferences.latitude,
                preferences.longitude,
                preferences.max_distance_km as f64,
            );

            let filtered: Vec<_> = candidates
                .iter()
                .filter(|p| {
                    haversine_distance(
                        preferences.latitude,
                        preferences.longitude,
                        p.latitude,
                        p.longitude,
                    ) < preferences.max_distance_km as f64
                })
                .filter(|p| p.is_active && !p.is_timeout)
                .filter(|p| preferences.preferred_genders.contains(&p.gender))
                .filter(|p| p.age >= preferences.min_age && p.age <= preferences.max_age)
                .collect();

            black_box(filtered)
        });
    });
}

criterion_group!(
    benches,
    bench_haversine_distance,
    bench_bounding_box,
    bench_matching,
    bench_filtering_pipeline
);

criterion_main!(benches);
