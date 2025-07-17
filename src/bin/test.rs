use edjc::config::load_config;
use edjc::inara::InaraClient;
use edjc::jump_calculator::JumpCalculator;
use std::io::{self, Write};

fn main() -> anyhow::Result<()> {
    // Try to load config
    let config = match load_config() {
        Ok(config) => config,
        Err(e) => {
            println!("Failed to load config: {}", e);
            println!("Please ensure edjc.toml exists with your Inara API key and CMDR name.");
            return Ok(());
        }
    };

    println!("Testing EDJC functionality...");
    println!("CMDR: {}", config.cmdr_name);

    // Create Inara client
    let inara_client = match InaraClient::new(config.inara_api_key.clone()) {
        Ok(client) => client,
        Err(e) => {
            println!("Failed to create Inara client: {}", e);
            return Ok(());
        }
    };

    // Test API connection
    print!("Testing Inara API connection... ");
    io::stdout().flush()?;

    match inara_client.test_connection(&config.cmdr_name) {
        Ok(true) => println!("✓ Connection successful, CMDR found"),
        Ok(false) => println!("✗ CMDR '{}' not found in Inara", config.cmdr_name),
        Err(e) => println!("✗ Connection failed: {}", e),
    }

    // Test getting CMDR location
    print!("Fetching CMDR location... ");
    io::stdout().flush()?;

    match inara_client.get_cmdr_location(&config.cmdr_name) {
        Ok(location) => {
            println!("✓ Current system: {}", location.current_system);
            println!("  CMDR: {}", location.cmdr_name);
            if let Some(station) = &location.current_station {
                println!("  Station: {}", station);
            }
        }
        Err(e) => println!("✗ Failed to get location: {}", e),
    }

    // Test getting ship info
    print!("Fetching ship information... ");
    io::stdout().flush()?;

    match inara_client.get_ship_info(&config.cmdr_name) {
        Ok(ship) => {
            println!("✓ Ship: {}", ship.ship_type);
            if let Some(name) = &ship.ship_name {
                println!("  Name: {}", name);
            }
            println!(
                "  Jump range: {:.2} - {:.2} LY",
                ship.min_jump_range, ship.max_jump_range
            );
        }
        Err(e) => println!("✗ Failed to get ship info: {}", e),
    }

    // Test jump calculation
    println!("\nTesting jump calculation:");
    let jump_calc = JumpCalculator::new();

    // Test with some example systems
    let test_cases = vec![
        ("Sol", "Sagittarius A*"),
        ("Deciat", "Colonia"),
        ("Shinrarta Dezhra", "Beagle Point"),
    ];

    for (from, to) in test_cases {
        print!("  {} -> {}: ", from, to);
        io::stdout().flush()?;

        // For this test, we'll use placeholder coordinates
        // In real use, these would come from the Inara API
        let from_coords = edjc::types::SystemCoordinates {
            name: "Sol".to_string(),
            x: 0.0,
            y: 0.0,
            z: 0.0,
            has_neutron_star: false,
            has_white_dwarf: false,
        };
        let to_coords = edjc::types::SystemCoordinates {
            name: "Test System".to_string(),
            x: 100.0,
            y: 50.0,
            z: 25.0,
            has_neutron_star: false,
            has_white_dwarf: false,
        };

        match jump_calc.calculate_route(&from_coords, &to_coords, 50.0) {
            Ok(route) => {
                println!("{} jumps ({:.2} LY)", route.jumps, route.total_distance);
            }
            Err(e) => println!("Failed: {}", e),
        }
    }

    println!("\nTest complete!");
    Ok(())
}
