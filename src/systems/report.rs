use crate::components::Nation;
use bevy::prelude::*;

/// Periodically reports the current state of all nations.
///
/// This system displays GDP and resource stock for each nation,
/// allowing observation of the simulation's progress.
pub fn report(nations: Query<&Nation>) {
    for nation in nations.iter() {
        println!(
            "{}: GDP={:.1}, Resources={:.1}",
            nation.name, nation.gdp, nation.resource_stock
        );
    }
}
