use crate::components::{Nation, TradeRoute, TransportFleet};
use bevy::prelude::*;

/// Initializes the simulation with two nations, a trade route, and a transport fleet.
///
/// This creates the basic scenario:
/// - Nation A: GDP 1000, Resources 500
/// - Nation B: GDP 800, Resources 300
/// - Trade Route: A -> B, 50 units per shipment
/// - Transport Fleet: 10 second cycle time
pub fn setup(mut commands: Commands) {
    // Create Nation A
    let nation_a = commands
        .spawn(Nation {
            name: "Nation A".into(),
            gdp: 1000.0,
            resource_stock: 500.0,
        })
        .id();

    // Create Nation B
    let nation_b = commands
        .spawn(Nation {
            name: "Nation B".into(),
            gdp: 800.0,
            resource_stock: 300.0,
        })
        .id();

    // Create trade route from A to B
    let route = commands
        .spawn(TradeRoute {
            origin: nation_a,
            destination: nation_b,
            volume: 50.0,
        })
        .id();

    // Create transport fleet
    commands.spawn(TransportFleet {
        cargo: 50.0,
        eta: 10.0,
        route,
    });

    println!("Trade simulation initialized:");
    println!("  - Nation A: GDP={}, Resources={}", 1000.0, 500.0);
    println!("  - Nation B: GDP={}, Resources={}", 800.0, 300.0);
    println!("  - Trade Route: A -> B ({} units per cycle)", 50.0);
}
