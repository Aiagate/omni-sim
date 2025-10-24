use bevy::prelude::*;

/// Nation entity representing an economic actor in the simulation.
///
/// Each nation has an economy (GDP), resources, and participates in trade.
#[derive(Component)]
pub struct Nation {
    /// Nation name for display purposes
    pub name: String,

    /// Gross Domestic Product - represents the economic size of the nation.
    /// This value increases through successful trade.
    pub gdp: f32,

    /// Resource stock available for export.
    /// Must remain non-negative. Trade is blocked if insufficient resources.
    pub resource_stock: f32,
}

/// Trade route between two nations.
///
/// Defines a persistent trade relationship with regular resource transfers.
#[derive(Component)]
pub struct TradeRoute {
    /// Entity ID of the exporting nation
    pub origin: Entity,

    /// Entity ID of the importing nation
    pub destination: Entity,

    /// Amount of resources transferred per shipment
    pub volume: f32,
}

/// Transport fleet carrying cargo along a trade route.
///
/// Fleets operate continuously, transporting resources when they arrive.
#[derive(Component)]
pub struct TransportFleet {
    /// Current cargo being transported
    pub cargo: f32,

    /// Estimated time of arrival in seconds.
    /// When this reaches 0, trade is executed and the timer resets.
    pub eta: f32,

    /// Entity ID of the trade route this fleet follows
    pub route: Entity,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nation_creation() {
        let nation = Nation {
            name: "Test Nation".into(),
            gdp: 1000.0,
            resource_stock: 500.0,
        };

        assert_eq!(nation.name, "Test Nation");
        assert_eq!(nation.gdp, 1000.0);
        assert_eq!(nation.resource_stock, 500.0);
    }

    #[test]
    fn test_trade_route_creation() {
        use bevy::ecs::world::World;

        let mut world = World::new();
        let origin = world.spawn_empty().id();
        let destination = world.spawn_empty().id();

        let route = TradeRoute {
            origin,
            destination,
            volume: 50.0,
        };

        assert_eq!(route.origin, origin);
        assert_eq!(route.destination, destination);
        assert_eq!(route.volume, 50.0);
    }

    #[test]
    fn test_transport_fleet_creation() {
        use bevy::ecs::world::World;

        let mut world = World::new();
        let route_entity = world.spawn_empty().id();

        let fleet = TransportFleet {
            cargo: 100.0,
            eta: 10.0,
            route: route_entity,
        };

        assert_eq!(fleet.cargo, 100.0);
        assert_eq!(fleet.eta, 10.0);
        assert_eq!(fleet.route, route_entity);
    }
}
