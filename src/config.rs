/*!
Configuration management for the EDJC plugin.

This module handles loading and managing configuration settings,
including API keys and plugin preferences.
*/

use anyhow::{anyhow, Result};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// CMDR name for location lookups
    pub cmdr_name: String,

    /// Ship name and jump range configuration
    pub ship: ShipConfig,

    /// Cache timeout in seconds
    #[serde(default = "default_cache_timeout")]
    pub cache_timeout_seconds: u64,

    /// Enable debug logging
    #[serde(default)]
    pub debug_mode: bool,

    /// Minimum distance threshold for suggesting neutron highway
    #[serde(default = "default_neutron_threshold")]
    pub neutron_highway_threshold_ly: f64,

    /// Minimum distance threshold for suggesting white dwarf assistance
    #[serde(default = "default_white_dwarf_threshold")]
    pub white_dwarf_threshold_ly: f64,

    /// Format string for jump calculation results
    #[serde(default = "default_result_format")]
    pub result_format: String,

    /// Whether to show fuel estimates
    #[serde(default = "default_show_fuel")]
    pub show_fuel_estimates: bool,

    /// Whether to show time estimates
    #[serde(default = "default_show_time")]
    pub show_time_estimates: bool,
}

/// Ship configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipConfig {
    /// Ship name/type (e.g., "Anaconda", "Asp Explorer")
    pub name: String,

    /// Laden jump range in light years (realistic jump range with cargo/fuel)
    pub laden_jump_range: f64,

    /// Optional: Maximum jump range (empty/optimized)
    #[serde(default)]
    pub max_jump_range: Option<f64>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cmdr_name: String::new(),
            ship: ShipConfig::default(),
            cache_timeout_seconds: default_cache_timeout(),
            debug_mode: false,
            neutron_highway_threshold_ly: default_neutron_threshold(),
            white_dwarf_threshold_ly: default_white_dwarf_threshold(),
            result_format: default_result_format(),
            show_fuel_estimates: default_show_fuel(),
            show_time_estimates: default_show_time(),
        }
    }
}

impl Default for ShipConfig {
    fn default() -> Self {
        Self {
            name: "Unknown Ship".to_string(),
            laden_jump_range: 30.0, // Reasonable default
            max_jump_range: None,
        }
    }
}

// Default value functions
fn default_cache_timeout() -> u64 {
    300
} // 5 minutes
fn default_neutron_threshold() -> f64 {
    500.0
}
fn default_white_dwarf_threshold() -> f64 {
    150.0
}
fn default_result_format() -> String {
    "🚀 {jumps} jumps to {system} ({distance:.1}ly) via {route}".to_string()
}
fn default_show_fuel() -> bool {
    false
}
fn default_show_time() -> bool {
    false
}

/// Load configuration from file or create default
pub fn load_config() -> Result<Config> {
    let config_path = get_config_path()?;

    if config_path.exists() {
        info!("Loading configuration from: {config_path:?}");
        let config_content = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&config_content)
            .map_err(|e| anyhow!("Failed to parse config file: {}", e))?;

        // Validate required settings
        if config.cmdr_name.is_empty() {
            warn!("CMDR name not configured. Please set it in the config file.");
        }

        if config.ship.laden_jump_range <= 0.0 {
            warn!("Invalid ship jump range configured. Using default.");
        }

        Ok(config)
    } else {
        info!("Configuration file not found, creating default: {config_path:?}");
        let config = Config::default();
        save_config(&config)?;
        Ok(config)
    }
}

/// Save configuration to file
pub fn save_config(config: &Config) -> Result<()> {
    let config_path = get_config_path()?;

    // Create config directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let config_content = toml::to_string_pretty(config)?;
    fs::write(&config_path, config_content)?;

    info!("Configuration saved to: {config_path:?}");
    Ok(())
}

/// Get the configuration file path
pub fn get_config_path() -> Result<PathBuf> {
    let config_dir = get_config_directory()?;
    Ok(config_dir.join("edjc.toml"))
}

