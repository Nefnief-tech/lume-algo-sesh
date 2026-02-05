/// Test data generator for Lume Algo
///
/// Generates CSV files containing test profiles and preferences
/// that can be imported via Appwrite Console.
///
/// Run: cargo run --bin generate-test-data

use std::fs::File;
use std::io::{BufWriter, Write};

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

struct Profile {
    document_id: String,
    user_id: String,
    name: String,
    age: u8,
    height_cm: u16,
    hair_color: String,
    gender: String,
    latitude: f64,
    longitude: f64,
    is_active: bool,
    is_verified: bool,
    is_timeout: bool,
    sports_preferences: String,
    description: String,
    image_file_ids: String,
    created_at: String,
    updated_at: String,
    email: String,
}

struct Preferences {
    document_id: String,
    user_id: String,
    preferred_genders: String,
    min_age: u16,
    max_age: u16,
    min_height_cm: u16,
    max_height_cm: u16,
    preferred_hair_colors: String,
    preferred_sports: String,
    max_distance_km: u16,
    latitude: f64,
    longitude: f64,
    updated_at: String,
    notifications_enabled: bool,
}

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

fn format_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    format!("{}000", secs) // Convert to milliseconds format
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace("\"", "\"\""))
    } else {
        s.to_string()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let num_profiles = 1000;

    println!("Generating {} test profiles...", num_profiles);

    let mut profiles = Vec::new();
    let mut preferences = Vec::new();

    for user_num in 0..num_profiles {
        std::thread::sleep(std::time::Duration::from_millis(1)); // Seed variation

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

        let timestamp = format_timestamp();
        let is_verified = rand_int(10) > 7; // 30% verified

        let profile = Profile {
            document_id: format!("test_profile_{:04}", user_num),
            user_id: user_id.clone(),
            name: format!("{} {:?}", rand_choice_str_slice(NAMES), user_num),
            age: age as u8,
            height_cm: height_cm as u16,
            hair_color: hair_color.to_string(),
            gender: gender.to_string(),
            latitude: lat,
            longitude: lon,
            is_active: true,
            is_verified,
            is_timeout: false,
            sports_preferences: format!("[\"{}\"]", sports_preferences.join("\",\"")),
            description: format!("Test profile from {}, looking for connections!", city_name),
            image_file_ids: "[]".to_string(),
            created_at: timestamp.clone(),
            updated_at: timestamp.clone(),
            email: format!("{}+{}@test", TEST_EMAIL, user_num),
        };
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

        let prefs = Preferences {
            document_id: format!("test_prefs_{:04}", user_num),
            user_id,
            preferred_genders: format!("[\"{}\"]", preferred_genders.join("\",\"")),
            min_age,
            max_age,
            min_height_cm,
            max_height_cm,
            preferred_hair_colors: format!("[\"{}\"]", preferred_hair_colors.join("\",\"")),
            preferred_sports: format!("[\"{}\"]", preferred_sports.join("\",\"")),
            max_distance_km: max_distance_km as u16,
            latitude: lat,
            longitude: lon,
            updated_at: timestamp,
            notifications_enabled: true,
        };
        preferences.push(prefs);
    }

    // Write profiles CSV
    let mut profiles_csv = BufWriter::new(File::create("test_profiles.csv")?);
    writeln!(
        profiles_csv,
        "userId,name,age,heightCm,hairColor,gender,latitude,longitude,isActive,isVerified,isTimeout,sportsPreferences,description,imageFileIds,createdAt,updatedAt,email"
    )?;
    for p in &profiles {
        writeln!(
            profiles_csv,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            escape_csv(&p.user_id),
            escape_csv(&p.name),
            p.age,
            p.height_cm,
            escape_csv(&p.hair_color),
            escape_csv(&p.gender),
            p.latitude,
            p.longitude,
            p.is_active,
            p.is_verified,
            p.is_timeout,
            escape_csv(&p.sports_preferences),
            escape_csv(&p.description),
            escape_csv(&p.image_file_ids),
            escape_csv(&p.created_at),
            escape_csv(&p.updated_at),
            escape_csv(&p.email),
        )?;
    }
    println!("Created test_profiles.csv with {} profiles", profiles.len());

    // Write preferences CSV
    let mut prefs_csv = BufWriter::new(File::create("test_preferences.csv")?);
    writeln!(
        prefs_csv,
        "userId,preferredGenders,minAge,maxAge,minHeightCm,maxHeightCm,preferredHairColors,preferredSports,maxDistanceKm,latitude,longitude,updatedAt,notificationsEnabled"
    )?;
    for p in &preferences {
        writeln!(
            prefs_csv,
            "{},{},{},{},{},{},{},{},{},{},{},{},{}",
            escape_csv(&p.user_id),
            escape_csv(&p.preferred_genders),
            p.min_age,
            p.max_age,
            p.min_height_cm,
            p.max_height_cm,
            escape_csv(&p.preferred_hair_colors),
            escape_csv(&p.preferred_sports),
            p.max_distance_km,
            p.latitude,
            p.longitude,
            escape_csv(&p.updated_at),
            p.notifications_enabled,
        )?;
    }
    println!("Created test_preferences.csv with {} preferences", preferences.len());

    println!();
    println!("To delete all test profiles, use this query in Appwrite:");
    println!("  query = startsWith(\"userId\", \"test_user_\")");
    println!();

    Ok(())
}
