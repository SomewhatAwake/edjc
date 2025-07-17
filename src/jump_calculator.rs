/*!
Jump calculation logic for Elite Dangerous.

This module handles the calculation of jump routes between systems,
taking into account ship jump ranges and stellar phenomena that
affect jump range (neutron stars and white dwarfs).
*/

use anyhow::Result;
use log::debug;

use crate::types::{JumpResult, SystemCoordinates};

/// Jump route calculator
#[derive(Debug)]
pub struct JumpCalculator;

/// Types of stellar phenomena that affect jump range
#[derive(Debug, Clone, Copy)]
pub enum StellarBoost {
    None,
    WhiteDwarf,  // 1.5x multiplier
    NeutronStar, // 4.0x multiplier
}

impl StellarBoost {
    /// Get the jump range multiplier for this boost type
    pub fn multiplier(self) -> f64 {
        match self {
            StellarBoost::None => 1.0,
            StellarBoost::WhiteDwarf => 1.5,
            StellarBoost::NeutronStar => 4.0,
        }
    }
}

impl JumpCalculator {
    /// Create a new jump calculator
    pub fn new() -> Self {
        Self
    }

    /// Calculate the optimal route between two systems
    pub fn calculate_route(
        &self,
        from: &SystemCoordinates,
        to: &SystemCoordinates,
        base_jump_range: f64,
    ) -> Result<JumpResult> {
        let total_distance = self.calculate_distance(from, to);

        debug!(
            "Calculating route from {} to {} ({}ly)",
            from.name, to.name, total_distance
        );

        // Calculate jumps for different scenarios
        let normal_jumps = self.calculate_jumps_direct(total_distance, base_jump_range);

        // Check if we can use neutron highway
        let neutron_jumps = self.calculate_jumps_with_boost(
            total_distance,
            base_jump_range,
            StellarBoost::NeutronStar,
        );

        // Check if white dwarf route is better
        let white_dwarf_jumps = self.calculate_jumps_with_boost(
            total_distance,
            base_jump_range,
            StellarBoost::WhiteDwarf,
        );

        // Determine the best route
        let (jumps, route_type) =
            if neutron_jumps < normal_jumps && neutron_jumps < white_dwarf_jumps {
                (neutron_jumps, "neutron highway".to_string())
            } else if white_dwarf_jumps < normal_jumps {
                (white_dwarf_jumps, "white dwarf assisted".to_string())
            } else {
                (normal_jumps, "direct".to_string())
            };

        Ok(JumpResult {
            jumps,
            total_distance,
            route_type,
            from_system: from.name.clone(),
            to_system: to.name.clone(),
        })
    }

    /// Calculate distance between two systems in 3D space
    fn calculate_distance(&self, from: &SystemCoordinates, to: &SystemCoordinates) -> f64 {
        let dx = to.x - from.x;
        let dy = to.y - from.y;
        let dz = to.z - from.z;

        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Calculate jumps using direct routing (no boosts)
    fn calculate_jumps_direct(&self, distance: f64, jump_range: f64) -> u32 {
        (distance / jump_range).ceil() as u32
    }

    /// Calculate jumps using stellar boost routing
    fn calculate_jumps_with_boost(
        &self,
        distance: f64,
        base_jump_range: f64,
        boost: StellarBoost,
    ) -> u32 {
        // Simplified calculation assuming we can find boost stars along the route
        // In reality, this would require pathfinding through actual stellar data

        let boosted_range = base_jump_range * boost.multiplier();

        // Assume we need to make one extra jump to reach a boost star
        // and can use boosted jumps for most of the journey
        let boost_overhead = 1; // Extra jump to reach boost star
        let boosted_jumps = ((distance * 0.8) / boosted_range).ceil() as u32;
        let normal_jumps = ((distance * 0.2) / base_jump_range).ceil() as u32;

        boost_overhead + boosted_jumps + normal_jumps
    }

    /// Estimate if a neutron highway route is available
    pub fn estimate_neutron_availability(&self, distance: f64) -> bool {
        // Neutron stars are relatively rare, so only worth it for longer routes
        distance > 500.0
    }

    /// Estimate if white dwarf assistance is worthwhile
    pub fn estimate_white_dwarf_availability(&self, distance: f64) -> bool {
        // White dwarfs are more common than neutron stars
        distance > 150.0
    }

    /// Calculate fuel usage for a route (approximate)
    pub fn estimate_fuel_usage(&self, jumps: u32, jump_range: f64) -> f64 {
        // Simplified fuel calculation
        // Real calculation would depend on ship mass, FSD rating, etc.
        let base_fuel_per_jump = 2.0; // tons
        let range_factor = (jump_range / 20.0).max(0.5); // Normalize around 20ly

        jumps as f64 * base_fuel_per_jump * range_factor
    }

    /// Get detailed route information
    pub fn get_route_details(
        &self,
        from: &SystemCoordinates,
        to: &SystemCoordinates,
        base_jump_range: f64,
    ) -> Result<RouteDetails> {
        let result = self.calculate_route(from, to, base_jump_range)?;
        let fuel_usage = self.estimate_fuel_usage(result.jumps, base_jump_range);

        Ok(RouteDetails {
            result: result.clone(),
            estimated_fuel_usage: fuel_usage,
            estimated_time_minutes: result.jumps as f64 * 2.0, // 2 minutes per jump average
            can_use_neutron: self.estimate_neutron_availability(result.total_distance),
            can_use_white_dwarf: self.estimate_white_dwarf_availability(result.total_distance),
        })
    }
}

/// Detailed route information
#[derive(Debug, Clone)]
pub struct RouteDetails {
    pub result: JumpResult,
    pub estimated_fuel_usage: f64,
    pub estimated_time_minutes: f64,
    pub can_use_neutron: bool,
    pub can_use_white_dwarf: bool,
}

impl Default for JumpCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_calculation() {
        let calc = JumpCalculator::new();

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

        let distance = calc.calculate_distance(&sol, &alpha_centauri);
        assert!((distance - 3.34).abs() < 0.1); // Should be about 3.34 ly
    }

    #[test]
    fn test_jump_calculation() {
        let calc = JumpCalculator::new();

        let jumps = calc.calculate_jumps_direct(100.0, 25.0);
        assert_eq!(jumps, 4); // 100ly / 25ly = 4 jumps

        let jumps = calc.calculate_jumps_direct(99.0, 25.0);
        assert_eq!(jumps, 4); // 99ly / 25ly = 3.96, rounded up to 4
    }

    #[test]
    fn test_stellar_boost_multipliers() {
        assert_eq!(StellarBoost::None.multiplier(), 1.0);
        assert_eq!(StellarBoost::WhiteDwarf.multiplier(), 1.5);
        assert_eq!(StellarBoost::NeutronStar.multiplier(), 4.0);
    }
}
