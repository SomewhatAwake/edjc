/*!
# Elite Dangerous Jump Calculator (EDJC) - HexChat Plugin

A HexChat plugin that calculates the number of jumps required to reach a specified system
in Elite: Dangerous, using EDSM system coordinates and user-configured ship jump range.

## Features

- Automatically detects RATSIGNAL messages in chat
- Fetches system coordinates from EDSM (Elite Dangerous Star Map)
- Calculates optimal jump routes considering:
  - User-configured ship laden jump range
  - Neutron star boosts (4x multiplier)
  - White dwarf boosts (1.5x multiplier)
- Displays results as HexChat notices

## Configuration

Users must configure their ship's laden jump range in the `edjc.toml` file.
No API keys or external authentication required - uses free EDSM data.

## Usage

The plugin automatically triggers when it detects a RATSIGNAL message from MechaSqueak[BOT]
containing system information. Users can also test the plugin manually using `/route <system>`.

Example trigger:
```text
RATSIGNAL Case #3 PC ODY - CMDR Whit3Arrow - System: "CRUCIS SECTOR IW-N A6-5" (Brown dwarf 51 LY from Fuelum) - Language: English (United States) (en-US) (ODY_SIGNAL)
```

Example manual test:
```text
/route Colonia
```
*/

pub mod config;
pub mod edsm;
mod hexchat;
pub mod jump_calculator;
pub mod types;

use anyhow::Result;
use libc::c_char;
use log::{error, info, warn};
use regex::Regex;
use std::ffi::CString;
use std::sync::OnceLock;

use crate::edsm::EdsmClient;
use crate::jump_calculator::JumpCalculator;
use crate::types::JumpResult;

/// Global plugin instance
static PLUGIN: OnceLock<EdJumpCalculator> = OnceLock::new();

/// Main plugin structure
#[derive(Debug)]
pub struct EdJumpCalculator {
    edsm_client: EdsmClient,
    jump_calculator: JumpCalculator,
    ratsignal_regex: Regex,
    cmdr_name: String,
    edsm_api_key: Option<String>,
    ship_jump_range: f64,
}

impl EdJumpCalculator {
    /// Initialize the plugin
    pub fn new() -> Result<Self> {
        let config = config::load_config()?;

        Ok(Self {
            edsm_client: EdsmClient::new()?,
            jump_calculator: JumpCalculator::new(),
            ratsignal_regex: Regex::new(
                r#"RATSIGNAL.*?Case\s*#(\d+).*?CMDR\s+([^–]+).*?System:\s*"([^"]+)".*?Language:\s*([^(]*)"#,
            )?,
            cmdr_name: config.cmdr_name,
            edsm_api_key: config.edsm_api_key,
            ship_jump_range: config.ship.laden_jump_range,
        })
    }

