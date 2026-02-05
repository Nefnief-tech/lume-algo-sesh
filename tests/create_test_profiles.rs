use reqwest::Client;
use serde_json::json;
use std::time::Duration;

const APPWRITE_ENDPOINT: &str = "https://fra.cloud.appwrite.io/v1";
const API_KEY: &str = "standard_661c0aadf919b14c97c3a4be7bd26bf6701b1e9e41fa9f8cecfa45a33fe22b2959f5312cf6d8468a74a2d4cb8fc293e99b0ba0a3dbc104b79326ced60c40a79e7b0f464b88ad37f2aeb1b31b901adaad0cf3f389e52340634f1e515d0edb2bf5c4f9e0b31c582c74cb418a287a76d1cb32ebe93e19854b31b5d775208f7f0cbe";
const PROJECT_ID: &str = "6899062700398ffeae4f";
const DATABASE_ID: &str = "threed-dating-db";
const PROFILES_COLLECTION: &str = "dating-profiles";
const PREFERENCES_COLLECTION: &str = "user-preferences";

// Test email for easy deletion
const TEST_EMAIL: &str = "test-profiles@lume-algo-test.local";

const NAMES: &[&str] = &[
    "Alex", "Jordan", "Taylor", "Morgan", "Casey", "Riley", "Quinn", "Avery",
    "Blake", "Carter", "Dakota", "Emerson", "Finley", "Gray", "Hayden", "Indigo",
    "Jade", "Kai", "Lake", "Milo", "Nova", "Onyx", "Phoenix", "River", "Sage",
    "Skyler", "Tatum", "Unity", "Valentine", "Willow", "Xavier", "Zion", "Luna",
    "Max", "Sam", "Charlie", "Drew", "Ellis", "Frankie", "Grayson", "Harper", "Ivy",
];

const HAIR_COLORS: &[&str] = &["blonde", "brown", "black", "white", "red", "gray", "other"];
const GENDERS: &[&str] = &["male", "female", "non_binary", "agender", "other"];
const SPORTS: &[&str] = &[
    "basketball", "football", "tennis", "cycling", "running", "swimming", "yoga",
    "martial_arts", "dancing", "hiking", "gym", "climbing", "skiing", "surfing",
    "boxing", "golf", "baseball", "soccer", "volleyball", "skating", "badminton",
];

const CITIES: &[(&str, f64, f64)] = &[
    ("Berlin", 52.5200, 13.4050),
    ("Munich", 48.1351, 11.5820),
    ("Hamburg", 53.5511, 9.9937),
    ("Cologne", 50.9375, 6.9603),
    ("Frankfurt", 50.1109, 8.6821),
    ("Stuttgart", 48.7758, 9.1829),
    ("Düsseldorf", 51.2277, 6.7735),
    ("Nuremberg", 49.4521, 11.0767),
    ("Leipzig", 51.3397, 12.3731),
    ("Dortmund", 51.5136, 7.4653),
];

// Simple random number generator using system time
fn get_seed() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64
}

fn rand_range(min: f64, max: f64) -> f64 {
    let seed = get_seed();
    let normalized = (seed as f64) / (u64::MAX as f64);
    min + normalized * (max - min)
}

fn rand_int(max: usize) -> usize {
    (get_seed() % max as u64) as usize
}

