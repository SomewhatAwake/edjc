/*!
# Elite Dangerous Jump Calculator (EDJC) - HexChat Plugin

A HexChat plugin that calculates the number of jumps required to reach a specified system
in Elite: Dangerous, taking into account the player's current location, ship jump range,
and jump multipliers from neutron stars and white dwarfs.

## Features

- Automatically detects RATSIGNAL messages in chat
- Fetches CMDR location and ship data from Inara API
- Calculates optimal jump routes considering:
  - Minimum jump range of the selected ship
  - Neutron star boosts (4x multiplier)
  - White dwarf boosts (1.5x multiplier)
- Displays results as HexChat notices

## Usage

The plugin automatically triggers when it detects a RATSIGNAL message from MechaSqueak[BOT]
containing system information.

Example trigger:
```
RATSIGNAL Case #3 PC ODY – CMDR Whit3Arrow – System: "CRUCIS SECTOR IW-N A6-5" (Brown dwarf 51 LY from Fuelum) – Language: English (United States) (en-US) (ODY_SIGNAL)
```
*/

pub mod config;
mod hexchat;
pub mod inara;
pub mod jump_calculator;
pub mod types;

use anyhow::Result;
use libc::c_char;
use log::{error, info, warn};
use regex::Regex;
use std::ffi::CString;
use std::sync::OnceLock;

use crate::inara::InaraClient;
use crate::jump_calculator::JumpCalculator;
use crate::types::JumpResult;

/// Global plugin instance
static PLUGIN: OnceLock<EdJumpCalculator> = OnceLock::new();

/// Main plugin structure
#[derive(Debug)]
pub struct EdJumpCalculator {
    inara_client: InaraClient,
    jump_calculator: JumpCalculator,
    ratsignal_regex: Regex,
    cmdr_name: String,
}

impl EdJumpCalculator {
    /// Initialize the plugin
    pub fn new() -> Result<Self> {
        let config = config::load_config()?;

        Ok(Self {
            inara_client: InaraClient::new(config.inara_api_key.clone())?,
            jump_calculator: JumpCalculator::new(),
            ratsignal_regex: Regex::new(
                r#"RATSIGNAL.*?Case\s*#(\d+).*?CMDR\s+([^–]+).*?System:\s*"([^"]+)".*?Language:\s*([^(]*)"#,
            )?,
            cmdr_name: config.cmdr_name,
        })
    }

    /// Validate plugin configuration
    pub fn validate_config(&self) -> Result<()> {
        if self.cmdr_name.is_empty() {
            return Err(anyhow::anyhow!(
                "CMDR name is not configured. Please set 'cmdr_name' in edjc.toml"
            ));
        }

        // Test Inara API connection
        match self.inara_client.test_connection(&self.cmdr_name) {
            Ok(true) => {
                info!(
                    "Inara API connection successful for CMDR: {}",
                    self.cmdr_name
                );
                Ok(())
            }
            Ok(false) => Err(anyhow::anyhow!(
                "CMDR '{}' not found in Inara database",
                self.cmdr_name
            )),
            Err(e) => Err(anyhow::anyhow!("Inara API connection failed: {}", e)),
        }
    }