    /// Validate plugin configuration
    pub fn validate_config(&self) -> Result<()> {
        if self.cmdr_name.is_empty() {
            return Err(anyhow::anyhow!(
                "CMDR name is not configured. Please set 'cmdr_name' in edjc.toml"
            ));
        }

        if self.ship_jump_range <= 0.0 {
            return Err(anyhow::anyhow!(
                "Ship laden jump range must be greater than 0. Please set 'ship.laden_jump_range' in edjc.toml"
            ));
        }

        // Test EDSM API connection
        match self.edsm_client.test_connection() {
            Ok(true) => {
                info!("EDSM API connection successful");
                Ok(())
            }
            Ok(false) => Err(anyhow::anyhow!("EDSM API connection test failed")),
            Err(e) => Err(anyhow::anyhow!("EDSM API connection failed: {}", e)),
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

            match self.calculate_jumps_with_origin(target_system) {
                Ok((result, origin_system)) => {
                    let response = format!(
                        "🚀 Case #{}: {} jumps to {} ({:.1}ly) via {} route (from {} with {:.1}ly range)",
                        case_number,
                        result.jumps,
                        target_system,
                        result.total_distance,
                        result.route_type,
                        origin_system,
                        self.ship_jump_range
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

    /// Handle the /route command for testing
    pub fn handle_route_command(&self, target_system: &str) -> String {
        if target_system.trim().is_empty() {
            return "Usage: /route <system_name>".to_string();
        }

        let system_name = target_system.trim();

        match self.calculate_jumps_with_origin(system_name) {
            Ok((result, origin_system)) => {
                format!(
                    "🚀 Route to {}: {} jumps ({:.1} LY) via {} route (from {} with {:.1} LY range)",
                    system_name,
                    result.jumps,
                    result.total_distance,
                    result.route_type,
                    origin_system,
                    self.ship_jump_range
                )
            }
            Err(e) => {
                error!("Failed to calculate route to {system_name}: {e}");
                format!("❌ Route calculation failed for {system_name}: {e}")
            }
        }
    }

    /// Calculate jumps to target system and return both result and origin system
    fn calculate_jumps_with_origin(&self, target_system: &str) -> Result<(JumpResult, String)> {
        // Try to get commander's current location from EDSM
        let current_system = match self
            .edsm_client
            .get_commander_location(&self.cmdr_name, self.edsm_api_key.as_deref())
        {
            Ok(system) => {
                info!(
                    "Using CMDR {}'s current location: {}",
                    self.cmdr_name, system
                );
                system
            }
            Err(e) => {
                warn!("Could not get CMDR location from EDSM: {e}. Using Sol as fallback.");
                "Sol".to_string()
            }
        };

        // Get system coordinates from EDSM
        let current_coords = self.edsm_client.get_system_coordinates(&current_system)?;
        let target_coords = self.edsm_client.get_system_coordinates(target_system)?;

        // Calculate jump route using the configured ship jump range
        let result = self.jump_calculator.calculate_route(
            &current_coords,
            &target_coords,
            self.ship_jump_range,
        )?;

        Ok((result, current_system))
    }
}

/// Initialize HexChat integration - basic version without command hooks
unsafe fn init_hexchat_integration(
    plugin_handle: *mut hexchat::HexChatPlugin,
    arg: *const c_char,
) -> Result<()> {
    // Store plugin handle for later use
    hexchat::store_plugin_handle(plugin_handle);

    // Initialize HexChat API
    if !hexchat::init_hexchat_api_from_arg(plugin_handle, arg) {
        warn!("Could not initialize HexChat API from arg parameter");
    }

    // Register the /route command - temporarily disabled for stability
    let route_cmd = CString::new("route")?;
    let _route_hook = hexchat::hexchat_hook_command(
        route_cmd.as_ptr(),
        Some(route_command_callback),
        std::ptr::null_mut(),
    );

    // Print startup messages
    let startup_msg =
        CString::new("[EDJC] Plugin loaded successfully! RATSIGNAL detection is active.")?;
    hexchat::hexchat_print(startup_msg.as_ptr());

    let help_msg = CString::new("[EDJC] Note: /route command temporarily disabled for stability. Use standalone calculator for testing.")?;
    hexchat::hexchat_print(help_msg.as_ptr());

    Ok(())
}

// HexChat plugin export functions

/// Initialize the HexChat plugin.
///
/// This function is called by HexChat when the plugin is loaded.
///
/// # Safety
///
/// This function is unsafe because it:
/// - Dereferences raw pointers (`plugin_name`, `plugin_desc`, `plugin_version`) without null checks
/// - Assumes the pointers point to valid memory locations that can be written to
/// - Converts Rust `CString`s to raw pointers and transfers ownership to HexChat
/// - Calls other unsafe functions that interact with HexChat's C API
///
/// The caller (HexChat) must ensure that:
/// - All pointer parameters point to valid, writable memory
/// - The plugin handle is valid for the lifetime of the plugin
/// - The arg parameter, if not null, points to valid C string data
#[no_mangle]
pub unsafe extern "C" fn hexchat_plugin_init(
    plugin_handle: *mut hexchat::HexChatPlugin,
    plugin_name: *mut *const c_char,
    plugin_desc: *mut *const c_char,
    plugin_version: *mut *const c_char,
    arg: *const c_char,
) -> i32 {
    // Initialize logging
    if let Err(e) = env_logger::try_init() {
        eprintln!("Failed to initialize logger: {e}");
    }

    // Set plugin info
    *plugin_name = CString::new("Elite Dangerous Jump Calculator")
        .unwrap()
        .into_raw();
    *plugin_desc = CString::new("Calculates jumps to RATSIGNAL systems")
        .unwrap()
        .into_raw();
    *plugin_version = CString::new("0.1.0").unwrap().into_raw();

    // Initialize plugin
    match EdJumpCalculator::new() {
        Ok(plugin) => {
            // Validate configuration
            if let Err(e) = plugin.validate_config() {
                error!("Configuration validation failed: {e}");

                // Still try to initialize but warn user
                let error_msg = format!("[EDJC] Configuration error: {e}");
                hexchat::hexchat_print(CString::new(error_msg).unwrap().as_ptr());
            }

            // Set up HexChat API integration
            if let Err(e) = init_hexchat_integration(plugin_handle, arg) {
                warn!("HexChat integration limited: {e}");
            } else {
                info!("HexChat integration initialized");
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

/// Deinitialize the HexChat plugin.
///
/// This function is called by HexChat when the plugin is being unloaded.
/// Returns 1 on success, 0 on failure.
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

/// Callback for the /route command
extern "C" fn route_command_callback(
    word: *const *const c_char,
    _word_eol: *const *const c_char,
    _user_data: *mut libc::c_void,
) -> i32 {
    if let Some(plugin) = PLUGIN.get() {
        unsafe {
            // Parse the command arguments
            let target_system = if !word.is_null() {
                // word[0] is the command name ("/route"), word[1] is the first argument
                let word1_ptr = *word.offset(1);
                if !word1_ptr.is_null() {
                    hexchat::c_str_to_string(word1_ptr)
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            // Handle the route command
            let response = plugin.handle_route_command(&target_system);

            // Send the response to HexChat
            let response_cstr = std::ffi::CString::new(response).unwrap();
            hexchat::hexchat_print(response_cstr.as_ptr());
        }
    } else {
        let error_msg = std::ffi::CString::new("❌ Plugin not initialized").unwrap();
        hexchat::hexchat_print(error_msg.as_ptr());
    }

    hexchat::HEXCHAT_EAT_ALL // Consume the command so HexChat doesn't show "unknown command"
}
