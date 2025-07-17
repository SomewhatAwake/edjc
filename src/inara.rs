/*!
Inara API client for fetching Elite Dangerous data.

This module handles communication with the Inara API to retrieve:
- CMDR current location
- Ship information and jump ranges
- System coordinates and data
*/

use anyhow::{anyhow, Result};
use log::debug;
use moka::sync::Cache;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::types::{CmdrInfo, ShipInfo, SystemCoordinates};

const INARA_API_URL: &str = "https://inara.cz/inapi/v1/";
const CACHE_TTL_SECONDS: u64 = 300; // 5 minutes

/// Inara API client
#[derive(Debug)]
pub struct InaraClient {
    client: Client,
    api_key: String,
    cache: Cache<String, String>,
}

/// Inara API request structure
#[derive(Serialize)]
struct InaraRequest {
    header: InaraHeader,
    events: Vec<InaraEvent>,
}

#[derive(Serialize)]
struct InaraHeader {
    #[serde(rename = "appName")]
    app_name: String,
    #[serde(rename = "appVersion")]
    app_version: String,
    #[serde(rename = "isDeveloped")]
    is_developed: bool,
    #[serde(rename = "APIkey")]
    api_key: String,
}

#[derive(Serialize)]
struct InaraEvent {
    #[serde(rename = "eventName")]
    event_name: String,
    #[serde(rename = "eventTimestamp")]
    event_timestamp: String,
    #[serde(rename = "eventData")]
    event_data: serde_json::Value,
}

/// Inara API response structure
#[derive(Deserialize)]
struct InaraResponse {
    header: InaraResponseHeader,
    events: Vec<InaraEventResponse>,
}

#[derive(Deserialize)]
struct InaraResponseHeader {
    #[serde(rename = "eventStatus")]
    event_status: i32,
    #[serde(rename = "eventStatusText")]
    event_status_text: Option<String>,
}

#[derive(Deserialize)]
struct InaraEventResponse {
    #[serde(rename = "eventStatus")]
    event_status: i32,
    #[serde(rename = "eventStatusText")]
    event_status_text: Option<String>,
    #[serde(rename = "eventData")]
    event_data: Option<serde_json::Value>,
}

/// CMDR location response from Inara
#[derive(Deserialize)]
struct CmdrLocationResponse {
    #[serde(rename = "commanderName")]
    commander_name: String,
    #[serde(rename = "starsystemName")]
    starsystem_name: String,
    #[serde(rename = "stationName")]
    station_name: Option<String>,
}

/// Ship data response from Inara
#[derive(Deserialize)]
struct ShipResponse {
    #[serde(rename = "shipType")]
    ship_type: String,
    #[serde(rename = "shipName")]
    ship_name: Option<String>,
    #[serde(rename = "shipIdent")]
    ship_ident: Option<String>,
    #[serde(rename = "isCurrentShip")]
    is_current_ship: bool,
    #[serde(rename = "jumpRangeMin")]
    jump_range_min: Option<f64>,
    #[serde(rename = "jumpRangeMax")]
    jump_range_max: Option<f64>,
}

/// System data response from Inara
#[derive(Deserialize)]
struct SystemResponse {
    #[serde(rename = "systemName")]
    system_name: String,
    #[serde(rename = "systemCoordinates")]
    system_coordinates: Vec<f64>,
    #[serde(rename = "primaryStar")]
    primary_star: Option<StarResponse>,
}

#[derive(Deserialize)]
struct StarResponse {
    #[serde(rename = "starType")]
    star_type: String,
    #[serde(rename = "starClass")]
    star_class: String,
}

impl InaraClient {
    /// Create a new Inara API client
    pub fn new(api_key: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("EDJC-HexChat-Plugin/0.1.0")
            .build()?;

        let cache = Cache::builder()
            .time_to_live(Duration::from_secs(CACHE_TTL_SECONDS))
            .max_capacity(1000)
            .build();

        Ok(Self {
            client,
            api_key,
            cache,
        })
    }

    /// Test API connection and verify CMDR exists
    pub fn test_connection(&self, cmdr_name: &str) -> Result<bool> {
        let request = InaraRequest {
            header: self.create_header(),
            events: vec![InaraEvent {
                event_name: "getCommanderProfile".to_string(),
                event_timestamp: chrono::Utc::now().to_rfc3339(),
                event_data: serde_json::json!({
                    "commanderName": cmdr_name
                }),
            }],
        };

        let response: InaraResponse = self
            .client
            .post(INARA_API_URL)
            .json(&request)
            .send()?
            .json()?;

        if let Some(event) = response.events.first() {
            if event.event_status == 200 {
                Ok(true) // CMDR found
            } else if event.event_status == 204 {
                Ok(false) // CMDR not found
            } else {
                Err(anyhow!(
                    "Inara API error {}: {}",
                    event.event_status,
                    event
                        .event_status_text
                        .as_deref()
                        .unwrap_or("Unknown error")
                ))
            }
        } else {
            Err(anyhow!("No response from Inara API"))
        }
    }

