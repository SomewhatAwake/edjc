use edjc::edsm::EdsmClient;
use edjc::jump_calculator::JumpCalculator;
use std::io::{self, Write};

fn main() -> anyhow::Result<()> {
    println!("Testing EDJC functionality with EDSM...");

    // Create EDSM client
    let edsm_client = match EdsmClient::new() {
        Ok(client) => client,
        Err(e) => {
            println!("Failed to create EDSM client: {e}");
            return Ok(());
        }
    };

    // Test API connection
    print!("Testing EDSM API connection... ");
    io::stdout().flush()?;

    match edsm_client.test_connection() {
        Ok(true) => println!("✓ Connection successful"),
        Ok(false) => println!("✗ Connection failed - unexpected data"),
        Err(e) => println!("✗ Connection failed: {e}"),
    }

    // Test getting system coordinates
    let test_systems = vec!["Sol", "Sagittarius A*", "Colonia", "Beagle Point"];

    println!("\nTesting system coordinate lookup:");
    for system in &test_systems {
        print!("  {system}: ");
        io::stdout().flush()?;

        match edsm_client.get_system_coordinates(system) {
            Ok(coords) => {
                println!("✓ ({:.2}, {:.2}, {:.2})", coords.x, coords.y, coords.z);
                if coords.has_neutron_star {
                    println!("    Has neutron star");
                }
                if coords.has_white_dwarf {
                    println!("    Has white dwarf");
                }
            }
            Err(e) => println!("✗ Failed: {e}"),
        }
    }

    // Test jump calculation with real coordinates
    println!("\nTesting jump calculation with real coordinates:");
    let jump_calc = JumpCalculator::new();

    // Test with different jump ranges to see the effect
    let jump_ranges = vec![20.0, 35.0, 50.0, 70.0];

    for jump_range in &jump_ranges {
        println!("\n--- Testing with {jump_range:.1} LY jump range ---");

        let test_cases = vec![
            ("Sol", "Alpha Centauri"), // Very short distance
            ("Sol", "Colonia"),        // Long distance, good for neutron highway
            ("Sol", "Sagittarius A*"), // Very long distance
            ("Deciat", "Maia"),        // Medium distance
        ];

        for (from, to) in &test_cases {
            print!("  {from} -> {to}: ");
            io::stdout().flush()?;

            // Get real coordinates from EDSM
            match (
                edsm_client.get_system_coordinates(from),
                edsm_client.get_system_coordinates(to),
            ) {
                (Ok(from_coords), Ok(to_coords)) => {
                    let direct_distance = ((to_coords.x - from_coords.x).powi(2)
                        + (to_coords.y - from_coords.y).powi(2)
                        + (to_coords.z - from_coords.z).powi(2))
                    .sqrt();

                    match jump_calc.calculate_route(&from_coords, &to_coords, *jump_range) {
                        Ok(route) => {
                            println!(
                                "{} jumps ({:.1} LY direct, {:.1} LY route) via {}",
                                route.jumps,
                                direct_distance,
                                route.total_distance,
                                route.route_type
                            );

                            // Show star types if detected
                            if from_coords.has_neutron_star {
                                println!("    📡 {from} has neutron star");
                            }
                            if from_coords.has_white_dwarf {
                                println!("    ⚪ {from} has white dwarf");
                            }
                            if to_coords.has_neutron_star {
                                println!("    📡 {to} has neutron star");
                            }
                            if to_coords.has_white_dwarf {
                                println!("    ⚪ {to} has white dwarf");
                            }
                        }
                        Err(e) => println!("Route calculation failed: {e}"),
                    }
                }
                (Err(e), _) => println!("Failed to get coordinates for {from}: {e}"),
                (_, Err(e)) => println!("Failed to get coordinates for {to}: {e}"),
            }
        }
    }

    // Test specific neutron star and white dwarf systems
    println!("\n--- Testing known neutron star and white dwarf systems ---");
    let special_systems = vec![
        "Jackson's Lighthouse", // Known neutron star
        "Swoals IL-Y e0",       // Known neutron star
        "Sirius",               // Known white dwarf companion
        "Procyon",              // Known white dwarf companion
    ];

    for system in &special_systems {
        print!("  Checking {system}: ");
        io::stdout().flush()?;

        match edsm_client.get_system_coordinates(system) {
            Ok(coords) => {
                print!("({:.1}, {:.1}, {:.1})", coords.x, coords.y, coords.z);
                if coords.has_neutron_star {
                    println!(" 📡 Neutron star detected!");
                } else if coords.has_white_dwarf {
                    println!(" ⚪ White dwarf detected!");
                } else {
                    println!(" ⭐ Regular star");
                }
            }
            Err(e) => println!("Failed: {e}"),
        }
    }

    // Test direct distance calculation
    println!("\nTesting direct distance calculations:");
    let distance_cases = vec![
        ("Sol", "Alpha Centauri"),
        ("Sol", "Sagittarius A*"),
        ("Sol", "Colonia"),
    ];

    for (from, to) in distance_cases {
        print!("  {from} -> {to}: ");
        io::stdout().flush()?;

        match edsm_client.calculate_distance(from, to) {
            Ok(distance) => println!("{distance:.2} LY"),
            Err(e) => println!("Failed: {e}"),
        }
    }

    println!("\nTest complete!");
    Ok(())
}