fn rand_choice_str_slice<'a>(options: &'a [&'a str]) -> &'a str {
    &options[rand_int(options.len())]
}

fn rand_choice_city(options: &[( &'static str, f64, f64)]) -> (&'static str, f64, f64) {
    let idx = rand_int(options.len());
    options[idx]
}

fn rand_choices_str(options: &[&str], count: usize) -> Vec<String> {
    let mut result = Vec::new();
    let mut used = std::collections::HashSet::new();
    let mut attempts = 0;
    while result.len() < count.min(options.len()) && attempts < 100 {
        let idx = rand_int(options.len());
        if used.insert(idx) {
            result.push(options[idx].to_string());
        }
        attempts += 1;
    }
    result
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client");

    println!("Creating 1000 test profiles in Appwrite...");
    println!("Database: {}/{}", DATABASE_ID, PROFILES_COLLECTION);
    println!("Test email domain: {}", TEST_EMAIL);
    println!();

    let batch_size = 50;
    let mut created = 0;
    let mut failed = 0;

    for batch in 0..(1000 / batch_size) {
        println!("Creating batch {} (profiles {}-{})...", batch + 1, batch * batch_size, (batch + 1) * batch_size);

        let mut profiles = Vec::new();
        let mut preferences = Vec::new();

        for i in 0..batch_size {
            tokio::time::sleep(Duration::from_millis(1)).await; // Seed variation

            let user_num = batch * batch_size + i;
            let user_id = format!("test_user_{:04}", user_num);
            let age = 18 + rand_int(72); // 18-90

            // Pick a city with some random offset
            let (city_name, base_lat, base_lon) = rand_choice_city(CITIES);
            let lat = base_lat + rand_range(-0.1, 0.1);
            let lon = base_lon + rand_range(-0.1, 0.1);

            let gender = rand_choice_str_slice(GENDERS);
            let hair_color = rand_choice_str_slice(HAIR_COLORS);
            let height_cm = 150 + rand_int(70); // 150-220 cm

            // Generate some sports preferences (1-5 sports)
            let sports_preferences: Vec<String> = rand_choices_str(SPORTS, 1 + rand_int(5));

            // Create dating profile - documentId is optional, omit it to let Appwrite auto-generate
            let profile = json!({
                "userId": user_id,
                "name": format!("{} {:?}", rand_choice_str_slice(NAMES), user_num),
                "age": age,
                "heightCm": height_cm,
                "hairColor": hair_color,
                "gender": gender,
                "latitude": lat,
                "longitude": lon,
                "isActive": true,
                "isVerified": rand_int(10) > 7, // 30% verified
                "isTimeout": false,
                "sportsPreferences": sports_preferences,
                "description": format!("Test profile from {}, looking for connections!", city_name),
                "imageFileIds": [],
                "email": format!("{}+{}@test", TEST_EMAIL, user_num),
            });
            profiles.push(profile);

            // Create preferences - realistic based on their own profile
            let preferred_genders: Vec<String> = if rand_int(3) > 0 {
                match gender {
                    "male" => vec!["female", "non_binary"],
                    "female" => vec!["male", "non_binary"],
                    "non_binary" => vec!["male", "female", "non_binary"],
                    _ => vec!["male", "female"],
                }
                .into_iter()
                .map(|s| s.to_string())
                .collect()
            } else {
                GENDERS.to_vec().into_iter().map(|s| s.to_string()).collect()
            };

            let min_age = (age as i16 - 5).max(18) as u16;
            let max_age = (age as i16 + 10).min(99) as u16;
            let min_height_cm = (height_cm as i16 - 10).max(140) as u16;
            let max_height_cm = (height_cm as i16 + 20).min(230) as u16;

            let preferred_hair_colors: Vec<String> = rand_choices_str(HAIR_COLORS, 2 + rand_int(4));
            let preferred_sports: Vec<String> = rand_choices_str(SPORTS, 3 + rand_int(6));
            let max_distance_km = 25 + rand_int(175); // 25-200 km

            let prefs = json!({
                "userId": user_id,
                "preferredGenders": preferred_genders,
                "minAge": min_age,
                "maxAge": max_age,
                "minHeightCm": min_height_cm,
                "maxHeightCm": max_height_cm,
                "preferredHairColors": preferred_hair_colors,
                "preferredSports": preferred_sports,
                "maxDistanceKm": max_distance_km,
                "latitude": lat,
                "longitude": lon,
                "notificationsEnabled": true,
            });
            preferences.push(prefs);
        }

        // Create profiles in batch
        let profile_base_url = format!(
            "{}/databases/{}/collections/{}/documents",
            APPWRITE_ENDPOINT.trim_end_matches('/'),
            DATABASE_ID,
            PROFILES_COLLECTION
        );

        let mut batch_created = 0;
        for (i, profile) in profiles.iter().enumerate() {
            // Generate a unique document ID for this profile
            let document_id = format!("test_profile_{:04}", i);
            let profile_url = format!("{}/{}", profile_base_url, document_id);

            // Debug: print the JSON for the first profile
            if i == 0 {
                eprintln!("DEBUG - URL: {}", profile_url);
                eprintln!("DEBUG - Sending JSON: {}", serde_json::to_string_pretty(profile).unwrap());
            }

            match client
                .post(&profile_url)
                .header("X-Appwrite-Key", API_KEY)
                .header("X-Appwrite-Project", PROJECT_ID)
                .json(profile)
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => batch_created += 1,
                Ok(resp) => {
                    let status = resp.status();
                    if let Ok(body) = resp.text().await {
                        eprintln!("Failed to create profile [{}]: {} - {}", i, status, body);
                    } else {
                        eprintln!("Failed to create profile [{}]: {}", i, status);
                    }
                }
                Err(e) => {
                    eprintln!("Error creating profile [{}]: {}", i, e);
                }
            }

            // Just do first profile for debugging
            if i == 0 {
                break;
            }
        }

        // Create preferences in batch
        let prefs_url = format!(
            "{}/databases/{}/collections/{}/documents",
            APPWRITE_ENDPOINT.trim_end_matches('/'),
            DATABASE_ID,
            PREFERENCES_COLLECTION
        );

        for pref in &preferences {
            match client
                .post(&prefs_url)
                .header("X-Appwrite-Key", API_KEY)
                .header("X-Appwrite-Project", PROJECT_ID)
                .json(pref)
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {}
                Ok(resp) => {
                    eprintln!("Failed to create prefs: {}", resp.status());
                }
                Err(e) => {
                    eprintln!("Error creating prefs: {}", e);
                }
            }
        }

        println!("  Created {} profiles in this batch", batch_created);
        created += batch_created;
        failed += batch_size - batch_created;

        // Small delay to avoid overwhelming the API
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    println!();
    println!("Done! Created {} profiles, {} failed", created, failed);
    println!();
    println!("To delete all test profiles, use this query in Appwrite:");
    println!("  query = startsWith(\"email\", \"{}+@test\")", TEST_EMAIL);
    println!();
    println!("Or filter by email contains: {}+@test", TEST_EMAIL);

    Ok(())
}
