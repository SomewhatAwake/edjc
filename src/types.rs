/*!
Type definitions for the EDJC plugin.

This module contains all the data structures used throughout the plugin
for representing Elite Dangerous game data and calculation results.
*/

use serde::{Deserialize, Serialize};

/// Information about a CMDR (player)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CmdrInfo {
    /// CMDR name
    pub cmdr_name: String,
    /// Current star system
    pub current_system: String,
    /// Current station (if docked)
    pub current_station: Option<String>,
}

/// Information about a ship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipInfo {
    /// Ship type (e.g., "Anaconda", "Asp Explorer")
    pub ship_type: String,
    /// Custom ship name
    pub ship_name: Option<String>,
    /// Minimum jump range in light years
    pub min_jump_range: f64,
    /// Maximum jump range in light years
    pub max_jump_range: f64,
}

/// 3D coordinates of a star system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemCoordinates {
    /// System name
    pub name: String,
    /// X coordinate
    pub x: f64,
    /// Y coordinate
    pub y: f64,
    /// Z coordinate
    pub z: f64,
    /// Whether the system has a neutron star
    pub has_neutron_star: bool,
    /// Whether the system has a white dwarf
    pub has_white_dwarf: bool,
}

/// Result of a jump calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JumpResult {
    /// Number of jumps required
    pub jumps: u32,
    /// Total distance in light years
    pub total_distance: f64,
    /// Type of route used
    pub route_type: String,
    /// Origin system name
    pub from_system: String,
    /// Destination system name
    pub to_system: String,
}

/// Information about a star system from various sources
#[derive(Debug, Clone)]
pub struct SystemInfo {
    /// System coordinates
    pub coordinates: SystemCoordinates,
    /// Distance from reference system (if applicable)
    pub distance_from_reference: Option<f64>,
    /// Population (0 for uninhabited)
    pub population: Option<u64>,
    /// Whether the system has stations
    pub has_stations: bool,
    /// Primary star information
    pub primary_star: Option<StarInfo>,
    /// System security level
    pub security: Option<SecurityLevel>,
}

/// Information about a star
#[derive(Debug, Clone)]
pub struct StarInfo {
    /// Star type (e.g., "G", "M", "Neutron Star")
    pub star_type: String,
    /// Star class (e.g., "G2V", "DA")
    pub star_class: String,
    /// Star mass in solar masses
    pub mass: Option<f64>,
    /// Star temperature in Kelvin
    pub temperature: Option<f64>,
    /// Whether this star can be used for FSD supercharging
    pub can_supercharge: bool,
    /// Supercharge multiplier (1.0 for no boost, 1.5 for white dwarf, 4.0 for neutron)
    pub supercharge_multiplier: f64,
}

/// System security levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SecurityLevel {
    High,
    Medium,
    Low,
    Lawless,
    Anarchy,
}

impl SecurityLevel {
    /// Convert from string representation
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "high" | "high security" => Some(SecurityLevel::High),
            "medium" | "medium security" => Some(SecurityLevel::Medium),
            "low" | "low security" => Some(SecurityLevel::Low),
            "lawless" => Some(SecurityLevel::Lawless),
            "anarchy" => Some(SecurityLevel::Anarchy),
            _ => None,
        }
    }

    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            SecurityLevel::High => "High Security",
            SecurityLevel::Medium => "Medium Security",
            SecurityLevel::Low => "Low Security",
            SecurityLevel::Lawless => "Lawless",
            SecurityLevel::Anarchy => "Anarchy",
        }
    }
}

/// Route planning options
#[derive(Debug, Clone)]
pub struct RouteOptions {
    /// Whether to use neutron stars for supercharging
    pub use_neutron_stars: bool,
    /// Whether to use white dwarfs for supercharging
    pub use_white_dwarfs: bool,
    /// Maximum detour distance for finding supercharge stars
    pub max_detour_ly: f64,
    /// Minimum fuel tank capacity in tons
    pub fuel_capacity: Option<f64>,
    /// Whether to avoid dangerous systems
    pub avoid_dangerous: bool,
    /// Whether to prefer populated systems
    pub prefer_populated: bool,
}

impl Default for RouteOptions {
    fn default() -> Self {
        Self {
            use_neutron_stars: true,
            use_white_dwarfs: true,
            max_detour_ly: 50.0,
            fuel_capacity: None,
            avoid_dangerous: true,
            prefer_populated: false,
        }
    }
}

/// Parsed RATSIGNAL information
#[derive(Debug, Clone)]
pub struct RatsignalInfo {
    /// Case number
    pub case_number: String,
    /// Platform (PC, PS, Xbox)
    pub platform: String,
    /// Game mode (Live, Odyssey, Horizons)
    pub mode: Option<String>,
    /// CMDR name in distress
    pub cmdr_name: String,
    /// System where the CMDR is located
    pub system_name: String,
    /// Additional system information (e.g., "Brown dwarf 51 LY from Fuelum")
    pub system_info: Option<String>,
    /// Language code
    pub language: Option<String>,
    /// Full original message
    pub raw_message: String,
}

