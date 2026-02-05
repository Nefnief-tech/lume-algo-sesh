use crate::models::BoundingBox;

/// Earth's radius in kilometers
const EARTH_RADIUS_KM: f64 = 6371.0;

/// Calculate the Haversine distance between two points in kilometers
///
/// # Arguments
/// * `lat1` - Latitude of first point in degrees
/// * `lon1` - Longitude of first point in degrees
/// * `lat2` - Latitude of second point in degrees
/// * `lon2` - Longitude of second point in degrees
///
/// # Returns
/// Distance in kilometers
#[inline]
pub fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let delta_lat = (lat2 - lat1).to_radians();
    let delta_lon = (lon2 - lon1).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS_KM * c
}

/// Calculate a bounding box around a center point
///
/// This is much faster than Haversine for pre-filtering.
/// 1° latitude ≈ 111km, 1° longitude ≈ 111km * cos(latitude)
///
/// # Arguments
/// * `lat` - Center latitude in degrees
/// * `lon` - Center longitude in degrees
/// * `radius_km` - Radius in kilometers
///
/// # Returns
/// BoundingBox with min/max lat/lon
pub fn calculate_bounding_box(lat: f64, lon: f64, radius_km: f64) -> BoundingBox {
    // 1 degree latitude is approximately 111 km
    let lat_delta = radius_km / 111.0;

    // 1 degree longitude varies by latitude
    let lon_delta = radius_km / (111.0 * lat.to_radians().cos().abs());

    BoundingBox {
        min_lat: lat - lat_delta,
        max_lat: lat + lat_delta,
        min_lon: lon - lon_delta,
        max_lon: lon + lon_delta,
    }
}

/// Check if a point is within a bounding box
#[inline]
pub fn is_within_bounding_box(
    lat: f64,
    lon: f64,
    bbox: &BoundingBox,
) -> bool {
    lat >= bbox.min_lat
        && lat <= bbox.max_lat
        && lon >= bbox.min_lon
        && lon <= bbox.max_lon
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haversine_distance() {
        // Distance from London to Paris (approximately 344 km)
        let london_lat = 51.5074;
        let london_lon = -0.1278;
        let paris_lat = 48.8566;
        let paris_lon = 2.3522;

        let distance = haversine_distance(london_lat, london_lon, paris_lat, paris_lon);
        assert!((distance - 344.0).abs() < 10.0, "Distance should be ~344km, got {}", distance);
    }

    #[test]
    fn test_bounding_box() {
        let bbox = calculate_bounding_box(40.7128, -74.0060, 10.0);

        assert!(bbox.min_lat < 40.7128);
        assert!(bbox.max_lat > 40.7128);
        assert!(bbox.min_lon < -74.0060);
        assert!(bbox.max_lon > -74.0060);

        // Check approximate size (20km / 111km per degree = ~0.18 degrees)
        let lat_span = bbox.max_lat - bbox.min_lat;
        assert!((lat_span - 0.18).abs() < 0.02, "Lat span should be ~0.18 degrees");
    }

    #[test]
    fn test_point_within_bbox() {
        let bbox = calculate_bounding_box(40.7128, -74.0060, 10.0);

        // Center point should be within
        assert!(is_within_bounding_box(40.7128, -74.0060, &bbox));

        // Close point should be within
        assert!(is_within_bounding_box(40.71, -74.0, &bbox));

        // Far point should not be within
        assert!(!is_within_bounding_box(50.0, -80.0, &bbox));
    }
}
