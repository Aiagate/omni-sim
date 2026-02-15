use bevy::prelude::*;

use crate::components::common::{SimName, Nation, BelongsToNation};
use crate::components::economy::Resources;
use crate::components::military::MilitaryStrength;
use crate::components::simulation_event::SimulationEvent;
use crate::components::technology::TechnologyState;
use crate::tick::CurrentTick;

/// 軍事システム
///
/// 1. 艦隊建造: 鉱物と工業品を消費して艦船を建造
/// 2. 維持コスト: 艦船数に応じたエネルギー消費
pub fn military_system(
    tick: Res<CurrentTick>,
    config: Res<crate::config::SimulationConfig>,
    mut query: Query<(&SimName, &mut MilitaryStrength, &mut Resources, &BelongsToNation)>,
    nation_query: Query<&TechnologyState, With<Nation>>,
    mut events: EventWriter<SimulationEvent>,
) {
    for (name, mut mil, mut res, belongs_to_nation) in &mut query {
        // 所属国家の技術状態を取得
        let tech = nation_query.get(belongs_to_nation.0).ok();
        let mil_tech_level = tech.map(|t| t.level(crate::components::technology::TechId::Military)).unwrap_or(0);
        let shields_level = tech.map(|t| t.level(crate::components::technology::TechId::Shields)).unwrap_or(0);
        let psi_tech_level = tech.map(|t| t.level(crate::components::technology::TechId::PsiTech)).unwrap_or(0);

        // === 艦船建造 ===
        // 基礎コスト: 鉱物 50 + 工業品 30 = 艦船 1 隻
        // 技術によるコスト削減
        let cost_reduction = 1.0 - (mil_tech_level as f64 * config.balance.military_tech_cost_reduction).min(0.5);
        let mineral_cost = 50.0 * cost_reduction;
        let goods_cost = 30.0 * cost_reduction;

        let buildable_from_minerals = (res.minerals / mineral_cost).floor() as u32;
        let buildable_from_goods = (res.manufactured_goods / goods_cost).floor() as u32;
        let new_ships = buildable_from_minerals.min(buildable_from_goods).min(2); // 1 Tick あたり最大 2 隻

        if new_ships > 0 {
            res.minerals -= new_ships as f64 * mineral_cost;
            res.manufactured_goods -= new_ships as f64 * goods_cost;
            mil.ships += new_ships;
            mil.recalculate();
        }

        // 技術による戦力ボーナスを適用 (10%/Lv)
        let power_bonus = 1.0 + (mil_tech_level as f64 * 0.1);
        mil.power *= power_bonus;

        // シールド技術による追加ボーナス (15%/Lv)
        if shields_level > 0 {
            mil.power *= 1.0 + (shields_level as f64 * 0.15);
        }

        // サイオニック技術による追加ボーナス (50%/Lv)
        if psi_tech_level > 0 {
            mil.power *= 1.0 + (psi_tech_level as f64 * 0.5);
        }

        // シナジーボーナス: Military (Shields + PsiTech)
        if let Some(t) = tech {
            mil.power *= 1.0 + t.synergy_bonus(crate::components::technology::TechId::Military);
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
