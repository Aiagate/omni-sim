use bevy::prelude::*;

// Re-export the components and systems for testing
// In a real project, these would be in a lib.rs
mod components {
    include!("../src/components.rs");
}

mod systems {
    pub mod transport {
        include!("../src/systems/transport.rs");
    }
    pub use transport::transport_execution;
}

use components::{Nation, TradeRoute, TransportFleet};
use systems::transport_execution;

#[test]
fn test_trade_execution() {
    let mut app = App::new();

    // Add Time plugin for delta_secs
    app.add_plugins(MinimalPlugins);

    // Create two nations
    let nation_a = app
        .world_mut()
        .spawn(Nation {
            name: "Nation A".into(),
            gdp: 1000.0,
            resource_stock: 500.0,
        })
        .id();

    let nation_b = app
        .world_mut()
        .spawn(Nation {
            name: "Nation B".into(),
            gdp: 800.0,
            resource_stock: 300.0,
        })
        .id();

    // Create trade route
    let route = app
        .world_mut()
        .spawn(TradeRoute {
            origin: nation_a,
            destination: nation_b,
            volume: 50.0,
        })
        .id();

    // Create fleet with ETA at 0 for immediate execution
    app.world_mut().spawn(TransportFleet {
        cargo: 50.0,
        eta: 0.0,
        route,
    });

    // Add the transport system
    app.add_systems(Update, transport_execution);

    // Run the app once to trigger the trade
    app.update();

    // Verify Nation A's resources decreased
    let nation_a_stock = app
        .world()
        .get::<Nation>(nation_a)
        .map(|n| n.resource_stock);
    assert_eq!(nation_a_stock, Some(450.0));

    // Verify Nation B's resources increased
    let nation_b_stock = app
        .world()
        .get::<Nation>(nation_b)
        .map(|n| n.resource_stock);
    assert_eq!(nation_b_stock, Some(350.0));

    // Verify Nation B's GDP increased
    let nation_b_gdp = app.world().get::<Nation>(nation_b).map(|n| n.gdp);
    assert_eq!(nation_b_gdp, Some(802.5));
}

#[test]
fn test_insufficient_resources() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Nation A with insufficient resources
    let nation_a = app
        .world_mut()
        .spawn(Nation {
            name: "Nation A".into(),
            gdp: 1000.0,
            resource_stock: 10.0, // Insufficient
        })
        .id();

    let nation_b = app
        .world_mut()
        .spawn(Nation {
            name: "Nation B".into(),
            gdp: 800.0,
            resource_stock: 300.0,
        })
        .id();

    let route = app
        .world_mut()
        .spawn(TradeRoute {
            origin: nation_a,
            destination: nation_b,
            volume: 50.0,
        })
        .id();

    app.world_mut().spawn(TransportFleet {
        cargo: 50.0,
        eta: 0.0,
        route,
    });

    app.add_systems(Update, transport_execution);
    app.update();

    // Trade should not execute - resources should remain unchanged
    let nation_a_stock = app
        .world()
        .get::<Nation>(nation_a)
        .map(|n| n.resource_stock);
    assert_eq!(nation_a_stock, Some(10.0));

    let nation_b_stock = app
        .world()
        .get::<Nation>(nation_b)
        .map(|n| n.resource_stock);
    assert_eq!(nation_b_stock, Some(300.0));
}
