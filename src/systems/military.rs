use bevy::prelude::*;

use crate::components::common::SimName;
use crate::components::economy::Resources;
use crate::components::military::MilitaryStrength;
use crate::components::simulation_event::SimulationEvent;
use crate::tick::CurrentTick;

/// 軍事システム
///
/// 1. 艦隊建造: 鉱物と工業品を消費して艦船を建造
/// 2. 維持コスト: 艦船数に応じたエネルギー消費
pub fn military_system(
    tick: Res<CurrentTick>,
    mut query: Query<(&SimName, &mut MilitaryStrength, &mut Resources)>,
    mut events: EventWriter<SimulationEvent>,
) {
    for (name, mut mil, mut res) in &mut query {
        // === 艦船建造 ===
        // 鉱物 50 + 工業品 30 = 艦船 1 隻
        let buildable_from_minerals = (res.minerals / 50.0).floor() as u32;
        let buildable_from_goods = (res.manufactured_goods / 30.0).floor() as u32;
        let new_ships = buildable_from_minerals.min(buildable_from_goods).min(2); // 1 Tick あたり最大 2 隻

        if new_ships > 0 {
            res.minerals -= new_ships as f64 * 50.0;
            res.manufactured_goods -= new_ships as f64 * 30.0;
            mil.ships += new_ships;
            mil.recalculate();
        }

        // === 維持コスト ===
        // 艦船 1 隻あたり 1.0 エネルギー/Tick
        let upkeep = mil.ships as f64 * 1.0;
        if res.energy >= upkeep {
            res.energy -= upkeep;
        } else {
            // エネルギー不足: 艦船が退役
            let deficit = upkeep - res.energy;
            let ships_lost = (deficit / 1.0).ceil() as u32;
            mil.ships = mil.ships.saturating_sub(ships_lost);
            mil.recalculate();
            res.energy = 0.0;
        }

        // 表示イベントを送出
        events.send(SimulationEvent::MilitaryReport {
            tick: tick.value,
            planet: name.0.clone(),
            ships: mil.ships,
            power: mil.power,
        });
    }
}
