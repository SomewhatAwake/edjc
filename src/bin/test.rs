use edjc::edsm::EdsmClient;
use edjc::jump_calculator::JumpCalculator;
use std::io::{self, Write};

fn main() -> anyhow::Result<()> {
    // Initialize logging to see debug output
    if let Err(e) =
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
            .try_init()
    {
        eprintln!("Failed to initialize logger: {e}");
    }

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
                println!(
                    "✓ ({:.2}, {:.2}, {:.2})", 
                    coords.x, coords.y, coords.z
                );
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

    // Test with real systems
    let test_cases = vec![
        ("Sol", "Sagittarius A*"),
        ("Sol", "Colonia"),
        ("Sol", "Beagle Point"),
        ("Deciat", "Colonia"),
    ];

    for (from, to) in test_cases {
        print!("  {from} -> {to}: ");
        io::stdout().flush()?;

        // Get real coordinates from EDSM
        match (edsm_client.get_system_coordinates(from), edsm_client.get_system_coordinates(to)) {
            (Ok(from_coords), Ok(to_coords)) => {
                match jump_calc.calculate_route(&from_coords, &to_coords, 50.0) {
                    Ok(route) => {
                        println!("{} jumps ({:.2} LY total distance)", route.jumps, route.total_distance);
                    }
                    Err(e) => println!("Route calculation failed: {e}"),
                }
            }
            (Err(e), _) => println!("Failed to get coordinates for {from}: {e}"),
            (_, Err(e)) => println!("Failed to get coordinates for {to}: {e}"),
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