/// Error types specific to EDJC operations
#[derive(Debug, thiserror::Error)]
pub enum EdjcError {
    #[error("Inara API error: {0}")]
    InaraApi(String),

    #[error("System not found: {0}")]
    SystemNotFound(String),

    #[error("CMDR not found: {0}")]
    CmdrNotFound(String),

    #[error("Invalid jump range: {0}")]
    InvalidJumpRange(f64),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Parsing error: {0}")]
    Parse(String),
}

/// Result type alias for EDJC operations
pub type EdjcResult<T> = Result<T, EdjcError>;

impl StarInfo {
    /// Create a new StarInfo for a neutron star
    pub fn neutron_star() -> Self {
        Self {
            star_type: "Neutron Star".to_string(),
            star_class: "N".to_string(),
            mass: None,
            temperature: None,
            can_supercharge: true,
            supercharge_multiplier: 4.0,
        }
    }

    /// Create a new StarInfo for a white dwarf
    pub fn white_dwarf(class: &str) -> Self {
        Self {
            star_type: "White Dwarf".to_string(),
            star_class: class.to_string(),
            mass: None,
            temperature: None,
            can_supercharge: true,
            supercharge_multiplier: 1.5,
        }
    }

    /// Create a new StarInfo for a regular star
    pub fn regular_star(star_type: &str, star_class: &str) -> Self {
        Self {
            star_type: star_type.to_string(),
            star_class: star_class.to_string(),
            mass: None,
            temperature: None,
            can_supercharge: false,
            supercharge_multiplier: 1.0,
        }
    }
}

impl SystemCoordinates {
    /// Calculate distance to another system
    pub fn distance_to(&self, other: &SystemCoordinates) -> f64 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        let dz = other.z - self.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Check if this system can provide FSD supercharging
    pub fn can_supercharge(&self) -> bool {
        self.has_neutron_star || self.has_white_dwarf
    }

    /// Get the supercharge multiplier for this system
    pub fn supercharge_multiplier(&self) -> f64 {
        if self.has_neutron_star {
            4.0
        } else if self.has_white_dwarf {
            1.5
        } else {
            1.0
        }
    }
}

impl JumpResult {
    /// Format the result as a human-readable string
    pub fn format(&self, template: &str) -> String {
        template
            .replace("{jumps}", &self.jumps.to_string())
            .replace("{distance}", &format!("{:.1}", self.total_distance))
            .replace("{system}", &self.to_system)
            .replace("{route}", &self.route_type)
            .replace("{from}", &self.from_system)
            .replace("{to}", &self.to_system)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_distance_calculation() {
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
            x: 3.03,
            y: 1.39,
            z: 0.16,
            has_neutron_star: false,
            has_white_dwarf: false,
        };

        let distance = sol.distance_to(&alpha_centauri);
        assert!((distance - 4.38).abs() < 0.1);
    }

    #[test]
    fn test_supercharge_multipliers() {
        let neutron_system = SystemCoordinates {
            name: "Test".to_string(),
            x: 0.0,
            y: 0.0,
            z: 0.0,
            has_neutron_star: true,
            has_white_dwarf: false,
        };

        let white_dwarf_system = SystemCoordinates {
            name: "Test".to_string(),
            x: 0.0,
            y: 0.0,
            z: 0.0,
            has_neutron_star: false,
            has_white_dwarf: true,
        };

        let normal_system = SystemCoordinates {
            name: "Test".to_string(),
            x: 0.0,
            y: 0.0,
            z: 0.0,
            has_neutron_star: false,
            has_white_dwarf: false,
        };

        assert_eq!(neutron_system.supercharge_multiplier(), 4.0);
        assert_eq!(white_dwarf_system.supercharge_multiplier(), 1.5);
        assert_eq!(normal_system.supercharge_multiplier(), 1.0);
    }

    #[test]
    fn test_security_level_parsing() {
        assert_eq!(SecurityLevel::from_str("high"), Some(SecurityLevel::High));
        assert_eq!(
            SecurityLevel::from_str("HIGH SECURITY"),
            Some(SecurityLevel::High)
        );
        assert_eq!(
            SecurityLevel::from_str("medium"),
            Some(SecurityLevel::Medium)
        );
        assert_eq!(SecurityLevel::from_str("invalid"), None);
    }

    #[test]
    fn test_jump_result_formatting() {
        let result = JumpResult {
            jumps: 5,
            total_distance: 123.45,
            route_type: "neutron highway".to_string(),
            from_system: "Sol".to_string(),
            to_system: "Colonia".to_string(),
        };

        let formatted = result.format("{jumps} jumps to {system} ({distance:.1}ly)");
        assert_eq!(formatted, "5 jumps to Colonia (123.5ly)");
    }
}