    /// Get CMDR current location
    pub fn get_cmdr_location(&self, cmdr_name: &str) -> Result<CmdrInfo> {
        let cache_key = format!("cmdr_location_{}", cmdr_name);

        if let Some(cached) = self.cache.get(&cache_key) {
            if let Ok(info) = serde_json::from_str::<CmdrInfo>(&cached) {
                debug!("Using cached CMDR location for {}", cmdr_name);
                return Ok(info);
            }
        }

        let request = InaraRequest {
            header: self.create_header(),
            events: vec![InaraEvent {
                event_name: "getCommanderProfile".to_string(),
                event_timestamp: chrono::Utc::now().to_rfc3339(),
                event_data: serde_json::json!({
                    "commanderName": cmdr_name
                }),
            }],
        };

        let response: InaraResponse = self
            .client
            .post(INARA_API_URL)
            .json(&request)
            .send()?
            .json()?;

        if let Some(event) = response.events.first() {
            if event.event_status != 200 {
                return Err(anyhow!(
                    "Inara API error: {}",
                    event
                        .event_status_text
                        .as_deref()
                        .unwrap_or("Unknown error")
                ));
            }

            if let Some(data) = &event.event_data {
                let location: CmdrLocationResponse = serde_json::from_value(data.clone())?;
                let cmdr_info = CmdrInfo {
                    cmdr_name: location.commander_name,
                    current_system: location.starsystem_name,
                    current_station: location.station_name,
                };

                // Cache the result
                let cached_data = serde_json::to_string(&cmdr_info)?;
                self.cache.insert(cache_key, cached_data);

                return Ok(cmdr_info);
            }
        }

        Err(anyhow!("No CMDR location data received from Inara"))
    }

    /// Get ship information for a CMDR
    pub fn get_ship_info(&self, cmdr_name: &str) -> Result<ShipInfo> {
        let cache_key = format!("ship_info_{}", cmdr_name);

        if let Some(cached) = self.cache.get(&cache_key) {
            if let Ok(info) = serde_json::from_str::<ShipInfo>(&cached) {
                debug!("Using cached ship info for {}", cmdr_name);
                return Ok(info);
            }
        }

        let request = InaraRequest {
            header: self.create_header(),
            events: vec![InaraEvent {
                event_name: "getCommanderShips".to_string(),
                event_timestamp: chrono::Utc::now().to_rfc3339(),
                event_data: serde_json::json!({
                    "commanderName": cmdr_name
                }),
            }],
        };

        let response: InaraResponse = self
            .client
            .post(INARA_API_URL)
            .json(&request)
            .send()?
            .json()?;

        if let Some(event) = response.events.first() {
            if event.event_status != 200 {
                return Err(anyhow!(
                    "Inara API error: {}",
                    event
                        .event_status_text
                        .as_deref()
                        .unwrap_or("Unknown error")
                ));
            }

            if let Some(data) = &event.event_data {
                let ships: Vec<ShipResponse> = serde_json::from_value(data.clone())?;

                // Find the current ship
                if let Some(current_ship) = ships.iter().find(|ship| ship.is_current_ship) {
                    let ship_info = ShipInfo {
                        ship_type: current_ship.ship_type.clone(),
                        ship_name: current_ship.ship_name.clone(),
                        min_jump_range: current_ship.jump_range_min.unwrap_or(10.0),
                        max_jump_range: current_ship.jump_range_max.unwrap_or(20.0),
                    };

                    // Cache the result
                    let cached_data = serde_json::to_string(&ship_info)?;
                    self.cache.insert(cache_key, cached_data);

                    return Ok(ship_info);
                }
            }
        }

        Err(anyhow!("No current ship data found for CMDR {}", cmdr_name))
    }

    /// Get system coordinates
    pub fn get_system_coordinates(&self, system_name: &str) -> Result<SystemCoordinates> {
        let cache_key = format!("system_{}", system_name);

        if let Some(cached) = self.cache.get(&cache_key) {
            if let Ok(coords) = serde_json::from_str::<SystemCoordinates>(&cached) {
                debug!("Using cached coordinates for {}", system_name);
                return Ok(coords);
            }
        }

        let request = InaraRequest {
            header: self.create_header(),
            events: vec![InaraEvent {
                event_name: "getSystem".to_string(),
                event_timestamp: chrono::Utc::now().to_rfc3339(),
                event_data: serde_json::json!({
                    "systemName": system_name
                }),
            }],
        };

        let response: InaraResponse = self
            .client
            .post(INARA_API_URL)
            .json(&request)
            .send()?
            .json()?;

        if let Some(event) = response.events.first() {
            if event.event_status != 200 {
                return Err(anyhow!(
                    "Inara API error: {}",
                    event
                        .event_status_text
                        .as_deref()
                        .unwrap_or("Unknown error")
                ));
            }

            if let Some(data) = &event.event_data {
                let system: SystemResponse = serde_json::from_value(data.clone())?;

                if system.system_coordinates.len() >= 3 {
                    let coords = SystemCoordinates {
                        name: system.system_name,
                        x: system.system_coordinates[0],
                        y: system.system_coordinates[1],
                        z: system.system_coordinates[2],
                        has_neutron_star: system
                            .primary_star
                            .as_ref()
                            .map(|star| star.star_type.to_lowercase().contains("neutron"))
                            .unwrap_or(false),
                        has_white_dwarf: system
                            .primary_star
                            .as_ref()
                            .map(|star| star.star_class.to_uppercase().starts_with("D"))
                            .unwrap_or(false),
                    };

                    // Cache the result
                    let cached_data = serde_json::to_string(&coords)?;
                    self.cache.insert(cache_key, cached_data);

                    return Ok(coords);
                }
            }
        }

        Err(anyhow!(
            "No coordinate data found for system {}",
            system_name
        ))
    }

    /// Create request header
    fn create_header(&self) -> InaraHeader {
        InaraHeader {
            app_name: "EDJC-HexChat-Plugin".to_string(),
            app_version: "0.1.0".to_string(),
            is_developed: true,
            api_key: self.api_key.clone(),
        }
    }
}
