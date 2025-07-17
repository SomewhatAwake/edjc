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
use std::ffi::{CString, CStr};
use std::sync::OnceLock;
use std::ptr;

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

/// Initialize HexChat integration with proper API hooks
unsafe fn init_hexchat_integration(
    plugin_handle: *mut hexchat::HexChatPlugin,
    _arg: *const c_char,
) -> Result<()> {
    // For now, we'll use a simplified approach since HexChat API function pointers
    // are complex to parse. In a real implementation, you'd parse the function
    // pointers from the arg parameter.
    
    // Store plugin handle for later use
    hexchat::store_plugin_handle(plugin_handle);
    
    // Hook into channel messages to detect RATSIGNAL
    let hook_name = CString::new("Channel Message")?;
    let _hook = hexchat::hexchat_hook_print(
        hook_name.as_ptr(),
        Some(channel_message_callback),
        ptr::null_mut(),
    );
    
    Ok(())
}

/// Callback for channel messages - detects RATSIGNAL messages
extern "C" fn channel_message_callback(
    word: *const *const c_char,
    _word_eol: *const *const c_char,
    _user_data: *mut libc::c_void,
) -> i32 {
    if word.is_null() {
        return hexchat::HEXCHAT_EAT_NONE;
    }

    unsafe {
        // word[0] = nick, word[1] = message
        let message_ptr = *word.add(1);
        if message_ptr.is_null() {
            return hexchat::HEXCHAT_EAT_NONE;
        }

        let message = match CStr::from_ptr(message_ptr).to_str() {
            Ok(s) => s,
            Err(_) => return hexchat::HEXCHAT_EAT_NONE,
        };

        // Check if it's from MechaSqueak[BOT] and contains RATSIGNAL
        let nick_ptr = *word;
        if !nick_ptr.is_null() {
            if let Ok(nick) = CStr::from_ptr(nick_ptr).to_str() {
                if nick == "MechaSqueak[BOT]" && message.contains("RATSIGNAL") {
                    // Process the RATSIGNAL message
                    if let Some(plugin) = PLUGIN.get() {
                        match plugin.process_message(nick, message) {
                            Ok(Some(response)) => {
                                // Display the response in HexChat
                                let formatted_response = format!("[EDJC] {response}");
                                hexchat::hexchat_print(
                                    CString::new(formatted_response).unwrap().as_ptr()
                                );
                            }
                            Ok(None) => {
                                // RATSIGNAL detected but couldn't parse or not from expected sender
                                info!("Message ignored or couldn't parse: {message}");
                            }
                            Err(e) => {
                                error!("Error processing RATSIGNAL: {e}");
                                let error_msg = format!("[EDJC] Error: {e}");
                                hexchat::hexchat_print(
                                    CString::new(error_msg).unwrap().as_ptr()
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    hexchat::HEXCHAT_EAT_NONE // Don't consume the message
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
                hexchat::hexchat_print(
                    CString::new(error_msg).unwrap().as_ptr(),
                );
            }

            // Try to set up HexChat API if we have the function pointers
            if !arg.is_null() {
                if let Err(e) = init_hexchat_integration(plugin_handle, arg) {
                    warn!("HexChat integration limited: {e}");
                } else {
                    info!("HexChat integration initialized");
                }
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
