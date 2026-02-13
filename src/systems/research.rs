use bevy::prelude::*;
use crate::components::common::{SimName, Nation, BelongsToNation};
use crate::components::economy::Resources;
use crate::components::national_ai::NationalCharacter;
use crate::components::population::Population;
use crate::components::simulation_event::SimulationEvent;
use crate::components::technology::{TechnologyState, TechField};
use crate::tick::CurrentTick;

/// 研究システム
pub fn research_system(
    tick: Res<CurrentTick>,
    config: Res<crate::config::SimulationConfig>,
    mut nation_query: Query<(Entity, &SimName, &mut TechnologyState, &NationalCharacter), With<Nation>>,
    mut planet_query: Query<(&Population, &mut Resources, &BelongsToNation)>,
    mut events: EventWriter<SimulationEvent>,
) {
    // 1. 各国家の研究レートをリセット (1パス目)
    for (_, _, mut tech, _) in &mut nation_query {
        tech.research_rate = 0.0;
    }

    // 2. 惑星を走査し、所属国家の研究レートに加算 (2パス目: O(Planets))
    for (pop, mut res, belongs_to_nation) in &mut planet_query {
        // 各惑星での研究レート計算
        let pop_factor = pop.count / 1_000_000.0;
        let research_cost = 2.0_f64.min(res.manufactured_goods);
        res.manufactured_goods -= research_cost;
        let goods_bonus = research_cost * 0.5;
        let research_rate = pop_factor + goods_bonus;
        
        if let Ok((_, _, mut tech, _)) = nation_query.get_mut(belongs_to_nation.0) {
            tech.research_rate += research_rate;
        }
    }

    // 3. 国家ごとの研究進行処理 (3パス目: O(Nations))
    for (_nation_entity, nation_name, mut tech, character) in &mut nation_query {
        let total_research_rate = tech.research_rate;

        // 研究ポイントの加算
        tech.research_points += total_research_rate;
        let cost = tech.cost_for_next_level(tech.current_research, &config.balance);

        if tech.research_points >= cost {
            tech.research_points -= cost;
            let current_field = tech.current_research;
            // レベルアップの適用
            tech.level_up(current_field);

            // 次の研究分野を性格に基づいて決定
            // 性格に基づいた重み付け
            let mut weights = vec![
                (TechField::Agriculture, 0.5 + character.trade_affinity * 0.5),
                (TechField::Mining, 0.5 + character.expansionism * 0.5),
                (TechField::EnergyTech, 1.0),
                (TechField::Manufacturing, 0.8 + character.research_focus * 0.4),
                (TechField::MilitaryTech, 0.2 + character.aggression * 1.5),
                (TechField::SpaceNavigation, 0.5 + character.expansionism * 1.0),
                (TechField::EnvironmentalTech, 1.0 - character.aggression * 0.5),
                (TechField::NuclearFusion, 0.3 + character.research_focus * 1.5),
            ];

            weights.sort_by(|(f1, w1), (f2, w2)| {
                let score1 = w1 / (tech.level(*f1) as f64 + 1.0);
                let score2 = w2 / (tech.level(*f2) as f64 + 1.0);
                score2.partial_cmp(&score1).unwrap()
            });
            tech.current_research = weights[0].0;

            events.send(SimulationEvent::TechLevelUp {
                tick: tick.value,
                planet: nation_name.0.clone(),
                total_level: tech.total_level(),
                next_field: tech.current_research,
                rate: total_research_rate,
            });
        }

        // レポート送出
        events.send(SimulationEvent::ResearchReport {
            tick: tick.value,
            planet: nation_name.0.clone(),
            levels: [
                tech.agriculture_level,
                tech.mining_level,
                tech.energy_level,
                tech.manufacturing_level,
                tech.military_level,
                tech.navigation_level,
                tech.environmental_level,
                tech.nuclear_fusion_level,
            ],
            rate: total_research_rate,
            progress: tech.research_points,
            cost: tech.cost_for_next_level(tech.current_research, &config.balance),
        });
    }
}