/// Get the configuration directory
fn get_config_directory() -> Result<PathBuf> {
    // Try to use XDG config directory on Unix, AppData on Windows
    if let Ok(config_dir) = std::env::var("XDG_CONFIG_HOME") {
        Ok(PathBuf::from(config_dir).join("edjc"))
    } else if let Ok(appdata) = std::env::var("APPDATA") {
        Ok(PathBuf::from(appdata).join("EDJC"))
    } else if let Ok(home) = std::env::var("HOME") {
        Ok(PathBuf::from(home).join(".config").join("edjc"))
    } else {
        // Fallback to current directory
        Ok(PathBuf::from(".").join("config"))
    }
}

/// Create a sample configuration file with instructions
pub fn create_sample_config() -> Result<()> {
    let config_path = get_config_path()?;

    if config_path.exists() {
        return Err(anyhow!(
            "Configuration file already exists at: {:?}",
            config_path
        ));
    }

    let sample_config = r#"# EDJC (Elite Dangerous Jump Calculator) Configuration
# 
# This plugin uses EDSM (Elite Dangerous Star Map) for system coordinates
# and jump calculations. No API key is required for EDSM.

# Your CMDR name (required) - this is your Elite Dangerous pilot name
cmdr_name = "YOUR_CMDR_NAME"

# Ship configuration
[ship]
# Ship name/type (for display purposes)
name = "Asp Explorer"
# Laden jump range in light years (your realistic jump range with cargo/fuel)
laden_jump_range = 35.0
# Optional: Maximum jump range when empty/optimized
# max_jump_range = 60.0

# Cache timeout in seconds (default: 300 = 5 minutes)
cache_timeout_seconds = 300

# Enable debug logging (default: false)
debug_mode = false

# Distance thresholds for route suggestions
neutron_highway_threshold_ly = 500.0
white_dwarf_threshold_ly = 150.0

# Result format string
# Available placeholders: {jumps}, {system}, {distance}, {route}
result_format = "🚀 {jumps} jumps to {system} ({distance:.1}ly) via {route}"

# Show additional estimates
show_fuel_estimates = false
show_time_estimates = false
"#;

    // Create config directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&config_path, sample_config)?;
    info!("Sample configuration created at: {config_path:?}");

    Ok(())
}

/// Validate configuration
pub fn validate_config(config: &Config) -> Result<()> {
    if config.cmdr_name.is_empty() {
        return Err(anyhow!("CMDR name is required but not configured"));
    }

    if config.ship.laden_jump_range <= 0.0 {
        return Err(anyhow!("Ship laden jump range must be greater than 0"));
    }

    if config.cache_timeout_seconds == 0 {
        return Err(anyhow!("Cache timeout must be greater than 0"));
    }

    if config.neutron_highway_threshold_ly < 0.0 {
        return Err(anyhow!("Neutron highway threshold must be non-negative"));
    }

    if config.white_dwarf_threshold_ly < 0.0 {
        return Err(anyhow!("White dwarf threshold must be non-negative"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.cache_timeout_seconds, 300);
        assert!(!config.debug_mode);
        assert_eq!(config.neutron_highway_threshold_ly, 500.0);
    }

    #[test]
    fn test_config_validation() {
        let config = Config {
            cmdr_name: "TestCMDR".to_string(),
            ship: ShipConfig {
                name: "Test Ship".to_string(),
                laden_jump_range: 30.0,
                max_jump_range: Some(50.0),
            },
            ..Default::default()
        };

        assert!(validate_config(&config).is_ok());

        let config = Config {
            cmdr_name: String::new(),
            ..Default::default()
        };
        assert!(validate_config(&config).is_err());

        let config = Config {
            cmdr_name: "TestCMDR".to_string(),
            ship: ShipConfig {
                name: "Test Ship".to_string(),
                laden_jump_range: 0.0, // Invalid jump range
                max_jump_range: None,
            },
            ..Default::default()
        };
        assert!(validate_config(&config).is_err());

        let config = Config {
            cmdr_name: "TestCMDR".to_string(),
            cache_timeout_seconds: 0,
            ..Default::default()
        };
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();

        assert_eq!(
            config.cache_timeout_seconds,
            deserialized.cache_timeout_seconds
        );
        assert_eq!(config.debug_mode, deserialized.debug_mode);
    }
}
