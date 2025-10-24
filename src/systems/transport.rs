use crate::components::{Nation, TradeRoute, TransportFleet};
use bevy::prelude::*;

/// System that manages transport fleet movement and executes trades.
///
/// This system:
/// 1. Decrements the ETA of each fleet based on elapsed time
/// 2. When ETA reaches 0, executes the trade
/// 3. Validates that sufficient resources exist before trading
/// 4. Updates nation resources and GDP
/// 5. Resets the fleet timer for the next cycle
pub fn transport_execution(
    time: Res<Time>,
    mut fleets: Query<&mut TransportFleet>,
    routes: Query<&TradeRoute>,
    mut nations: Query<&mut Nation>,
) {
    for mut fleet in fleets.iter_mut() {
        // Decrement ETA based on elapsed time
        fleet.eta -= time.delta_secs();

        if fleet.eta <= 0.0 {
            // Get route information
            let Ok(route) = routes.get(fleet.route) else {
                println!("Warning: Invalid route entity for fleet");
                fleet.eta = 10.0;
                continue;
            };

            // Prevent same-entity mutable borrow conflict
            if route.origin == route.destination {
                println!("Warning: Origin and destination are the same entity");
                fleet.eta = 10.0;
                continue;
            }

            // Safely get mutable references to both nations
            let Ok([mut origin, mut dest]) =
                nations.get_many_mut([route.origin, route.destination])
            else {
                println!("Warning: Invalid nation entities in trade route");
                fleet.eta = 10.0;
                continue;
            };

            // Validate sufficient resources
            if origin.resource_stock < fleet.cargo {
                println!(
                    "Warning: Insufficient resources - {} has {:.1} but needs {:.1}",
                    origin.name, origin.resource_stock, fleet.cargo
                );
                fleet.eta = 10.0;
                continue;
            }

            // Execute trade
            origin.resource_stock -= fleet.cargo;
            dest.resource_stock += fleet.cargo;
            dest.gdp += fleet.cargo * 0.05;

            println!(
                "Trade completed: {:.1} units from {} to {}",
                fleet.cargo, origin.name, dest.name
            );

            // Reset timer for next cycle
            fleet.eta = 10.0;
        }
    }
}
