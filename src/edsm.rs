/*!
EDSM API client for fe/// EDSM system response
#[derive(Debug, Deserialize)]
struct EdsmSystemResponse {
    name: String,
    coords: Option<EdsmCoordinates>,
    #[serde(rename = "primaryStar")]
    primary_star: Option<EdsmStar>,
}

#[derive(Debug, Deserialize)]
struct EdsmCoordinates {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Debug, Deserialize)]
struct EdsmStar {
    #[serde(rename = "type")]
    star_type: Option<String>,
    #[serde(rename = "subType")]
    sub_type: Option<String>,
}Dangerous system coordinates.

This module handles communication with the EDSM API to retrieve system coordinates
for jump calculations.
*/

use anyhow::{anyhow, Result};
use log::debug;
use moka::sync::Cache;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::time::Duration;

use crate::types::SystemCoordinates;

const EDSM_API_URL: &str = "https://www.edsm.net/api-v1";
const CACHE_TTL_SECONDS: u64 = 3600; // 1 hour (EDSM data changes rarely)

/// EDSM API client
#[derive(Debug)]
pub struct EdsmClient {
    client: Client,
    cache: Cache<String, String>,
}

/// EDSM system response
#[derive(Debug, Deserialize)]
struct EdsmSystemResponse {
    name: String,
    coords: Option<EdsmCoordinates>,
    #[serde(rename = "primaryStar")]
    primary_star: Option<EdsmStar>,
}

#[derive(Debug, Deserialize)]
struct EdsmCoordinates {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Debug, Deserialize)]
struct EdsmStar {
    #[serde(rename = "type")]
    star_type: Option<String>,
    #[serde(rename = "subType")]
    sub_type: Option<String>,
}

impl EdsmClient {
    /// Create a new EDSM client
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Elite Dangerous Jump Calculator/0.1.0")
            .build()?;

        let cache = Cache::builder()
            .time_to_live(Duration::from_secs(CACHE_TTL_SECONDS))
            .max_capacity(1000)
            .build();

        Ok(Self { client, cache })
    }

    /// Get system coordinates from EDSM
    pub fn get_system_coordinates(&self, system_name: &str) -> Result<SystemCoordinates> {
        let cache_key = format!("coords:{}", system_name.to_lowercase());

        // Check cache first
        if let Some(cached) = self.cache.get(&cache_key) {
            if let Ok(coords) = serde_json::from_str::<SystemCoordinates>(&cached) {
                debug!("Cache hit for system coordinates: {system_name}");
                return Ok(coords);
            }
        }

        debug!("Fetching coordinates for system: {system_name}");

        let url = format!("{EDSM_API_URL}/system");
        let response = self
            .client
            .get(&url)
            .query(&[
                ("systemName", system_name),
                ("showCoordinates", "1"),
                ("showPrimaryStar", "1"),
            ])
            .send()?;

        if !response.status().is_success() {
            return Err(anyhow!("EDSM API request failed: {}", response.status()));
        }

        let system_data: EdsmSystemResponse = response.json()?;

        let coords = system_data
            .coords
            .ok_or_else(|| anyhow!("System '{}' not found or has no coordinates", system_name))?;

        // Determine if system has neutron star or white dwarf
        let (has_neutron_star, has_white_dwarf) = if let Some(star) = &system_data.primary_star {
            let star_type = star.star_type.as_deref().unwrap_or("");
            let sub_type = star.sub_type.as_deref().unwrap_or("");

            let has_neutron = star_type.contains("Neutron") || sub_type.contains("Neutron");
            let has_white_dwarf = star_type.contains("White Dwarf")
                || sub_type.contains("DA")
                || sub_type.contains("DB")
                || sub_type.contains("DC");

            (has_neutron, has_white_dwarf)
        } else {
            (false, false)
        };

        let coordinates = SystemCoordinates {
            name: system_data.name,
            x: coords.x,
            y: coords.y,
            z: coords.z,
            has_neutron_star,
            has_white_dwarf,
        };

        // Cache the result
        if let Ok(cached_data) = serde_json::to_string(&coordinates) {
            self.cache.insert(cache_key, cached_data);
        }

        Ok(coordinates)
    }

    /// Calculate distance between two systems
    pub fn calculate_distance(&self, from_system: &str, to_system: &str) -> Result<f64> {
        let from_coords = self.get_system_coordinates(from_system)?;
        let to_coords = self.get_system_coordinates(to_system)?;
        Ok(calculate_3d_distance(&from_coords, &to_coords))
    }

    /// Test connection to EDSM by looking up Sol
    pub fn test_connection(&self) -> Result<bool> {
        debug!("Testing EDSM connection with Sol system");

        match self.get_system_coordinates("Sol") {
            Ok(coords) => {
                // Sol should be at (0, 0, 0)
                let distance_from_origin =
                    (coords.x.powi(2) + coords.y.powi(2) + coords.z.powi(2)).sqrt();
                Ok(distance_from_origin < 1.0)
            }
            Err(_) => Ok(false),
        }
    }
}

/// Calculate 3D distance between two system coordinates
fn calculate_3d_distance(from: &SystemCoordinates, to: &SystemCoordinates) -> f64 {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let dz = to.z - from.z;
    (dx.powi(2) + dy.powi(2) + dz.powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_calculation() {
        let sol = SystemCoordinates {
            name: "Sol".to_string(),
            x: 0.0,
            y: 0.0,
            z: 0.0,
            has_neutron_star: false,
            has_white_dwarf: false,
        };

        let alpha_centauri = SystemCoordinates {
            name: "Alpha Centauri".to_string(),
            x: 3.03125,
            y: -0.09375,
            z: 3.15625,
            has_neutron_star: false,
            has_white_dwarf: false,
        };

        let distance = calculate_3d_distance(&sol, &alpha_centauri);
        // Alpha Centauri is approximately 4.3 LY from Sol
        assert!((distance - 4.3).abs() < 0.5);
    }

    #[test]
    fn test_large_distance_calculation() {
        let sol = SystemCoordinates {
            name: "Sol".to_string(),
            x: 0.0,
            y: 0.0,
            z: 0.0,
            has_neutron_star: false,
            has_white_dwarf: false,
        };

        // Sagittarius A* coordinates (approximate)
        let sagittarius_a = SystemCoordinates {
            name: "Sagittarius A*".to_string(),
            x: 25.21875,
            y: -20.90625,
            z: 25899.96875,
            has_neutron_star: false,
            has_white_dwarf: false,
        };

        let distance = calculate_3d_distance(&sol, &sagittarius_a);
        // Sagittarius A* is approximately 25,900 LY from Sol
        assert!((distance - 25900.0).abs() < 100.0);
    }
}
