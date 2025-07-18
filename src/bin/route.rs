/*!
Standalone route calculator for testing EDJC functionality.

This binary allows users to test the jump calculation functionality
without loading the HexChat plugin.
*/

use edjc::config;
use edjc::edsm::EdsmClient;
use edjc::jump_calculator::JumpCalculator;
use std::env;
use std::io::{self, Write};

fn main() -> anyhow::Result<()> {
    println!("EDJC Route Calculator - Standalone Test");
    println!("=======================================");

    // Load configuration
    let config = match config::load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Warning: Could not load config: {e}");
            eprintln!("Using default ship jump range of 35.0 LY");
            println!();

            // Create a default config
            config::Config {
                cmdr_name: "Test CMDR".to_string(),
                edsm_api_key: None,
                ship: config::ShipConfig {
                    name: "Test Ship".to_string(),
                    laden_jump_range: 35.0,
                    max_jump_range: None,
                },
                cache_timeout_seconds: 300,
                debug_mode: false,
                neutron_highway_threshold_ly: 500.0,
                white_dwarf_threshold_ly: 150.0,
                result_format: "🚀 {jumps} jumps to {system} ({distance:.1}ly) via {route}"
                    .to_string(),
                show_fuel_estimates: false,
                show_time_estimates: false,
            }
        }
    };

    println!("Configuration:");
    println!("  CMDR: {}", config.cmdr_name);
    println!("  Ship jump range: {:.1} LY", config.ship.laden_jump_range);
    println!();

    // Create clients
    let edsm_client = EdsmClient::new()?;
    let jump_calculator = JumpCalculator::new();

    // Test EDSM connection
    print!("Testing EDSM connection... ");
    io::stdout().flush()?;

    match edsm_client.test_connection() {
        Ok(true) => println!("✓ Connected"),
        Ok(false) => {
            println!("✗ Connection test failed");
            return Ok(());
        }
        Err(e) => {
            println!("✗ Connection failed: {e}");
            return Ok(());
        }
    }

    // Get command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <target_system> [current_system]", args[0]);
        println!();
        println!("If current_system is not provided, your CMDR's current location will be");
        println!("retrieved from EDSM automatically (if available).");
        println!();
        println!("Examples:");
        println!(
            "  {} Colonia                           # Route from your current location",
            args[0]
        );
        println!(
            "  {} Colonia Deciat                    # Route from Deciat to Colonia",
            args[0]
        );
        println!(
            "  {} \"Sagittarius A*\"                  # Route from current location",
            args[0]
        );
        println!(
            "  {} \"Sagittarius A*\" \"Shinrarta Dezhra\" # Route from Shinrarta Dezhra",
            args[0]
        );
        return Ok(());
    }

    let target_system = &args[1];
    let current_system = if args.len() >= 3 {
        args[2].clone()
    } else {
        // Try to get commander's current location from EDSM
        println!(
            "Getting {}'s current location from EDSM...",
            config.cmdr_name
        );
        match edsm_client.get_commander_location(&config.cmdr_name) {
            Ok(system) => {
                println!("✓ Found {} in {}", config.cmdr_name, system);
                system
            }
            Err(e) => {
                println!("⚠️ Could not get commander location: {e}");
                if config.edsm_api_key.is_none() {
                    println!("   Note: No EDSM API key configured. Add 'edsm_api_key = \"your_key\"' to edjc.toml");
                    println!(
                        "   to access private location data, or enable public profile on EDSM."
                    );
                }
                println!("   Using Sol as starting point. You can specify current system as: {} {} <current_system>", args[0], target_system);
                "Sol".to_string()
            }
        }
    };

    println!("Calculating route from {current_system} to {target_system}...");
    println!();

    // Get system coordinates with better error handling
    println!("Looking up {current_system} coordinates...");
    let current_coords = match edsm_client.get_system_coordinates(&current_system) {
        Ok(coords) => {
            println!(
                "✓ {} found at ({:.1}, {:.1}, {:.1})",
                current_system, coords.x, coords.y, coords.z
            );
            coords
        }
        Err(e) => {
            println!("❌ Failed to get {current_system} coordinates: {e}");
            return Ok(());
        }
    };

    println!("Looking up {target_system} coordinates...");
    let target_coords = match edsm_client.get_system_coordinates(target_system) {
        Ok(coords) => {
            println!(
                "✓ {} found at ({:.1}, {:.1}, {:.1})",
                target_system, coords.x, coords.y, coords.z
            );
            coords
        }
        Err(e) => {
            println!("❌ Failed to get {target_system} coordinates: {e}");
            println!("   This could mean:");
            println!("   - System name is misspelled");
            println!("   - System doesn't exist in EDSM database");
            println!("   - Network connection issue");
            return Ok(());
        }
    };

    // Calculate direct distance
    let direct_distance = ((target_coords.x - current_coords.x).powi(2)
        + (target_coords.y - current_coords.y).powi(2)
        + (target_coords.z - current_coords.z).powi(2))
    .sqrt();

    println!("System Information:");
    println!(
        "  {}: ({:.1}, {:.1}, {:.1})",
        current_system, current_coords.x, current_coords.y, current_coords.z
    );
    println!(
        "  {}: ({:.1}, {:.1}, {:.1})",
        target_system, target_coords.x, target_coords.y, target_coords.z
    );
    println!("  Direct distance: {direct_distance:.1} LY");

    if current_coords.has_neutron_star {
        println!("  📡 {current_system} has a neutron star!");
    }
    if current_coords.has_white_dwarf {
        println!("  ⚪ {current_system} has a white dwarf!");
    }
    if target_coords.has_neutron_star {
        println!("  📡 {target_system} has a neutron star!");
    }
    if target_coords.has_white_dwarf {
        println!("  ⚪ {target_system} has a white dwarf!");
    }
    println!();

    // Calculate route
    match jump_calculator.calculate_route(
        &current_coords,
        &target_coords,
        config.ship.laden_jump_range,
    ) {
        Ok(result) => {
            println!("Route Calculation:");
            println!("  🚀 {} jumps required", result.jumps);
            println!("  📏 {:.1} LY total route distance", result.total_distance);
            println!("  🛣️ Route type: {}", result.route_type);
            println!(
                "  ⛽ Ship jump range: {:.1} LY",
                config.ship.laden_jump_range
            );

            if result.route_type.contains("neutron") {
                println!("  💫 Using neutron highway for 4x boost!");
            } else if result.route_type.contains("white dwarf") {
                println!("  ⭐ Using white dwarf assistance for 1.5x boost!");
            }
        }
        Err(e) => {
            println!("❌ Route calculation failed: {e}");
        }
    }

    Ok(())
}