    /// Process a chat message and check for RATSIGNAL
    pub fn process_message(&self, sender: &str, message: &str) -> Result<Option<String>> {
        // Only process messages from MechaSqueak[BOT]
        if sender != "MechaSqueak[BOT]" {
            return Ok(None);
        }

        if let Some(captures) = self.ratsignal_regex.captures(message) {
            let case_number = captures.get(1).map(|m| m.as_str()).unwrap_or("Unknown");
            let distressed_cmdr = captures
                .get(2)
                .map(|m| m.as_str().trim())
                .unwrap_or("Unknown");
            let target_system = captures.get(3).unwrap().as_str();
            let language = captures
                .get(4)
                .map(|m| m.as_str().trim())
                .unwrap_or("Unknown");

            info!(
                "RATSIGNAL detected - Case #{case_number}, CMDR: {distressed_cmdr}, System: {target_system}, Language: {language}"
            );

            match self.calculate_jumps(target_system) {
                Ok(result) => {
                    let response = format!(
                        "🚀 Case #{}: {} jumps to {} ({:.1}ly) via {} route (for CMDR {})",
                        case_number,
                        result.jumps,
                        target_system,
                        result.total_distance,
                        result.route_type,
                        self.cmdr_name
                    );
                    Ok(Some(response))
                }
                Err(e) => {
                    error!("Failed to calculate jumps for case #{case_number}: {e}");
                    Ok(Some(format!(
                        "❌ Case #{case_number}: Jump calculation failed for {target_system} - {e}"
                    )))
                }
            }
        } else {
            // Check if it's a RATSIGNAL but didn't match our pattern
            if message.contains("RATSIGNAL") {
                warn!("RATSIGNAL detected but couldn't parse: {message}");
                Ok(Some(
                    "⚠️ RATSIGNAL detected but couldn't parse system information".to_string(),
                ))
            } else {
                Ok(None)
            }
        }
    }

    /// Calculate jumps to target system
    fn calculate_jumps(&self, target_system: &str) -> Result<JumpResult> {
        // Get current CMDR location and ship info using the configured CMDR name
        let cmdr_info = self.inara_client.get_cmdr_location(&self.cmdr_name)?;
        let ship_info = self.inara_client.get_ship_info(&self.cmdr_name)?;

        // Get system coordinates
        let current_coords = self
            .inara_client
            .get_system_coordinates(&cmdr_info.current_system)?;
        let target_coords = self.inara_client.get_system_coordinates(target_system)?;

        // Calculate jump route
        self.jump_calculator.calculate_route(
            &current_coords,
            &target_coords,
            ship_info.min_jump_range,
        )
    }
}

// HexChat plugin export functions
#[no_mangle]
pub extern "C" fn hexchat_plugin_init(
    _plugin_handle: *mut hexchat::HexChatPlugin,
    plugin_name: *mut *const c_char,
    plugin_desc: *mut *const c_char,
    plugin_version: *mut *const c_char,
    _arg: *const c_char,
) -> i32 {
    // Initialize logging
    if let Err(e) = env_logger::try_init() {
        eprintln!("Failed to initialize logger: {e}");
    }

    // Set plugin info
    unsafe {
        *plugin_name = CString::new("Elite Dangerous Jump Calculator")
            .unwrap()
            .into_raw();
        *plugin_desc = CString::new("Calculates jumps to RATSIGNAL systems")
            .unwrap()
            .into_raw();
        *plugin_version = CString::new("0.1.0").unwrap().into_raw();
    }

    // Initialize plugin
    match EdJumpCalculator::new() {
        Ok(plugin) => {
            // Validate configuration
            if let Err(e) = plugin.validate_config() {
                error!("Configuration validation failed: {e}");
                return 0; // Failure
            }

            PLUGIN.set(plugin).unwrap();

            info!("EDJC plugin initialized successfully");
            info!("Monitoring for RATSIGNAL messages from MechaSqueak[BOT]");

            1 // Success
        }
        Err(e) => {
            error!("Failed to initialize EDJC plugin: {e}");
            0 // Failure
        }
    }
}

#[no_mangle]
pub extern "C" fn hexchat_plugin_deinit() -> i32 {
    info!("EDJC plugin deinitialized");
    1
}

/// Callback for chat messages - placeholder for future implementation
#[allow(dead_code)]
extern "C" fn message_callback(
    _word: *const *const c_char,
    _word_eol: *const *const c_char,
    _user_data: *mut libc::c_void,
) -> i32 {
    // This would be implemented when we have proper HexChat API access
    // For now, just return HEXCHAT_EAT_NONE
    hexchat::HEXCHAT_EAT_NONE
}
